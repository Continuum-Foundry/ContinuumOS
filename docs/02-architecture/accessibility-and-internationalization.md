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

Physical keys identify hardware usages. Keymaps/modifiers generate logical actions;
text input is a distinct event path. IME composition is transient state with commit/
cancel rules, not a sequence of artificial key presses. Keep a stable input service
so a shell reload cannot leave modifiers or pointer capture stuck.

Drag/drop and clipboard use typed offers with explicit grants and lifetimes. A
clipboard reference must not give arbitrary access to the source app's memory or
files. Native password/secure input and screen-capture permission are later security
gates before real authentication is introduced.

## Localization

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
