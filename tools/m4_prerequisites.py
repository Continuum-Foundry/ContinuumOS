"""P4-01 locked host planning inputs and bounded existing lifecycle fixture.

No guest desktop, driver, authentication construction or ABI freeze is supplied.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
import uuid

import m3_runtime as lifecycle

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / 'config/m4-profiles.json'
PROFILE_SHA256 = '333b7fa2170877508d737d9f0fb97fe190a082fc5562fea6f016484798704f1a'
SDK_SHA256 = 'debe57866ed308f46ff2f48f5065e739c7457d0f28d0e8dccc93d42abd4c891b'
digest, require, read, save = lifecycle.digest, lifecycle.require, lifecycle.read_json, lifecycle.save


def validate(profile):
    canonical = json.dumps(profile, sort_keys=True, separators=(',', ':')).encode()
    require(hashlib.sha256(canonical).hexdigest() == PROFILE_SHA256,
            'unsupported M4 profile/type/field/measurement revision')
    require(digest(ROOT / 'contracts/sdk-v1.json') == SDK_SHA256, 'frozen SDK changed')
    return profile


def plan(profile, width, height, development=False, port=None):
    validate(profile)
    require(type(development) is bool, 'development must be explicit bool')
    require((width, height) in ((1280, 720), (1920, 1080)), 'unsupported output')
    require(type(width) is int and type(height) is int, 'invalid geometry type')
    if development:
        require(type(port) is int and 1024 <= port <= 65535, 'loopback port required')
    else:
        require(port is None, 'disabled development cannot expose a port')
    args = ['-vga', 'none', '-device', f'VGA,xres={width},yres={height},vgamem_mb=16',
            '-device', 'virtio-keyboard-pci,disable-legacy=on',
            '-device', 'virtio-tablet-pci,disable-legacy=on']
    if development:
        args += ['-chardev', f'socket,id=continuumdev,host=127.0.0.1,port={port},server=on,wait=off,nodelay=on',
                 '-device', 'virtio-serial-pci,id=devctl,disable-legacy=on,max_ports=2',
                 '-device', 'virtserialport,bus=devctl.0,nr=1,chardev=continuumdev,name=org.continuum.dev']
    return {'scope': 'planned argv delta only; no VM/device delivery acceptance', 'arguments': args,
            'bootstrap_development_authority': development,
            'diagnostic_serial': 'preserve independent baseline COM1',
            'device_grants': ['dev-mutator'] if development else []}


def fixture(profile_path, image, sha256, case, interpreter=None):
    folder = ROOT / 'build/m4/P4-01' / f'fixture-{time.time_ns()}-{uuid.uuid4().hex}'
    folder.mkdir(parents=True)
    report = {'scope': 'host-lifecycle-only; image never executed', 'status': 'rejected',
              'outcome': 'validation-rejected', 'launched': False, 'cleanup': 'not-launched',
              'profile_path': str(profile_path), 'runner_sha256': digest(__file__)}
    try:
        validate(read(profile_path))
        report['profile_sha256'] = digest(profile_path)
        child_folder, child = lifecycle.run(lifecycle.PROFILE, image, sha256, case, interpreter)
        report.update(outcome=child['outcome'], launched=child['launched'], cleanup=child['cleanup'],
                      child_result=str(child_folder / 'result.json'), child_result_sha256=digest(child_folder / 'result.json'))
        report['status'] = 'passed' if child['outcome'] == 'complete' else 'rejected'
    except (ValueError, OSError) as error:
        report['error'] = str(error)
    save(folder / 'result.json', report)
    return folder, report


def inventory(folder):
    folder.mkdir(parents=True)
    pins = read(ROOT / 'config/m1-inputs.lock.json')
    pin = next(item for item in pins['files'] if item['id'] == 'qemu')
    exe = ROOT / pin['path']
    require(digest(exe) == pin['sha256'], 'QEMU identity mismatch')
    commands = [('version', ['--version']), ('devices', ['-device', 'help'])]
    commands += [(name, ['-device', name + ',help']) for name in
                 ('VGA', 'virtio-keyboard-pci', 'virtio-tablet-pci', 'virtio-serial-pci', 'virtserialport')]
    rows = []
    for name, args in commands:
        result = subprocess.run([str(exe), *args], stdin=subprocess.DEVNULL, capture_output=True,
                                timeout=5, creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == 'win32' else 0)
        data = result.stdout + result.stderr
        require(result.returncode == 0 and len(data) <= 65536, 'device help failed/oversized')
        (folder / (name + '.txt')).write_bytes(data)
        rows.append({'name': name, 'arguments': args, 'exit_code': result.returncode,
                     'sha256': digest(folder / (name + '.txt')), 'bytes': len(data)})
    report = {'scope': 'pinned executable help only; no VM launched', 'qemu_sha256': pin['sha256'], 'commands': rows}
    save(folder / 'inventory.json', report)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    sub.add_parser('validate')
    device = sub.add_parser('devices'); device.add_argument('--output', type=Path, required=True)
    child = sub.add_parser('fixture')
    child.add_argument('--config', type=Path, default=PROFILE)
    child.add_argument('--image', type=Path, required=True)
    child.add_argument('--sha256', required=True)
    child.add_argument('--case', default='success')
    args = parser.parse_args()
    if args.command == 'validate':
        validate(read(PROFILE)); print('M4 fixed profile validated'); return 0
    if args.command == 'devices':
        inventory(args.output); print(args.output); return 0
    folder, report = fixture(args.config, args.image, args.sha256, args.case)
    print(folder / 'result.json')
    return 0 if report['status'] == 'passed' else 1


if __name__ == '__main__':
    sys.exit(main())
