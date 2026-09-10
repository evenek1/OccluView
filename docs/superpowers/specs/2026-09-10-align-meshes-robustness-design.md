# Align Meshes: robustness and operator UI design

## Status

This is the design handoff for the Align Meshes repair. It is deliberately
separate from implementation: the alignment kernel, application state machine,
egui panels, localization, and measured-map shader all participate in the
behavior, so a cosmetic-only patch would leave the causal bug in place.

The current branch was clean at discovery time. The baseline command
`cargo test -p occluview-align -p occluview-app` completed successfully (100
alignment tests and 845 application tests; 8 application tests ignored). No
real-scan fixture was configured in this checkout, so the operator's actual
clinical case remains a later acceptance gate.

## Operator contract

The tool will have one unambiguous lifecycle:

1. Point pairs and `Perform alignment` provide a coarse pose only.
2. `Best fit matching` is the only operation that establishes a refined match.
3. A heatmap may be measured or shown only for a current, successfully landed
   refined match.
4. Returning from `Manually` to `Automatically` only restores the panel. It
   never starts a measurement by itself.
5. Any pose, scan-role, geometry, or exclusion-mask change invalidates the
   refined-match authority and removes the map. The operator must run Best fit
   again.
6. Hiding and re-showing a map without changing the match is allowed; it may
   reuse the cached map or recolour it, but it cannot create a map for an
   unrefined pose.

The distinction is intentional: naming two scans is enough to compare files in
the low-level measurement API, but it is not proof that this Align Meshes
session has completed its refinement stage.

## Verified current causes

### Heatmap appears after Manual

`AlignTool::can_measure()` returns true as soon as both layer roles are named.
The arm-time provisional pair and `clear_points()` preserve those roles. In
`settle_align_tab_change()`, entering `Automatically` calls
`measure_if_shown()`. The default `show_deviation` is true, so the named-role
condition submits a Measure job even though no `Refined` outcome has landed.

The fix is an explicit session fact, not a change to the meaning of
`can_measure()`: add a refined-match gate to `AlignState`, clear it centrally
when a fit becomes stale, and set it only after a `Refined` result has been
successfully committed. Both automatic remeasurement and direct UI actions
must check the same gate. The tab-settling path will not call measurement when
returning to Automatic.

`Refined` has to mark the gate after `commit_align_pose()`, because committing
through `set_scene()` currently invalidates derived alignment state. A failed or
stale completion must never mark it. Mask edits already invalidate the map and
will also invalidate the gate.

### Unwanted text

The main panel currently renders the percentage within tolerance, RMS, grey
vertex counts broken down by cause, saturation advice, and long refine status
sentences containing coverage, weak axes, and blind directions. The brush
window renders `X% marked out of match`. These are diagnostic data useful for
tests or a future diagnostic view, but they compete with the operator's actual
decisions in the working window.

The repair will retain internal `DeviationStats`, `Unmeasured`, and
`Observability` values for correctness and future diagnostics, but remove their
persistent presentation from the production Align Meshes and Brush windows.
Only compact, actionable transient states remain: matching in progress, map
ready, or a short failure instruction such as moving the scans closer,
increasing max influence, or running Best fit first. No percentages, RMS,
grey-count sentences, weak-axis explanations, or brush coverage percentages
appear in the working panels.

### Deviation ramp

The application already defaults to magnitude mode, but the shared kernel's
`RampSettings::default()` is signed and `ramp_color()` currently uses the
tolerance as a flat nominal band. That makes a range of nonzero distances look
identical and makes the displayed control mean something different from the
requested color scale.

The working map will use one continuous magnitude ramp:

- exactly `0.00 mm` is the cold blue stop;
- the configured display maximum defaults to `0.10 mm` and is the hot red stop;
- values above the maximum clamp to the same red stop;
- tolerance remains a statistics/acceptance concept only and cannot flatten
  measured colors;
- the default shared ramp mode becomes magnitude so library and application
  defaults cannot diverge.

The main panel exposes one compact `Deviation range` slider, bounded to the
working range and visibly tied to the legend. The default is `0.10 mm`. Signed
mode and stepped bands remain available only if a later diagnostic surface
needs them; they are not part of the operator's primary path.

The legend will show `0.00 mm`, the selected maximum, and a compact `> max`
red indication. A single static `No data` swatch may remain; dynamic reasons
and counts do not.

### Saturation and gloss

The current color stops are already reasonably saturated in CPU memory. The
measured-map shader then darkens them with a broad lighting mix and deliberately
skips specular lighting. The repair will make the map brighter and more
readable by changing only a scalar luminance factor: stronger color retention,
controlled form lighting, and a restrained scalar Fresnel/gloss term. It will
not add white to individual channels, tint the ramp, change normals, or mutate
geometry. Thus a red/blue reading remains the same reading under lighting.

The GPU tests will pin both properties: hue ratios are preserved and a curved
test surface has more useful luminance/form separation than the current map.
The existing non-map golden image remains a regression guard.

## Robust matching design

The current refiner is a deterministic, two-level local point-to-plane ICP:
one-sided nearest surface correspondences, residual trimming, damped 6x6 solve,
and no trial-step acceptance. The surface index has exact brute-force parity
tests, so the first hypothesis is not a missing spatial-index hit. The causal
risk is local registration: a rough or partially overlapping placement can
find a nearby wrong patch, slide tangentially, or accept a step that worsens
the actual objective. The final pose can be a local minimum even when a nearby
valid seating exists.

This is consistent with the primary registration literature: point-to-plane is
fast near a good initial pose but is not a general solution for far or noisy
initial placements; robust or global strategies are needed when overlap and
initial transform are poor. The product must therefore improve the bounded
local path and refuse low-confidence results rather than silently claiming a
global-registration guarantee.

### Chosen implementation direction

The existing point-fit stage and worker/cache contracts stay in place. The
refine stage will be strengthened in this order:

1. Add a deterministic failing synthetic fixture for the reported class:
   adjacent/partial surfaces with a coarse pose where the current solver moves
   sideways or accepts a worse seating. Keep the fixture curved or textured so
   a mathematically unobservable flat-plane slide is not misclassified as a
   solver bug.
2. Make the objective explicit. Correspondences carry the target point as
   well as the point-to-plane residual. Trimming and candidate evaluation use a
   robust geometric distance, while the normal equations retain the stable
   point-to-plane term.
3. Add bounded backtracking. Evaluate the current objective, try a fixed
   sequence of step scales, recompute correspondences for each trial, and accept
   only a finite improvement with sufficient correspondence coverage. If no
   trial improves the objective, keep the best accepted pose and stop instead
   of committing a sideways/worse step. Return the pose and summary from the
   same evaluated state; never report statistics for one pose while returning a
   different unevaluated pose.
4. Add overlap-consistency protection. Use reciprocal or equivalent bounded
   fixed-to-moving evidence where the fixture shows one-sided nearest patches
   are the failure. The check must respect masks, orientation, partial overlap,
   and deterministic sample order. Do not impose a full-overlap threshold that
   rejects legitimate dental scans.
5. Add a bounded influence-radius ladder inside the operator's configured max:
   start narrow to avoid unrelated surfaces, widen only when the current level
   cannot establish enough correspondences, and never exceed the visible `Max
   influence distance` control. A broad radius is a search budget, not a silent
   override.
6. Add a no-improvement/ambiguous refusal when the solver has enough points but
   cannot improve the starting objective or cannot establish stable overlap.
   The application message will be short and actionable. A failed refine does
   not commit a new pose and does not enable the heatmap.

The exact reciprocal term and threshold will be selected by the failing fixture
and existing deterministic tests, not by a cosmetic score or a single real
case. If the fixture demonstrates that local refinement still cannot recover a
coarse pose, the UI will require/encourage the documented point-pair coarse
stage rather than introduce an unbounded or nondeterministic global search.

The two-pair frame fit remains guarded for degenerate normals and collinearity.
It will not be replaced by a second alignment implementation. Its output stays
coarse and never authorizes a heatmap until Best fit has succeeded.

## Window design

The main panel keeps the current two-tab interaction model and the controls
that map to the established dental-CAD workflow:

1. title and `Automatically` / `Manually` tabs;
2. one compact moving-to-fixed role row;
3. point prompt plus `Back` and `Clear`;
4. `Perform alignment` and primary `Best fit matching` buttons;
5. `Matching parts` and `Max influence distance` sliders;
6. collapsed orientation controls;
7. `Exclude selected parts` launcher;
8. gated `Heatmap`/`Show distance` and the deviation legend/range slider;
9. `Cancel` and `Done`.

The panel will not show diagnostic paragraphs or metric cards. Long text is
not hidden behind a fold only to remain in the primary working flow; it is
removed from that flow. Tooltips retain only short explanations of a control.
Keyboard focus and enabled/disabled states remain visible, and every disabled
heatmap control has one concise next step: run Best fit matching.

The separate Brush window remains movable and keeps the useful whole-mesh
commands, brush size, inverse mode, and automatic radius. It removes the
subtitle/hint clutter that is not needed while painting and removes the dynamic
marked-percentage line entirely. Command labels and tooltips remain localized.

This follows the documented exocad Align Meshes vocabulary and sequence: point
pairs, Perform alignment, Best fit matching, matching-parts ratio, maximum
influence distance, orientation, distance display, and exclusion brush. The
existing two-window interaction model is preserved rather than replaced.

## Localization and persistence

All user-facing additions or removals go through the existing seven Fluent
catalogs (`en`, `ru`, `de`, `es`, `fr`, `it`, `pt-BR`). Dynamic numeric
formatting is limited to the compact slider/legend and uses the existing
millimetre convention. Removed metric strings are removed from call sites and
catalogs together after parity checks; no English fallback is introduced by
leaving an unlocalized literal in a new branch.

Display-only range changes recolor/reuse the cached map. Geometry, pose,
orientation, influence radius, or mask changes invalidate the relevant cache or
refined authority as appropriate. The worker generation guard remains the sole
authority for discarding stale completions.

## Test-first implementation and acceptance

Implementation will proceed with failing tests first and small causal commits
or reviewable patches:

1. Pure refined-match gate tests: role naming alone, tab return, point fit,
   successful refine, failed/stale refine, manual drag, role swap, geometry
   change, mask edit, hide/show, and stale worker completion.
2. Worker/app tests proving a Measure job cannot be submitted before refined
   authority and that a successful Refined result can trigger exactly one
   measurement when the map is enabled.
3. Ramp tests for exact zero blue, exact `0.10 mm` red, all values above red,
   continuous intermediate stops, and tolerance independence.
4. Panel/brush contract tests proving the requested controls remain and the
   rejected diagnostic phrases/coverage line are absent from production UI.
5. Fluent catalog parity and long-locale/compact-layout source checks.
6. ICP fixture tests for objective monotonicity, accepted-step consistency,
   partial overlap, mask/orientation handling, deterministic repeatability, and
   refusal without a trustworthy improvement.
7. Render tests for saturated measured-map readability, hue preservation, and no
   geometry/normal/raycast mutation.

Automated proof is not the final acceptance gate. After implementation, the
remaining gates are a GPU desktop visual pass, the user's real adjacent/coarse
scan pair, and (if this is intended for clinical use) domain/operator
acceptance. No real-case or clinical claim will be made from unit tests alone.

## Independent audit at the end

After implementation and verification, three read-only subagents will audit
independently in fresh contexts:

- ICP math, fixture quality, objective/coverage/refusal behavior;
- UI state gate, tab transitions, localization, and removal of diagnostic text;
- ramp/shader/render/cache authority and performance.

Each report must include evidence and PASS/FAIL, not general approval. Any real
finding will be fixed with a targeted test and reverified. The audit is bounded
to one pass plus at most one focused follow-up; agents will not write into this
worktree concurrently.

## External references

- exocad, [Align Meshes documentation](https://wiki.exocad.com/wiki/index.php/Align_Meshes)
- Mitra et al., [local registration / point-to-plane limitations](https://graphics.stanford.edu/~niloy/research/local_registration/paper_docs/local_registration_sgp_04.pdf)
- Mitra et al., [registration accuracy and convergence behavior](https://graphics.stanford.edu/~ngelfand/papers/registration/mitra-registration-04.pdf)
- Yang et al., [Go-ICP: global registration for difficult initial poses](https://openaccess.thecvf.com/content_iccv_2013/html/Yang_Go-ICP_Solving_3D_2013_ICCV_paper.html)
- Vercel Interface Guidelines, [interaction and feedback rules](https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md)
