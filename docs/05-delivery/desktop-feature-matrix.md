# Desktop feature and release matrix

Status: approved roadmap scope, 2026-10-21; future acceptance, not shipped features.
See [milestones](milestones.md), [backlog](backlog.md) and [validation](validation.md).
Every owner is a responsibility, not a person assignment. Bxx owners carry the
work; P-level stories are created only on the owner's milestone decomposition request.

M8 remains the installable experimental VM release. M14 adds usable-desktop
workflows; M15 qualifies one named PC after the VM release. Optional research is
not a release dependency. For deferred rows, milestone/task references identify
the decision owner, not a promise to deliver that feature in that milestone.
All rows below describe planned acceptance; existing reviews alone establish
current implementation. The baseline cold-idle target remains 256 MiB.

## Coverage and acceptance

The column structure is checked by the documentation audit. Classifications are
`M8 required`, `M14 required`, `named-PC profile`, `optional research`, or `deferred`.

| ID | Feature | Owner | Milestones | Backlog | Classification | Acceptance outcome |
| --- | --- | --- | --- | --- | --- | --- |
| F01 | Original kernel and optical ISO | Platform/Build | M1,M8 | B03,B04,B38 | M8 required | Exact ISO cold-boots offline through the declared firmware/optical route without host shares. |
| F02 | Process isolation and scoped authority | Kernel/Security | M3,M7,M8 | B11,B12,B13,B41 | M8 required | Guest tests deny forged/cross-owner authority and ordinary apps cannot administer accounts or disks. |
| F03 | Installer, accounts, repair and preserving reinstall | Installation/Security | M6,M7,M8 | B41,B42,B43,B44 | M8 required | Graphical keyboard/pointer install, ISO-detached boot, owner setup/login, preserving repair/reinstall and interrupted recovery pass. |
| F04 | Supported live resource/app/service replacement | Updates | M2,M4,M5,M6,M8 | B10,B17,B21,B25 | M8 required | Declared changes preserve accepted state and boot identity with one writer and honest unsupported/reboot outcomes. |
| F05 | Durable files and boot recovery | Storage/Recovery | M6,M8 | B22,B23,B24,B26,B40 | M8 required | Acknowledged writes survive tested faults and compatible known-good/rescue recovery preserves current data. |
| F06 | Initial text, keyboard and semantics | UI/Accessibility | M2,M4,M8 | B08,B09,B16 | M8 required | Declared text/scale/keyboard/semantic corpus passes in the guest; native assistive delivery is not inferred. |
| F07 | Baseline efficiency and diagnostics | Performance | M3,M6,M8 | B13,B32 | M8 required | Named workloads account for guest memory/idle/frame/reload cost and bounded useful diagnostics against the 256 MiB target. |
| F08 | File operations, trash and pickers | Files/UI | M9,M14 | B45 | M14 required | Create/copy/move/rename/delete/trash and open/save work with grants, conflicts, progress, cancellation and interruption tests. |
| F09 | Clipboard and drag/drop | Desktop/Security | M9,M14 | B46 | M14 required | Cross-app typed offers transfer only authorized data and reject stale, revoked or excessive requests. |
| F10 | File associations and default apps | Desktop | M9,M14 | B46 | M14 required | Explicit user defaults launch authorized installed apps; missing/uninstalled apps produce recoverable choices. |
| F11 | Notifications | Desktop/Accessibility | M9,M14 | B46 | M14 required | Bounded notifications/actions support keyboard focus, dismissal, expiry and permission/rate-limit tests. |
| F12 | Time, time zone, locale and keyboard | Settings/Platform | M9,M14 | B47 | M14 required | Settings persist across reboot; wall-time changes preserve monotonic deadlines and stored file identities. |
| F13 | Native SDK and app lifecycle | SDK/Apps | M10,M14 | B48 | M14 required | An independently built native app installs, launches, updates and uninstalls against the supported SDK. |
| F14 | App permissions and uninstall data retention | Apps/Security | M10,M14 | B48 | M14 required | Permission increases need policy review and uninstall retains or removes only explicitly selected app data. |
| F15 | Verified offline software updates | Updates/Security | M10,M14 | B49 | M14 required | Verified app/system bundles activate recoverably, reject bad authority and report reboot-required changes separately from dev reload. |
| F16 | Documents/settings backup and restore | Data/Recovery | M11,M14 | B50 | M14 required | Verified export restores to a fresh installation with explicit ownership mapping after original guest-disk loss. |
| F17 | Task/service management and diagnostics | Administration | M11,M14 | B51 | M14 required | Authorized users inspect CPU/memory/disk and stop hung apps; unsafe service controls and unbounded exports are denied. |
| F18 | Shutdown and session recovery | Lifecycle | M11,M14 | B52 | M14 required | Bounded save/flush shutdown reports failures and restores only compatible authorized sessions without crash loops. |
| F19 | Ethernet, addressing, DHCP/DNS, TCP/TLS and trust | Networking/Security | M12,M14 | B28,B53 | M14 required | Controlled network, malformed traffic, time/entropy and certificate tests pass without verification bypass. |
| F20 | Connections and online update client | Networking/Updates | M12,M14 | B54 | M14 required | Connection settings and interrupted downloads use the same verifier/activation path and preserve offline use. |
| F21 | Playback, capture, mixing and permissions | Audio | M13,M14 | B55 | M14 required | Supported VM devices/formats work within budgets with capture indicators, denial/revocation and failure cleanup. |
| F22 | Richer text and native assistive speech | Accessibility | M13,M14 | B31 | M14 required | Semantic focus/value/actions produce validated speech output and keyboard behavior for the declared language/device matrix. |
| F23 | Coherent usable VM desktop delivery | Release | M14 | B56 | M14 required | Exact packaged ISO and installed disk pass combined workflows, faults, resource budgets and provenance review. |
| F24 | One named UEFI PC and required drivers | Platform/Release | M15 | B57,B58 | named-PC profile | Actual selected hardware passes boot/input/storage/display/network/audio/power and installed/backup/rescue acceptance. |
| F25 | x64 Windows EXE study and probes | Compatibility | M16 | B59 | optional research | Reviewed native-service/Wine comparison and bounded guest probes yield a supported-subset proposal or evidenced stop/replan. |
| F26 | Native versus Windows-route browser study | Browser/Security | M17 | B60 | optional research | Dependency/render/network/sandbox/update/memory findings support a separate decision without claiming browser delivery. |
| F27 | GPU acceleration | Graphics | M18 | B29 | optional research | Exact backend shows measured benefit, fault/resource correctness and a maintained software fallback. |
| F28 | SMP | Kernel | M19 | B30 | optional research | Multicore context/TLB/ordering/update tests and optimized workload comparisons pass on the named profile. |
| F29 | Bounded Wasm | Runtime | M20 | B27 | optional research | Exact interpreter/import path enforces fuel/memory/stack/cancel limits with native cost comparison. |
| F30 | Optional local AI | Runtime/Actions | M21 | B39 | optional research | Named model/backend uses scoped typed actions, bounded memory and safe worker crash/unload without breaking desktop/reload. |
| F31 | General Windows/managed/driver compatibility | Compatibility | M16 | B59 | deferred | Initial x64 user-app probes confer no Windows-driver, WOW64, managed-runtime, DRM or broad application support. |
| F32 | Broader PCs, wireless, Bluetooth, printing and laptop sleep | Platform | M15 | B57,B58 | deferred | Named-PC qualification records exact devices and explicitly excludes untested profiles/device families or power states. |
| F33 | Encryption and Secure Boot | Security/Platform | M7,M15 | B41,B57,B58 | deferred | Require separate key/trust/recovery and actual platform evidence; account login and installation do not establish these features. |
| F34 | Complete-machine backup | Data/Recovery | M11 | B50 | deferred | The first backup contract excludes system generations, credentials, secrets and privileged policy from its guarantee. |
| F35 | Mixed-DPI displays, HDR and arbitrary GPUs | Graphics | M18 | B29 | deferred | Acceleration on one backend does not assert multi-display/color/hardware coverage without separate tests. |
| F36 | IPv6 and broader networking | Networking | M12 | B53 | deferred | Publish exact protocol coverage; baseline IPv4/TCP/TLS acceptance cannot imply untested IPv6 or other protocol support. |

## Release dependency boundaries

B35 accepts M8 and requires B41–B44. B56 accepts M14 and requires the M8 root,
B45–B55 and B31. B58 requires B56 through B57 plus installed lifecycle work.
B27/B29/B30/B39/B59/B60 remain outside B35/B56. Milestone numbers do not impose a
serial dependency on optional tracks. No row authorizes physical writes or publication.
