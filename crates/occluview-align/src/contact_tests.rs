//! Tests for the occlusal contact scale, split out of `contact.rs` to hold the
//! workspace's file budget.

use super::{ContactScale, CLINICAL, LOAD_MAX_MM, LOAD_MIN_MM, TIGHTNESS};

/// How far the sampled sweep may dip between two stops before it counts as the
/// ramp going the wrong way. Oklab takes the shortest perceptual path between
/// stops, not the shortest path in this file's crude scalar.
const WOBBLE: i32 = 12;

fn tightness() -> ContactScale {
    ContactScale::new(&TIGHTNESS, TIGHTNESS.load_mm)
}

fn clinical() -> ContactScale {
    ContactScale::new(&CLINICAL, CLINICAL.load_mm)
}

/// How red a colour reads against whichever other channel is loudest.
///
/// Not red-minus-blue: that calls orange warmer than red, because orange has
/// almost no blue in it. Measuring red against the strongest rival channel
/// orders the whole blue-cyan-green-yellow-orange-red path the way an eye does.
fn warmth(color: [u8; 4]) -> i32 {
    i32::from(color[0]) - i32::from(color[1]).max(i32::from(color[2]))
}

/// Rough perceptual brightness, for the deep end where the ramp darkens rather
/// than reddening further.
fn luminance(color: [u8; 4]) -> i32 {
    (2 * i32::from(color[0]) + 5 * i32::from(color[1]) + i32::from(color[2])) / 8
}

#[test]
fn articulating_paper_leaves_the_rest_of_the_tooth_bare() {
    // The lesson the reference paid for twice: paint the whole approach band
    // and a case with a handful of real contacts reads as a field of colour
    // with the marks lost inside it. Anything that is not a contact is bare.
    let scale = tightness();

    assert_eq!(scale.color_at(0.05)[3], 0, "a 50 um gap is not a contact");
    assert_eq!(scale.color_at(0.2)[3], 0, "a 200 um gap is not a contact");
    assert_eq!(scale.color_at(1.0)[3], 0, "a 1 mm gap is certainly not");
    assert_eq!(
        scale.color_at(f64::NAN)[3],
        0,
        "a vertex that found no opposing surface is not painted, or guessed at"
    );
    assert_eq!(scale.color_at(0.0)[3], 255, "the touch line is a contact");
    assert_eq!(
        scale.color_at(-0.1)[3],
        255,
        "and so is 100 um into the bite"
    );
}

#[test]
fn the_paint_leaves_by_opacity_and_never_by_turning_pale() {
    // A ramp that fades toward white reads as a lighting artefact rather than
    // as data. Across the whole tolerance the colour has to hold still while
    // only the alpha moves.
    let scale = tightness();
    let touch = scale.color_at(0.0);

    let mut previous = 255_u8;
    for step in 0..=10 {
        let gap = f64::from(step) * (TIGHTNESS.paint_far_mm / 10.0);
        let faded = scale.color_at(gap);
        if faded[3] == 0 {
            continue;
        }
        assert_eq!(
            [faded[0], faded[1], faded[2]],
            [touch[0], touch[1], touch[2]],
            "the tolerance drifted in hue at {gap} mm instead of only in alpha"
        );
        assert!(
            faded[3] <= previous,
            "the feather is not monotonic at {gap} mm"
        );
        previous = faded[3];
    }
    assert!(previous < 255, "the feather never actually faded");
}

#[test]
fn red_sits_on_the_load_side_and_nowhere_earlier() {
    // Red at the far end put a ring around every mark, because a tooth curves
    // away from a contact within half a millimetre and the geometry then
    // guarantees the ring. So warmth has to rise the whole way INTO the bite,
    // from the lightest contact there is to where the law says red starts.
    let scale = tightness();

    assert!(
        warmth(scale.color_at(0.0)) < 0,
        "the lightest contact there is must read cool, not warm"
    );
    assert!(
        warmth(scale.color_at(-TIGHTNESS.load_mm)) > 0,
        "the load depth must be where red has arrived"
    );

    // Strictly at the named stops, which is where the law makes its claim.
    let mut previous = i32::MIN;
    for &(mm, _) in TIGHTNESS.stops {
        if mm > 0.0 || mm < -TIGHTNESS.load_mm {
            continue;
        }
        let here = warmth(scale.color_at(mm));
        assert!(
            here > previous,
            "warmth fell between stops at {mm} mm: {here} after {previous}"
        );
        previous = here;
    }

    // And between them, within the wobble a perceptual interpolation is
    // entitled to. Oklab takes the shortest perceptual path between two stops,
    // not the shortest path in this test's crude scalar, so a few units either
    // way is the mixing being right rather than the ramp being wrong.
    // Seeded from the first sample, not from `i32::MIN`: subtracting the
    // tolerance from that underflows.
    let mut previous = warmth(scale.color_at(0.0));
    for step in 0..=44 {
        let depth = -f64::from(step) * (TIGHTNESS.load_mm / 44.0);
        let here = warmth(scale.color_at(depth));
        assert!(
            here >= previous - WOBBLE,
            "warmth fell going deeper at {depth} mm: {here} after {previous}"
        );
        previous = previous.max(here);
    }
}

#[test]
fn past_full_load_the_ramp_darkens_rather_than_reddening_further() {
    // Red has nowhere warmer to go, so the deep stops carry a gross
    // interference by getting darker. Pinned because the obvious assumption —
    // that warmth keeps climbing — is wrong, and a later edit made on that
    // assumption would flatten the useful part of the ramp.
    let scale = tightness();
    let loaded = scale.color_at(-TIGHTNESS.load_mm);
    let heavy = scale.color_at(-0.32);
    let gross = scale.color_at(-TIGHTNESS.clamp_mm);

    assert!(
        luminance(gross) < luminance(heavy) && luminance(heavy) < luminance(loaded),
        "the deep end should darken: {} then {} then {}",
        luminance(loaded),
        luminance(heavy),
        luminance(gross)
    );
    assert!(
        warmth(gross) > 0 && warmth(heavy) > 0,
        "darkening must not cost the deep end its red"
    );
}

#[test]
fn the_slider_moves_where_the_ramp_turns_red() {
    // What the one slider is FOR. The same interference, read with a tighter
    // load depth, has to move toward the alarm colour.
    let interference = -0.12;
    let strict = ContactScale::new(&TIGHTNESS, 0.08);
    let lenient = ContactScale::new(&TIGHTNESS, 0.45);

    assert!(
        warmth(strict.color_at(interference)) > warmth(lenient.color_at(interference)),
        "tightening the load depth did not redden a 120 um interference"
    );
    assert!((strict.load_mm() - 0.08).abs() < 1e-9);
    assert!((lenient.load_mm() - 0.45).abs() < 1e-9);
}

#[test]
fn the_slider_never_moves_what_counts_as_touching() {
    // The gap side is about measurement noise, not about load. If the slider
    // stretched it too, no single reading on screen could be attributed to
    // either half of the question.
    for load_mm in [LOAD_MIN_MM, 0.15, TIGHTNESS.load_mm, LOAD_MAX_MM] {
        let scale = ContactScale::new(&TIGHTNESS, load_mm);
        assert_eq!(
            scale.color_at(TIGHTNESS.paint_far_mm * 2.0)[3],
            0,
            "a gap outside the tolerance became a contact at load {load_mm}"
        );
        assert_eq!(
            scale.color_at(0.0)[3],
            255,
            "the touch line stopped being solid at load {load_mm}"
        );
    }
}

#[test]
fn the_slider_is_bounded_and_survives_nonsense() {
    assert!((ContactScale::new(&TIGHTNESS, -5.0).load_mm() - LOAD_MIN_MM).abs() < 1e-9);
    assert!((ContactScale::new(&TIGHTNESS, 900.0).load_mm() - LOAD_MAX_MM).abs() < 1e-9);
    // A non-finite setting falls back to the law rather than to a clamp end:
    // there is no reading to preserve, so the law's own answer is the honest
    // one.
    assert!((ContactScale::new(&TIGHTNESS, f64::NAN).load_mm() - TIGHTNESS.load_mm).abs() < 1e-9);
}

#[test]
fn every_named_stop_lands_on_its_own_colour() {
    // The stops are the part a clinician reads against a legend, so they must
    // arrive exactly and not as whatever the interpolation happens to give.
    for law in [&TIGHTNESS, &CLINICAL] {
        let scale = ContactScale::new(law, law.load_mm);
        for &(mm, srgb) in law.stops {
            // The outermost stop sits exactly on the far edge, where the
            // feather has already reached zero. It has a colour, but nothing
            // ever shows it, so it is checked for being invisible instead.
            if scale.paint_weight_at(mm) <= 0.0 {
                assert_eq!(scale.color_at(mm)[3], 0);
                continue;
            }
            let painted = scale.color_at(mm);
            for (channel, (got, want)) in painted.iter().zip(srgb).enumerate() {
                assert!(
                    i32::from(*got).abs_diff(i32::from(want)) <= 1,
                    "stop at {mm} mm channel {channel}: got {got}, want {want}"
                );
            }
        }
    }
}

#[test]
fn interpolation_stays_vivid_between_the_stops() {
    // The reason the mixing runs in Oklab. A naive lerp across
    // blue-cyan-green-yellow-red dips through grey between stops; a
    // perceptual one does not. Midpoints must be at least as saturated as the
    // duller of the two stops they sit between.
    let scale = tightness();
    let stops = TIGHTNESS.stops;

    for pair in stops.windows(2) {
        let (high_mm, _) = pair[0];
        let (low_mm, _) = pair[1];
        if high_mm <= low_mm {
            continue;
        }
        let ends = [scale.color_at(high_mm), scale.color_at(low_mm)];
        let midpoint = scale.color_at((high_mm + low_mm) / 2.0);
        let chroma = |c: [u8; 4]| {
            let (r, g, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
            r.max(g).max(b) - r.min(g).min(b)
        };
        let floor = ends.iter().copied().map(chroma).min().unwrap_or(0);
        assert!(
            chroma(midpoint) >= floor - 12,
            "the midpoint between {high_mm} and {low_mm} mm went dull: \
             {} against a {floor} floor",
            chroma(midpoint)
        );
    }
}

#[test]
fn the_clinical_law_paints_the_approach_the_tightness_law_leaves_bare() {
    // The two laws answer different questions about the same bite, and the
    // difference has to be visible in what they paint at all.
    let gap = 0.1;
    assert_eq!(tightness().color_at(gap)[3], 0, "paper marks contacts only");
    assert_eq!(
        clinical().color_at(gap)[3],
        255,
        "the clinical law reads how close the antagonist is"
    );
    assert!(
        warmth(clinical().color_at(gap)) < 0,
        "safe proximity reads cool on the clinical law"
    );
    assert!(
        warmth(clinical().color_at(-CLINICAL.load_mm)) > 0,
        "load reads warm on the clinical law"
    );
}
