---
name: Poster Art Direction
description: This skill should be used when the user asks to "create a poster", "make a social visual", "design an event announcement", "use an inspiration image", or "generate a campaign visual". It directs an AI through research, editorial hierarchy, reference analysis, and poster-quality gates before generating production output.
version: 1.0.0
---

# Poster Art Direction

Create communication design, not decorative text on a coloured background. Treat every output as an editorial poster with a message, a reading order, a visual rhythm, and a reason to exist.

## Required workflow

1. Extract audience-facing facts from the brief. Preserve names, dates, venues, URLs, deadlines and calls to action exactly. Never turn a brand token, hex value, font name, palette label or design note into public copy.
2. Inspect every supplied reference before proposing a direction. Identify its grid, density, text-to-image ratio, focal point, cropping, contrast, typography scale, repeated motifs, visual tempo and use of negative space.
3. Separate responsibilities. Let the brand kit contribute its colours, type and voice. Let the reference contribute compositional logic and visual energy. Let the brief decide the information hierarchy.
4. Decide the reading order before writing copy: primary message, qualifier, practical information, action. Keep one dominant idea. Do not give equal visual weight to every line.
5. Select an art direction that suits the content: event posters need date and action legibility; campaign posters need a recognizable key image or assertive type system; editorial posters need a deliberate grid; image-led references need meaningful image placement rather than a generic shape substitute.
6. Create at least two visual tensions: scale contrast, cropped versus open space, quiet field versus dense information, or a strong colour interruption. Do not use a single circle or an arbitrary rectangle as a substitute for composition.
7. Run the quality gate in `references/poster-quality.md`. Reject the output when it reads as a template, exposes implementation metadata, or fails at thumbnail size.

## Reference precedence

When references exist, preserve their structural idea rather than their literal pixels. Match the arrangement, hierarchy, density and mood while applying the supplied brand system. Do not copy logos, trademarked artwork or protected text from a reference.

When no reference exists, choose a clear editorial system from the content instead of defaulting to centered text. Use an asymmetric grid for information-heavy events, a type-led composition for a short campaign line, and an image-led split only when a meaningful image is available.

## Output contract

Return public copy separately from internal reasoning. Include only audience-facing content in title, kicker, subline, details, CTA and footer. Keep technical reasoning in the internal plan. Never expose colours, font names, palette roles, prompt text or design-analysis terms in a poster.

## Additional resource

Read `references/poster-quality.md` before finalizing any poster direction.
