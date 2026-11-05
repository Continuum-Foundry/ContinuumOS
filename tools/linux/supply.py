"""Supply checksum-locked Linux tools locally; never rewrite expected hashes.

Use --download --build for explicit acquisition/build, --fetch for Cargo cache
supply. With no flags, validate retained inputs without network or installation.
"""
import argparse
import bz2
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / 'config/m1-inputs.linux.lock.json'

def run(*args, **kw):
    subprocess.run(list(map(str, args)), check=True, **kw)

def verify(path, pin):
    if path.stat().st_size != pin['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest() != pin['sha256']:
        raise ValueError(f'Hash/size mismatch: {path}; retained for diagnosis')

def acquire(pin):
    path = ROOT / pin['path']
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(pin['url'], timeout=120) as response, path.open('wb') as out:
            shutil.copyfileobj(response, out)
    verify(path, pin)
    return path

def assemble():
    base = ROOT / 'work/tools/linux'
    for name in ('bin', 'lib', 'libexec', 'share'):
        (base / name).mkdir(parents=True, exist_ok=True)
    sources = ROOT / 'work/linux/src/qemu-git'
    for name, source in [('qemu-system-x86_64', sources / 'build/qemu-system-x86_64'), ('xorriso', ROOT / 'work/linux/sysroot/usr/bin/xorriso')]:
        shutil.copy2(source, base / 'libexec' / name)
        env = os.environ | {'LD_LIBRARY_PATH': str(ROOT / 'work/linux/sysroot/usr/lib/x86_64-linux-gnu')}
        for line in subprocess.check_output(['ldd', source], env=env, text=True).splitlines():
            if '=>' not in line:
                continue
            path = line.split('=>', 1)[1].strip().split()[0]
            if path.startswith('/') and not Path(path).name.startswith('libc.so'):
                shutil.copy2(path, base / 'lib' / Path(path).name, follow_symlinks=True)
        wrapper = '#!/bin/sh\nset -eu\nbase=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)\nexport LD_LIBRARY_PATH="$base/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"\nexec "$base/libexec/' + name + '" ' + ('-L "$base/share" ' if name.startswith('qemu') else '') + '"$@"\n'
        dest = base / 'bin' / name
        dest.write_text(wrapper)
        dest.chmod(0o755)
    for path in (sources / 'pc-bios').iterdir():
        if path.is_file() and path.suffix != '.bz2' and path.name != 'meson.build':
            shutil.copy2(path, base / 'share' / path.name)
    for name in ('edk2-x86_64-code.fd', 'edk2-i386-vars.fd'):
        (base / 'share' / name).write_bytes(bz2.decompress((sources / 'pc-bios' / (name + '.bz2')).read_bytes()))

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--download', action='store_true')
    parser.add_argument('--build', action='store_true')
    parser.add_argument('--fetch', action='store_true')
    args = parser.parse_args()
    os.chdir(ROOT)
    lock = json.loads(LOCK.read_text())
    if args.download:
        for pin in lock['acquisitions']:
            path = acquire(pin)
            if path.suffix == '.deb':
                run('dpkg-deb', '-x', path, ROOT / 'work/linux/sysroot')
            elif path.name == 'powershell.tar.gz':
                dest = ROOT / 'work/tools/powershell'
                dest.mkdir(parents=True, exist_ok=True)
                with tarfile.open(path) as archive:
                    archive.extractall(dest, filter='data')
                (dest / 'pwsh').chmod(0o755)
            elif path.name == 'limine-binary.zip':
                with zipfile.ZipFile(path) as archive:
                    archive.extractall(ROOT / 'work/tools/limine')
            elif path.suffix == '.crate':
                with tarfile.open(path) as archive:
                    archive.extractall(ROOT / 'work/tools/bindings', filter='data')
        windows = json.loads((ROOT / 'config/m1-inputs.lock.json').read_text())
        for pin in windows['files']:
            if 'work/tools/limine-protocol/' not in pin['path']:
                continue
            name = Path(pin['path']).name
            remote = 'include/limine.h' if name == 'limine.h' else name
            acquire(pin | {'url': f"https://raw.githubusercontent.com/Limine-Bootloader/limine-protocol/{lock['protocol_commit']}/{remote}"})
        fonts = json.loads((ROOT / 'fixtures/m2/text-v1/fonts.lock.json').read_text())
        for pin in fonts['fonts']:
            acquire(pin)
    if args.build:
        source = ROOT / 'work/linux/src/qemu-git'
        if not source.exists():
            run('git', 'clone', '--depth', '1', '--branch', 'v11.1.0', 'https://github.com/qemu/qemu.git', source)
        if subprocess.check_output(['git', '-C', source, 'rev-parse', 'HEAD'], text=True).strip() != lock['qemu_source_commit']:
            raise ValueError('QEMU checkout differs from reviewed commit')
        wheels = [str(ROOT / pin['path']) for pin in lock['acquisitions'] if pin['path'].endswith('.whl')]
        run(sys.executable, '-m', 'pip', 'install', '--no-index', '--no-deps', '--target', ROOT / 'work/linux/python', *wheels)
        env = os.environ | {
            'PYTHONPATH': str(ROOT / 'work/linux/python'),
            'PATH': str(ROOT / 'work/linux/python/bin') + os.pathsep + os.environ['PATH'],
            'PKG_CONFIG_SYSROOT_DIR': str(ROOT / 'work/linux/sysroot'),
            'PKG_CONFIG_LIBDIR': ':'.join(map(str, [ROOT / 'work/linux/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig', ROOT / 'work/linux/sysroot/usr/share/pkgconfig', Path('/usr/lib/x86_64-linux-gnu/pkgconfig'), Path('/usr/share/pkgconfig')])),
            'LD_LIBRARY_PATH': str(ROOT / 'work/linux/sysroot/usr/lib/x86_64-linux-gnu'),
        }
        run('./configure', *lock['build']['configure'], cwd=source, env=env)
        run('make', '-j4', 'qemu-system-x86_64', cwd=source, env=env)
        assemble()
    if args.fetch:
        env = os.environ | {'CARGO_HOME': str(ROOT / 'work/toolchains/cargo'), 'RUSTUP_HOME': str(ROOT / 'work/toolchains/rustup')}
        env['PATH'] = str(Path(env['CARGO_HOME']) / 'bin') + os.pathsep + env['PATH']
        for manifest in ('Cargo.toml', 'experiments/dependencies/Cargo.toml', 'experiments/ui-link/Cargo.toml', 'experiments/boot-bindings/Cargo.toml', 'experiments/sdk-client/Cargo.toml', 'experiments/sdk-service/Cargo.toml', 'experiments/sdk-stress/Cargo.toml'):
            run('cargo', 'fetch', '--locked', '--config', 'net.offline=false', '--manifest-path', manifest, env=env)
    for pin in lock['files']:
        verify(ROOT / pin['path'], pin)
    print(f"Verified {len(lock['files'])} locked Linux/common runtime inputs")

if __name__ == '__main__':
    main()
