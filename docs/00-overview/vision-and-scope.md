# Vision and scope

## User intent

The conversation asks for an OS from scratch, a distributable ISO, modern
languages rather than a C/assembly-heavy codebase, an exceptional but efficient UI,
and source changes visible inside a running VM without restarting the machine.
Live evolution is a structural property of the OS, not merely a file watcher.

The working project folder is ContinuumOS. The user has not selected a CPU, VM product, final language, visual style,
license, budget, release date, or physical hardware. Recommendations below are
explicit assumptions, not invented preferences.

## Working assumptions

| Property | Proposed starting point | Reason |
| --- | --- | --- |
| Product | VM-first experimental desktop for development and creative interaction | Makes a testable first system possible |
| CPU | x86-64, single CPU first | Fits a Windows PC workflow and bootable optical image requirement |
| VM | QEMU, pinned Q35 machine version, OVMF UEFI | Scriptable devices and debugging; portability of exact configuration must be tested |
| Host | Windows orchestration; optional Linux build environment | Current workspace is Windows; guest does not inherit the build host's kernel |
| System language | Rust, freestanding kernel | Modern features with a supported bare-metal target |
| Architecture | Small original kernel with hardware-isolated services | Supports replaceable services without granting all code kernel privilege |
| Initial graphics | Software rendering; explicit acceleration milestone | A usable framebuffer does not imply a GPU driver |
| UI authoring | Compare Slint and a minimal custom Rust core; select by prototype evidence | Avoids an untested full-toolkit commitment |
| Live code | Native process replacement first; Wasm interpreter spike second | OS services need isolation regardless of the app language |
| Distribution | UEFI bootable ISO plus optional writable data image | Read-only distribution and mutable user state have different lifecycles |

## What “from scratch” means

We author the kernel, OS object model, service composition, update controller,
desktop behavior, and integration. We do not use Linux, BSD, Redox, seL4, or another
existing kernel as the foundation. Existing systems are references.

Reusing a compiler, firmware, bootloader, font, cryptographic library, or a bounded
portable library is compatible with this working interpretation. Record provenance
and license obligations. The initial recommendation reuses Limine and OVMF; this
is not a claim that the entire boot chain is first-party or free of upstream C.
If the user later means every boot instruction must be authored here, follow the
own-UEFI-loader alternative and revise ADR-0002. No practical OS begins without
some CPU/platform firmware assumptions.

The Rust language can use a C-compatible ABI without writing C source. Likewise,
host use of an ISO tool implemented in C does not make the guest a C-based OS.
The working preference is no first-party C and a small, audited amount of assembly.

## Product characteristics

The desktop should feel direct, coherent, quiet, and legible. Input should receive
feedback promptly; applications should not be able to freeze the whole shell.
Users should understand whether a change applied, was rejected, or needs a reboot.
Documents should survive app upgrades and failures. Permission prompts should
correspond to actual authority, not decorative UI. Idle components should sleep.
Visual features must have measured costs and graceful lower-cost modes.

Developer builds need rapid changes, inspectors, traces, reversible updates, and
clear diagnostics. Distribution builds need reproducibility, integrity checking,
disabled development authority, predictable recovery, and explicit hardware support.
Neither build is described as production-secure until its assurance gates pass.

Typed actions and scoped context are core platform interfaces. Local AI is an optional
isolated service after the live-reload baseline, not a boot/desktop/release dependency.
See [the reconciliation](reconciled-plan.md).

## First release scope

A reproducible VM image with keyboard/mouse, basic compositing, launcher, settings,
file browser, text editor, diagnostics console, persistent files on a virtual disk,
and supported live UI/app/service updates. M8 additionally requires an offline,
graphical, keyboard-accessible VM installer, local account setup/login, boot/system
repair and reinstall preserving compatible user data and ownership. The installed
disk must boot without the ISO. Earlier live previews have separately scoped review
and do not complete M8. Booting, isolation, storage recovery, installation/account
lifecycle and updates must be demonstrated with tests. Networking starts as a controlled
development facility; public browsing is not a first-release assumption.

## Later desktop and platform delivery

Preserve M8's installable experimental VM release. M9–M13 add everyday file/
desktop workflows, native app/software management, independent document/settings
backup, administration/shutdown, networking and audio/assistive integration.
M14 accepts a coherent usable VM desktop; M15 qualifies one named UEFI PC only.
See the [feature matrix](../05-delivery/desktop-feature-matrix.md) for exact release
classification, ownership and future acceptance. A working browser is not yet a
release promise: M16 researches Windows compatibility and M17 compares browser
routes before any separate production-browser decision. Optional GPU/SMP/Wasm/AI
have M18–M21 gates and do not block M8/M14.

## Deliberately later

General Windows/Linux binary compatibility, a full browser engine, arbitrary
GPU drivers, Wi-Fi, USB device coverage, laptop suspend, full-disk encryption,
network identity/roaming, secure boot provisioning, hard real-time guarantees, and general
kernel replacement. Each is scoped in the feature matrix and risk register.
These are substantial product programs, not minor polish items.

## Definition of this research deliverable

Coverage is complete when the boot-to-desktop-to-update-to-recovery lifecycle has
an architecture, alternatives, evidence, interfaces, acceptance criteria, and an
ordered implementation path. Research does not prove our code works. Unknowns
are converted to bounded experiments rather than described as solved.
