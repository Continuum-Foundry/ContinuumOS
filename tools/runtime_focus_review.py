"""Focused runtime artifact reviews, without claiming old aggregate story gates.

Uses the existing independent serial, ELF, ISO and mutation oracles. Source-set
and tool freshness are enforced by story_validation around execution and review.
"""
from pathlib import Path
import os
import shutil
import subprocess
import p3_acceptance as base

require, read, digest = base.require, base.read, base.digest


def review(key, folder):
    folder = Path(folder).resolve()
    require(folder.is_relative_to(base.ROOT), 'artifact outside workspace')
    inputs = read(folder / 'build-inputs.json')
    require(bool(inputs['sources']), 'missing measured source inputs')
    for name, expected in inputs['sources'].items():
        source = (base.ROOT / name).resolve()
        require(source.is_relative_to(base.ROOT) and source.is_file()
                and digest(source) == expected, 'stale runtime source: ' + name)
    # Detect additions/deletions too, matching the directory inputs collected by
    # xtask, rather than accepting a stale artifact whose old files still match.
    for name in ('kernel', 'tools/xtask', 'tools/windows', 'tools/linux',
                 'experiments/user-images', 'sdk/continuum-rt', 'experiments/sdk-client',
                 'experiments/sdk-service', 'experiments/sdk-stress'):
        current = {p.relative_to(base.ROOT).as_posix() for p in (base.ROOT / name).rglob('*')
                   if p.is_file() and '__pycache__' not in p.parts}
        measured = {p for p in inputs['sources'] if p.startswith(name + '/')}
        require(current == measured, 'changed runtime source set: ' + name)
    require(shutil.which('rustc'), 'focused runtime review requires Enter-Dev compiler context')
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], cwd=base.ROOT, text=True).strip())
    compiler = sysroot / 'bin' / ('rustc.exe' if os.name == 'nt' else 'rustc')
    require(digest(compiler) == inputs['rustc_sha256'], 'changed runtime compiler')
    from story_validation import RUNTIME
    _, story = RUNTIME[key]
    module = base.load_reviewer(story)
    if key in ('capabilities', 'channels', 'waits', 'supervisor', 'policy', 'routes'):
        report, helpers = module.focused(folder)
        return dict(status='passed', scope='focused regression only', audit=report, codegen=helpers)
    if key in ('sdk', 'sdk-runtime'):
        return module.focused(folder, key == 'sdk-runtime')
    memory = base.load_reviewer('P3-02')
    manifest = read(folder / 'build-manifest.json')
    require(read(folder / 'result.json')['status'] == 'passed', 'failed runtime case')
    require(manifest['kernel_sha256'] == digest(folder / 'kernel.elf'), 'changed kernel ELF')
    memory.elf_audit(folder / 'kernel.elf')
    def launch(name):
        run = folder / (name + '-run')
        result, context, case = memory.lifecycle(run)
        require(result['emulator_test_exit'] == 33 and result['build_id'] == manifest['build_id']
                and context['fixture'] == name, 'guest exit/identity')
        rows = memory.records((run / 'serial.log').read_bytes(), manifest['build_id'], result['host_session_id'])
        return result, context, case, rows
    if key == 'memory':
        require(manifest['flavor'] == 'runtime-memory-test', 'wrong memory flavor')
        result, _, _, rows = launch('owned-memory')
        return dict(status='passed', scope='focused memory regression only',
                    audit=memory.table_audit(rows), mutations=memory.mutation_checks(rows),
                    iso_sha256=result['iso_sha256'])
    if key == 'loader':
        require(manifest['flavor'] == 'static-loader-test', 'wrong loader flavor')
        result, _, case, rows = launch('static-loader')
        names = ('user_a', 'user_b', 'service')
        images = [module.expected_pages(folder / (n + '.elf')) for n in names]
        descriptors = read(folder / 'user-images.json')
        require(descriptors['trusted_descriptors_sha256'] == digest(folder / 'trusted-images.rs'), 'changed descriptor')
        require(len(descriptors['images']) == 3, 'missing loader image')
        for name, image, descriptor in zip(names, images, descriptors['images']):
            require(image[2]['sha256'] == descriptor['sha256']
                    == digest(folder / ('static-loader-package/extracted payload/boot/' + name + '.elf'))
                    == case['package']['payload']['boot/' + name + '.elf'], 'loader ISO/image mismatch')
            require(not (folder / (name + '-llvm-nm.stdout.log')).read_text().strip(), 'undefined user symbol')
            args = read(folder / (name + '-build.command.json'))['arguments']
            require(all(flag in args for flag in ('--target=x86_64-unknown-none', 'code-model=small',
                        'relocation-model=static', 'no-redzone=yes', 'panic=abort', 'opt-level=1', name)), 'loader compiler flags')
        return dict(status='passed', scope='focused loader regression only',
                    audit=module.audit(rows, images), mutations=module.mutation_checks(rows, images))
    if key == 'boundary':
        require(manifest['flavor'] == 'user-boundary-test', 'wrong boundary flavor')
        audits, mutations, sessions = [], [], set()
        for group, suffix in enumerate('abcd'):
            result, _, case, rows = launch('user-boundary-' + suffix)
            sessions.add(result['host_session_id'])
            audits.append(module.audit(rows, group))
            mutations.extend(module.mutation_checks(rows, group))
            for image in read(folder / 'user-images.json')['images']:
                name = image['name']
                require(image['sha256'] == digest(folder / (name + '.elf'))
                        == digest(folder / ('user-boundary-' + suffix + '-package/extracted payload/boot/' + name + '.elf'))
                        == case['package']['payload']['boot/' + name + '.elf'], 'boundary ISO/image mismatch')
        require(len(sessions) == 4, 'reused boundary session')
        return dict(status='passed', scope='focused boundary regression only', audit=audits,
                    mutations=mutations, codegen=module.codegen(folder, syscall_scratch_guard=True))
    if key == 'scheduler':
        require(manifest['flavor'] == 'scheduler-test', 'wrong scheduler flavor')
        audits, mutations, sessions = [], [], set()
        for suffix, traced in (('on', True), ('off', False)):
            result, context, case, rows = launch('scheduler-' + suffix)
            args = context['arguments']
            require(args[args.index('-cpu') + 1] == 'qemu64-v1,+apic'
                    and args[args.index('-accel') + 1] == 'tcg'
                    and args[args.index('-machine') + 1] == 'pc-q35-11.1,hpet=on', 'scheduler VM profile')
            sessions.add(result['host_session_id'])
            audits.append(module.audit(rows, traced, True))
            mutations.extend(module.mutations(rows, traced, True))
            for image in read(folder / 'user-images.json')['images']:
                require(case['package']['payload']['boot/' + image['name'] + '.elf'] == image['sha256'], 'scheduler ISO image')
        require(len(sessions) == 2, 'reused scheduler session')
        _, context, _, rows = launch('scheduler-cpu')
        args = context['arguments']
        denials = [f for e, f in rows if e == 'm1.scheduler_cpu_rejected']
        require('-sse2' in args[args.index('-cpu') + 1] and context['vm']['cpu'].endswith(',-sse2')
                and len(denials) == 1 and denials[0]['user_entries'] == '0'
                and not int(denials[0]['cpuid_edx'], 0) & (1 << 26)
                and not any(e == 'm1.task_start' for e, _ in rows), 'missing real CPU denial')
        helpers, instructions = module.codegen(folder)
        require(helpers['exception_common'].count('pushq') == 16
                and helpers['exception_common'].count('popq') == 15
                and '%cr0' in helpers['exception_common'], 'exception frame/TS guard')
        return dict(status='passed', scope='focused scheduler regression only', audit=audits,
                    mutations=mutations, codegen=helpers, instructions=instructions)
    raise ValueError('unknown runtime check: ' + key)
