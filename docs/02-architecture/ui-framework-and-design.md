# UI framework and visual design specification

Status: proposed product direction, not a completed visual design. R04, R08, R15.
Revision 2 gates app-toolkit selection through the
[Slint/custom comparison](../01-research/ui-toolkit-comparison.md). Required behavior
below remains toolkit independent; do not implement every widget twice.

## Design principles

Prioritize clear hierarchy, precise typography, coherent spacing, immediate feedback,
predictable window behavior and excellent keyboard use. Richness comes from quality
and consistency; effects are selected by purpose and measured cost. Maintain
equivalent functionality in a reduced-effects profile.

The final visual identity remains a product choice. Build two or three reference
scenes before committing to a skin. Initial test scenes: editor with sidebar and
search, file browser with a large virtualized list, overlapping windows with a
launcher/settings panel, and a high-contrast/reduced-motion variant.

## Token system

Version color roles, typography scale, spacing scale, corner radii, elevation,
motion timing and icon sizes. Use semantic roles such as text-primary, surface,
focus-ring and destructive-action rather than embedding literal colors in widgets.
Provisional spacing uses multiples of 4 logical pixels; body text starts around
14–16 logical pixels. Values require visual review at multiple scales and fonts.

Theme reload validates the complete token set, resolves assets off the critical
frame path, and swaps an immutable theme generation at a scene boundary. Failure
leaves the previous generation visible. Readability settings can override a theme.

## Widget and layout architecture

Use stable logical widget IDs, distinct from allocation addresses. Layout produces
geometry; painting consumes it; semantics describes meaning separately. Dirty flags
distinguish measure, arrange, paint, semantics and composition changes. Avoid running
full layout for color changes or window movement. Bound layout recursion/iterations
and detect cyclic constraints.

Initial containers: row, column, overlay, constrained box, scroll viewport and
virtualized list. Initial controls: label, button, text field, checkbox/toggle,
slider, menu, dialog and tree/list item. Stable identity restores focus and selection
across a compatible reload. If a focused control is removed, move focus deliberately.

The custom candidate uses a Rust builder and data assets; Slint uses its supported
frontend and a custom OS adapter. Choose by prototype evidence before broad widget
investment. Bind domain behavior through [typed actions](actions-and-optional-ai.md)
and stable event IDs, not native function pointers in assets. Guest compiled-code
replacement remains available if interpreter-backed layout reload is impractical.

## Interaction

Pointer routing uses current compositor hit-test transforms. Keep physical key,
text input, modifiers and IME composition separate. Focus/capture have explicit
lifetime and release rules. Do not lose pointer-up/key-up during an update: finish
the gesture, safely cancel it, or transfer a versioned logical gesture state.

Use consistent window controls, snapping, resize affordances, focus indication,
shortcuts, drag/drop and modal behavior. Sensitive system UI cannot be spoofed by an
app gaining global focus or capture authority. Notifications have bounded lifetime,
rate limits and keyboard access.

## Motion

Motion communicates source/destination and preserves spatial context. Proposed
starting durations: 100–180 ms for small state changes, 180–280 ms for workspace/
window transitions. These are design hypotheses. Input feedback must not wait for
the whole animation, and interrupted transitions start from their current visible
state. Prefer compositor transforms over layout animation when semantics allow it.

Reduced motion disables unnecessary translation/zoom/parallax; it is not merely
slower motion. Avoid always-animated wallpaper and idle decorative loops by default.

## Acceptance

Visual snapshots at multiple sizes/scales catch clipping, baselines, wrapping and
focus rings. Interactive tests cover keyboard-only workflows, long translations,
mixed-direction text, large lists, update mid-gesture and missing assets. A host
prototype is useful, but guest input/rendering behavior must be tested separately.
See [text/accessibility](accessibility-and-internationalization.md) for required
semantics rather than treating them as finishing polish.
