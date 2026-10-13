# Web interface design

The page is a single datasheet-like column, at most 640px wide, with near-black ink (`#141414`) on off-white paper
(`#f7f6f2`), inverted in dark mode. Structure comes from 1px hairline rules, a 2px radius and an 8px grid. There are no
shadows, gradients, icons or illustrations. One ink-blue accent (`#1d4e9e`, or `#8cb2ff` in dark mode) appears only in
the focus ring and the primary result, so the eye goes straight to the answer. Text uses the system UI font and every
number and unit uses the system monospace font with tabular figures, on a 1.25 modular scale with the result at the top
of the scale. Errors and notes sit directly under the field they concern, in ink rather than colour, and context fields
appear in place only when Rust asks for them, so the form never shows inputs that do not matter yet. Interactions are
instant: colour and opacity changes take at most 100ms, and none at all under `prefers-reduced-motion`.

## Measured contrast (WCAG 2.2)

| Pair | Light | Dark | Requirement |
| --- | --- | --- | --- |
| Text on background | 17.0 | 15.4 | 4.5 (AA) |
| Muted text on background | 7.0 | 7.6 | 4.5 (AA) |
| Accent (result, focus) on background | 7.4 | 8.7 | 4.5 (AA) |
| Input border on background | 3.3 | 4.0 | 3.0 (non-text) |

The tokens live in `web/src/styles/tokens.css`. The automated axe scan in `web/tests/e2e` checks both colour schemes.
