//! Painting an occlusal contact map, kept out of `align_worker` to hold the
//! workspace's 800-line file budget.
//!
//! The measurement is the deviation map exactly as it already runs. Only the
//! paint differs, which is the whole reason a slider drag is instant: the
//! worker's cache is keyed on what the DISTANCES depend on, and the contact
//! scale is not one of those things.

use occluview_align::{ContactScale, DeviationMap, Validity};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// What an unpainted vertex reads as: the plain scan material, opaque.
///
/// Not the deviation map's grey. Grey there means "measured nothing, and you
/// should know" — on a contact map the same vertex means "no contact here",
/// which is most of the arch and is not a diagnostic. Painting it grey would
/// bury the marks in exactly the flat field the scale is designed to avoid.
/// This is `occluview_core::DEFAULT_UNTEXTURED_MESH_TINT` in the 0..255 form
/// the colour channel carries, pinned by
/// `bare_surface_is_the_plain_scan_material` below so the two cannot drift.
const BARE_SURFACE: [u8; 4] = [209, 173, 107, 255];

/// One RGBA per map entry: the contact scale composited over bare surface.
///
/// The composite happens here rather than in the shader because the layer's
/// colour channel is the only thing the map travels through, and it carries
/// no second surface to blend against downstream. The cost is that a triangle
/// spanning the edge of a mark has its blend interpolated across it — the
/// same property the deviation map already has, and invisible at the size
/// dental triangles come in next to a 10 um tolerance.
pub(crate) fn contact_colors(map: &DeviationMap, scale: ContactScale) -> Vec<[u8; 4]> {
    map.signed_mm
        .par_iter()
        .zip(map.validity.par_iter())
        .map(|(value, state)| {
            if *state != Validity::Measured {
                return BARE_SURFACE;
            }
            over_bare_surface(scale.color_at(f64::from(*value)))
        })
        .collect()
}

/// Blend one painted colour onto bare surface by its own alpha.
fn over_bare_surface(painted: [u8; 4]) -> [u8; 4] {
    match painted[3] {
        0 => BARE_SURFACE,
        255 => [painted[0], painted[1], painted[2], 255],
        weight => {
            let mix = |mark: u8, bare: u8| {
                let mark = u32::from(mark) * u32::from(weight);
                let bare = u32::from(bare) * (255 - u32::from(weight));
                #[allow(clippy::cast_possible_truncation)]
                {
                    ((mark + bare) / 255) as u8
                }
            };
            [
                mix(painted[0], BARE_SURFACE[0]),
                mix(painted[1], BARE_SURFACE[1]),
                mix(painted[2], BARE_SURFACE[2]),
                255,
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{contact_colors, over_bare_surface, BARE_SURFACE};
    use occluview_align::{ContactScale, DeviationMap, Validity, TIGHTNESS};

    fn map_of(values: &[f32], states: &[Validity]) -> DeviationMap {
        DeviationMap {
            signed_mm: values.to_vec(),
            validity: states.to_vec(),
        }
    }

    #[test]
    fn a_vertex_with_no_opposing_surface_reads_as_bare_scan_not_as_a_finding() {
        // The deviation map's grey means "measured nothing, and you should
        // know". On a contact map that is most of the arch, and it is not a
        // diagnostic — it is the absence of a contact.
        let map = map_of(&[0.0, 0.0], &[Validity::OutOfReach, Validity::Measured]);

        let colors = contact_colors(&map, ContactScale::new(&TIGHTNESS, TIGHTNESS.load_mm));

        assert_eq!(colors[0], BARE_SURFACE);
        assert_ne!(colors[1], BARE_SURFACE, "the touch line is a contact");
    }

    #[test]
    fn a_gap_wider_than_the_tolerance_leaves_no_mark() {
        let map = map_of(&[0.4], &[Validity::Measured]);

        let colors = contact_colors(&map, ContactScale::new(&TIGHTNESS, TIGHTNESS.load_mm));

        assert_eq!(colors[0], BARE_SURFACE);
    }

    #[test]
    fn the_feather_lands_between_the_mark_and_the_surface() {
        // Half opacity has to sit between the two, on every channel. A blend
        // that overshot either end would put a rim around every mark, which is
        // the artefact the feather exists to remove.
        let mark = [200_u8, 40, 40, 128];

        let blended = over_bare_surface(mark);

        for channel in 0..3 {
            let (low, high) = (
                mark[channel].min(BARE_SURFACE[channel]),
                mark[channel].max(BARE_SURFACE[channel]),
            );
            assert!(
                (low..=high).contains(&blended[channel]),
                "channel {channel} left the pair: {} outside {low}..={high}",
                blended[channel]
            );
        }
        assert_eq!(blended[3], 255, "the layer's colour channel is opaque");
    }

    #[test]
    fn bare_surface_is_the_plain_scan_material() {
        // Written as bytes because that is the form the colour channel takes;
        // pinned here so it cannot drift from the material the renderer shows
        // an unpainted scan in.
        let [red, green, blue, _] = occluview_core::DEFAULT_UNTEXTURED_MESH_TINT;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let as_bytes = [
            (red * 255.0).round() as u8,
            (green * 255.0).round() as u8,
            (blue * 255.0).round() as u8,
        ];
        assert_eq!(
            [BARE_SURFACE[0], BARE_SURFACE[1], BARE_SURFACE[2]],
            as_bytes
        );
    }

    #[test]
    fn a_fully_opaque_mark_survives_the_composite_untouched() {
        assert_eq!(over_bare_surface([12, 34, 56, 255]), [12, 34, 56, 255]);
        assert_eq!(over_bare_surface([12, 34, 56, 0]), BARE_SURFACE);
    }
}
