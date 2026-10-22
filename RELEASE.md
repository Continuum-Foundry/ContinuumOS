# Release and publication gates

There is no approved production release. Local Cargo `0.1.0` package versions
and passing engineering tests are not release declarations. Original project
material is all rights reserved; no public binary download is authorized merely
by building an ISO. An administrator must record a release decision and verify
the relevant rights holders' distribution permission.

The [U.S.-only handling and remote-transfer hold](EXPORT_CONTROL.md) blocks all
new uploads and external distribution now. No foreign release or transfer is
authorized under the current instruction, even if a legal exception might exist.
Any proposed domestic delivery still needs verified end-to-end containment and
completed legal classification/access review.

## Required review record

For a proposed source or binary release, retain a review identifying the candidate
commit/tag, reviewers, exact artifacts, intended audience, applicable agreements,
passed gates, remaining limitations, and the authorization to publish. Do not
retrospectively rewrite failed runs or treat a dirty-worktree boot as a clean release.

The owner's 2026-10-21 [roadmap amendment](docs/00-overview/reconciled-plan.md#installer-and-accounts-amendment--2026-10-21)
requires M8's exact ISO to provide a graphical, keyboard-accessible offline VM
installer with real local account setup, repair and preserving reinstall. Earlier
live technology previews need separately scoped review records and cannot complete
M8. This product decision does not authorize publication or waive the handling hold.
The later [desktop feature matrix](docs/05-delivery/desktop-feature-matrix.md)
defines B56/M14 usable VM delivery and B58/M15 named-PC qualification. Their
broader workflows have separate acceptance; an M8 result is not proof of M14 or
physical support. Optional browser/EXE/GPU/SMP/Wasm/AI work is not a hidden release
prerequisite or a claimed shipped capability.

- [ ] Chain of title and contributor permissions reviewed; first-party rights and
  any customer grants approved by the actual rights holders.
- [ ] U.S.-only infrastructure and all handling paths verified; export jurisdiction,
  classification, and access eligibility reviewed; no foreign-transfer exception
  assumed and no use of unapproved GitHub/AI/hosting channels.
- [ ] Actual artifact contents inventoried; complete upstream license texts,
  copyright/attribution notices, selected license alternatives, required source
  delivery/offers, and an artifact-specific SPDX/CycloneDX SBOM reviewed.
- [ ] Build inputs locked; clean source identity, compiler/tool identities, firmware,
  loader, kernel and ISO hashes, exact VM profile, and provenance retained.
- [ ] Appropriate milestone tests run against the exact packaged artifact, including
  cold optical boots, negative cases, cleanup, and supported recovery behavior.
- [ ] For M8, B41–B44 pass on the exact release ISO: graphical keyboard/pointer
  installation, target confirmation, ISO-detached/fresh-firmware installed boot,
  first-user setup/login, persistent files/ownership, boot/system repair, preserving
  reinstall and interruption/rescue outcomes. Unsupported formats are refused;
  unrelated disks remain unchanged. Account/session/RBAC and credential recovery
  are reviewed, with no default password or incomplete-setup bypass.
- [ ] For M14, B56's full desktop/app/update/backup/admin/network/audio/assistive
  matrix passes on the exact ISO and installed disk, including keyboard, faults
  and resource accounting. For M15, B57/B58 add actual named-profile qualification
  and installed/backup/rescue evidence. A VM result cannot substitute for hardware.
  Publication review applies to each exact release artifact and audience.
- [ ] Release reproducibility assessed against the applicable release gate; unexplained differences block
  any reproducibility claim. A Phase 0 probe comparison is not an ISO comparison.
- [ ] Development authority and emulator test devices explicitly reviewed for the
  selected release type; no exposed host command channel, secret, share, or
  physical-disk default. Secure Boot is not advertised as supported.
- [ ] Known issues, supported hardware/profile, update limits, persistence mode,
  and security-support policy written for that release.
- [ ] Private security contact tested; access/MFA/recovery, incident handling,
  artifact permissions, and repository protections reviewed.
- [ ] Privacy/export/branding/commercial requirements assessed for the actual
  audience and product. Required legal/entity/contact details verified.
- [ ] Checksums and release metadata prepared. Any signing claim identifies the
  signing identity, custody, verification instructions, and revocation/recovery
  process. A checksum alone is not a signature.
- [ ] Published files re-downloaded and verified against approved hashes; rollback
  or withdrawal instructions and responsible administrator recorded.

## Current blockers

M1's declared experimental VM baseline passes local acceptance; it is not a usable
desktop or release. A supported guest desktop, isolation, persistence, live state
transfer, the required installer/account lifecycle, production security assurance,
and release reproducibility remain open. Ownership/title review, a durable confidential contact, complete
runtime attribution, and artifact-level SBOM/distribution clearance are also open.
Adding these policies does not satisfy those gates. See
[milestones](docs/05-delivery/milestones.md), [legal review](LEGAL.md), and
[repository controls](docs/05-delivery/repository-controls.md).
