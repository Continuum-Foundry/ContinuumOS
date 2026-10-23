# Accessibility, input and internationalization

Status: proposed first-class requirements. R08, R15.

## Semantic infrastructure

Every interactive widget exposes a stable semantic ID, role, name, value, state,
actions, bounds, relationships and focus. The accessibility tree is distinct from
the drawing tree: decorative nodes may disappear, but meaningful controls must
remain discoverable. A virtualized list exposes logical count and navigation without
materializing every row visually.

AccessKit is a useful toolkit reference, but existing platform adapters do not
provide a screen reader for our OS. We need a native semantic transport and an
assistive consumer, plus speech/audio integration later.
[S51](../01-research/sources.md#s51)

## Interaction requirements

All first-party workflows are operable with a keyboard, including launcher, window
switching, dialogs, settings, file selection and editor controls. Focus is visible,
predictable, restored after modal dismissal, and preserved by stable logical IDs
through compatible updates. Avoid trapping focus in a destroyed/reloaded control.

Support logical UI scaling, text-size settings, high contrast, reduced motion,
non-color status cues and adequately sized hit targets. Follow established keyboard
and focus conventions. WCAG/APG are useful references by analogy; this is not a
claim of web conformance or complete native accessibility certification.
[S49](../01-research/sources.md#s49), [S50](../01-research/sources.md#s50)

## Text pipeline

P2-04's [bounded hosted text contract](../06-contracts/hosted-text.md) now records
the actual local experiment, six locked fonts, exact Unicode/version limits,
independent shaping references and common candidate obligations. It is not a
universal script/IME implementation or guest text runtime; the general pipeline
below remains the product requirement.

UTF-8 input/storage → grapheme/word boundaries → bidi/script runs → font fallback
→ shaping → line breaking/layout → glyph rasterization/cache → paint. Selection,
caret movement and hit testing must map between logical text positions and visual
glyph positions. A byte offset is not a user-perceived character index.

Unicode segmentation, bidi and line-breaking algorithms are separate pieces;
shaping is not a substitute for them. Pin compatible Unicode data versions and use
their official fixtures where available. Cache shaping/layout by text, font, size,
direction, script/features and scale, rather than text alone.
[S45](../01-research/sources.md#s45), [S46](../01-research/sources.md#s46),
[S47](../01-research/sources.md#s47), [S48](../01-research/sources.md#s48)

Initial font inventory needs licensing, fallback rules and coverage tests. Font
parsers handle untrusted structured data; bound table sizes, outlines, recursion,
glyph counts and caches. Distance fields can help scalable decoration or particular
glyph cases, but do not automatically solve small-text hinting and readability.
Evaluate ordinary coverage rasterization first.

## Input model

P2-10's selected hosted navigation keeps line/stop affinity for dual bidi caret
positions and filters shaping-cluster stops through document-wide grapheme
boundaries. The hosted semantic consumer has hierarchy, values/actions, state,
virtual row positions/counts and diffs. These internal snapshots and modeled
composition events are not native assistive speech, Windows UI Automation or a
guest IME. See [the exact selected-core policies](../05-delivery/phase-2/selected-ui-runbook.md).

Physical keys identify hardware usages. Keymaps/modifiers generate logical actions;
text input is a distinct event path. IME composition is transient state with commit/
cancel rules, not a sequence of artificial key presses. Keep a stable input service
so a shell reload cannot leave modifiers or pointer capture stuck.

Drag/drop and clipboard use typed offers with explicit grants and lifetimes. A
clipboard reference must not give arbitrary access to the source app's memory or
files. M4 establishes trusted setup/login input prerequisites: protected focus,
restricted capture/clipboard access and sensitive-event exclusion from ordinary
diagnostics, tested with synthetic input. M7 validates the actual secure path before
accepting credentials. M8 requires graphical installer, first-user setup and login
flows to work by keyboard with visible focus and accessible semantic controls.

## Localization

M9/B47 implements persistent locale, time-zone and keyboard settings. M13/B31/B55
expands text and native assistive speech on declared language/device profiles;
M14 requires their combined keyboard/semantic/audio acceptance. M8's initial
semantics and text corpus do not imply this later native consumer already exists.

Use message IDs with parameterized formatting. Do not build sentences by concatenating
translated fragments. Layout tolerates long strings and right-to-left direction.
Locale affects formatting and sorting but does not silently rewrite file identities.
Time zones and wall time belong to a service; monotonic timeout logic stays separate.

## Acceptance corpus

Test combining accents, emoji sequences, surrogate-independent Unicode handling,
Arabic/Hebrew mixed with numbers, Indic shaping, CJK wrapping, ligatures, fallback
fonts, long paths, high scale and keyboard-only tasks. Include reload during IME
composition and with an assistive focus target active. If initial script coverage
is limited, document the exact limitation rather than saying “full Unicode support.”
