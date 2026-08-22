//! The occlusal contact scale: how hard two surfaces meet, as a colour.
//!
//! Reads the same signed per-vertex field [`crate::deviation`] produces —
//! positive is a gap to the opposing surface, zero is exact touch, negative is
//! penetration depth. What makes it a clinical reading rather than a metrology
//! one is where the alarm sits, how much of the surface is painted at all, and
//! that the far edge leaves by opacity instead of by turning pale.
//!
//! TWO LAWS, ONE EVALUATOR. A law is data: band edges and an ordered stop
//! table. The interpolation, the feather and the clamp are written once, so the
//! two laws differ in numbers and never in behaviour.
//!
//! - [`TIGHTNESS`] is digital articulating paper. It marks only where the
//!   arches actually meet and colours that by how deep the bite is there.
//! - [`CLINICAL`] is the T-Scan convention: cool blues for safe proximity,
//!   warm reds only for load, painting the approach as well as the contact.
//!
//! Three things decide whether the map is readable, and all three were learned
//! the expensive way.
//!
//! **Where red sits.** Put red at the far end and every mark wears a red ring,
//! because a tooth curves away from a contact within half a millimetre and the
//! geometry then guarantees the ring. Red belongs on the load side.
//!
//! **How much is painted.** Paint the whole approach band and a case with a
//! handful of real contacts reads as a field of blue with the marks lost inside
//! it. Paper marks where it is squeezed and leaves the rest of the tooth bare.
//!
//! **How the paint ends.** By alpha, never by fading toward white: a ramp that
//! washes out reads as a lighting artefact rather than as data.
//!
//! Interpolation runs in Oklab. The perceptual straight line between two stops
//! has no neon band and no hue overshoot, which a naive sRGB lerp across
//! blue to cyan to green to yellow to red always produces.

use std::sync::OnceLock;

/// One stop: where it sits on the signed field, and its display sRGB colour.
type Stop = (f64, [u8; 3]);

/// A complete colour law. Everything a caller needs to paint and to gate is
/// here, so the law travels as one value rather than as a loose threshold.
#[derive(Debug)]
pub struct ContactLaw {
    /// Field values above this stay unpainted: bare scan surface.
    pub paint_far_mm: f64,
    /// Feather width at the far edge, where paint fades into the surface.
    pub far_fade_mm: f64,
    /// Penetration depth this law calls fully loaded — where the ramp turns
    /// red. This is the number the slider moves.
    pub load_mm: f64,
    /// Penetration past this clamps to the last stop, so one gross
    /// interference cannot flatten the useful part of the ramp.
    pub clamp_mm: f64,
    /// Stops from the far edge down to the deep clamp, descending. The
    /// evaluator relies on that order.
    pub stops: &'static [Stop],
    /// Oklab form of `stops`, built once on first use.
    oklab: OnceLock<Vec<Oklab>>,
}

/// Digital articulating paper: only where the arches meet, coloured by how hard.
///
/// Nothing is painted where the teeth do not touch. `paint_far_mm` is a
/// measurement-noise tolerance rather than a band — ten microns is below what a
/// scan pair can resolve, so a vertex a hair short of touching still belongs to
/// its mark, and any real gap stays bare surface. The tolerance carries the
/// touch colour flat, so inside it only the opacity changes and a vertex fades
/// out instead of drifting to some other colour.
///
/// The hues sit on the saturated blue-cyan-green-yellow-orange-red path rather
/// than on a chord through it: interpolating between two stops that both lie on
/// the path keeps every intermediate colour vivid, where one long chord from
/// blue to red passes through washed-out mud.
pub static TIGHTNESS: ContactLaw = ContactLaw {
    paint_far_mm: 0.01,
    far_fade_mm: 0.01,
    load_mm: 0.22,
    clamp_mm: 0.5,
    stops: &[
        (0.01, [29, 78, 216]),   // the noise tolerance, carrying the touch colour flat
        (0.0, [29, 78, 216]),    // #1d4ed8 the touch line: the lightest contact there is
        (-0.03, [21, 149, 201]), // #1595c9
        (-0.06, [18, 171, 143]), // #12ab8f
        (-0.09, [64, 192, 87]),  // #40c057 a normal working contact
        (-0.12, [183, 203, 39]), // #b7cb27 about one thickness of articulating paper
        (-0.15, [252, 196, 25]), // #fcc419 firm
        (-0.18, [251, 106, 26]), // #fb6a1a strong, on its way to red
        (-0.22, [239, 62, 54]),  // #ef3e36 RED STARTS HERE, and nowhere earlier
        (-0.32, [193, 39, 45]),  // #c1272d heavy
        (-0.5, [122, 18, 18]),   // #7a1212 gross interference
    ],
    oklab: OnceLock::new(),
};

/// The T-Scan convention: blue is safe proximity, red is load, and the approach
/// is painted as well as the contact.
///
/// The reading for "how close is the antagonist" rather than "where does it
/// touch". Both questions get asked about the same bite, so both laws exist.
pub static CLINICAL: ContactLaw = ContactLaw {
    paint_far_mm: 0.2,
    far_fade_mm: 0.06,
    load_mm: 0.12,
    clamp_mm: 0.35,
    stops: &[
        (0.2, [165, 216, 255]),   // #a5d8ff far fade edge, barely visible
        (0.12, [77, 171, 247]),   // #4dabf7 the calm zone
        (0.06, [59, 201, 219]),   // #3bc9db still calm, approaching the warm transition
        (0.025, [105, 219, 124]), // #69db7c light proximity, about to graze
        (0.0, [255, 212, 59]),    // #ffd43b exact touch
        (-0.025, [255, 146, 43]), // #ff922b initial penetration
        (-0.06, [250, 82, 82]),   // #fa5252 loaded contact
        (-0.12, [224, 49, 49]),   // #e03131 heavy load
        (-0.35, [140, 17, 17]),   // #8c1111 deep interference
    ],
    oklab: OnceLock::new(),
};

/// The narrowest load depth the slider offers, in millimetres.
///
/// Below this the whole ramp lives inside one scan's noise and the map stops
/// reporting the bite and starts reporting the scanner.
pub const LOAD_MIN_MM: f64 = 0.05;

/// The widest load depth the slider offers, in millimetres.
///
/// Past half a millimetre nothing on the ramp reads as a contact any more.
pub const LOAD_MAX_MM: f64 = 0.60;

impl ContactLaw {
    /// Oklab form of the stops, built once.
    fn oklab(&self) -> &[Oklab] {
        self.oklab.get_or_init(|| {
            self.stops
                .iter()
                .map(|(_, srgb)| linear_to_oklab(srgb_to_linear(*srgb)))
                .collect()
        })
    }

    /// How opaque the paint is at `signed_mm`: fully on across the scale, and
    /// feathered to nothing over the last `far_fade_mm` of the far band.
    ///
    /// Smoothstep rather than linear, so the feather has no visible crease
    /// where it begins. Zero means bare surface, which is the point of the law
    /// rather than an edge case of it.
    #[must_use]
    pub fn paint_weight_at(&self, signed_mm: f64) -> f64 {
        if !signed_mm.is_finite() || signed_mm > self.paint_far_mm {
            return 0.0;
        }
        if self.far_fade_mm <= 0.0 || signed_mm <= self.paint_far_mm - self.far_fade_mm {
            return 1.0;
        }
        let t = ((self.paint_far_mm - signed_mm) / self.far_fade_mm).clamp(0.0, 1.0);
        t * t * t.mul_add(-2.0, 3.0)
    }
}

/// The scale as the operator has it set: a law, and the load depth they moved
/// the slider to.
///
/// One number, because the question a bite poses is a question about a
/// threshold — where does close stop being contact and start being pressure —
/// and the honest way to answer it is to move the threshold and watch the map,
/// not to type a value. The gap side does not move with it: that side is about
/// measurement noise, not about load.
#[derive(Clone, Copy, Debug)]
pub struct ContactScale {
    law: &'static ContactLaw,
    depth: f64,
}

impl ContactScale {
    /// The scale for `law` with the ramp reaching full load at `load_mm`.
    #[must_use]
    pub fn new(law: &'static ContactLaw, load_mm: f64) -> Self {
        let load = if load_mm.is_finite() {
            load_mm.clamp(LOAD_MIN_MM, LOAD_MAX_MM)
        } else {
            law.load_mm
        };
        Self {
            law,
            depth: load / law.load_mm,
        }
    }

    /// The law this scale paints with.
    #[must_use]
    pub fn law(&self) -> &'static ContactLaw {
        self.law
    }

    /// The load depth the ramp reaches full red at, in millimetres.
    #[must_use]
    pub fn load_mm(&self) -> f64 {
        self.law.load_mm * self.depth
    }

    /// Where a stop sits once the slider has moved.
    ///
    /// Only the penetration side scales. Stretching the tolerance on the gap
    /// side with it would make the slider quietly change what counts as
    /// touching at the same time as what counts as heavy, and then no single
    /// reading on screen could be attributed to either.
    fn stop_mm(&self, index: usize) -> f64 {
        let mm = self.law.stops[index].0;
        if mm < 0.0 {
            mm * self.depth
        } else {
            mm
        }
    }

    /// The colour at a signed field value: display sRGB, with the far feather
    /// carried in alpha.
    ///
    /// A non-finite value is a vertex that found no opposing surface. It is not
    /// painted, and it is not guessed at either.
    #[must_use]
    pub fn color_at(&self, signed_mm: f64) -> [u8; 4] {
        let weight = self.paint_weight_at(signed_mm);
        if weight <= 0.0 {
            return [0, 0, 0, 0];
        }
        let linear = oklab_to_linear(self.oklab_at(signed_mm));
        let [red, green, blue] = linear_to_srgb(linear);
        [red, green, blue, to_u8(weight)]
    }

    /// How opaque the paint is at `signed_mm`. See
    /// [`ContactLaw::paint_weight_at`] — the tolerance does not scale, so this
    /// is the law's own answer.
    #[must_use]
    pub fn paint_weight_at(&self, signed_mm: f64) -> f64 {
        self.law.paint_weight_at(signed_mm)
    }

    /// Piecewise-linear in Oklab between the stops, clamped to the end stops.
    ///
    /// Evaluated exactly rather than through a lookup table. A table would have
    /// to be fine enough that every named stop lands on a sample or the stop
    /// colours drift by interpolation error, and the exact form is already
    /// cheap next to the nearest-point search that produced the field.
    fn oklab_at(&self, signed_mm: f64) -> Oklab {
        let oklab = self.law.oklab();
        let last = self.law.stops.len() - 1;
        let value = signed_mm.clamp(self.stop_mm(last), self.stop_mm(0));
        for index in 0..last {
            let high = self.stop_mm(index);
            let low = self.stop_mm(index + 1);
            if value > high || value < low {
                continue;
            }
            let span = high - low;
            let t = if span <= 0.0 {
                0.0
            } else {
                (high - value) / span
            };
            return oklab[index].mix(oklab[index + 1], t);
        }
        oklab[last]
    }
}

/// A colour in Oklab, the space the stops are interpolated through.
#[derive(Clone, Copy, Debug)]
struct Oklab {
    lightness: f64,
    green_red: f64,
    blue_yellow: f64,
}

impl Oklab {
    fn mix(self, other: Self, t: f64) -> Self {
        Self {
            lightness: self.lightness + (other.lightness - self.lightness) * t,
            green_red: self.green_red + (other.green_red - self.green_red) * t,
            blue_yellow: self.blue_yellow + (other.blue_yellow - self.blue_yellow) * t,
        }
    }
}

fn srgb_to_linear(srgb: [u8; 3]) -> [f64; 3] {
    srgb.map(|channel| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    })
}

fn linear_to_srgb(linear: [f64; 3]) -> [u8; 3] {
    linear.map(|value| {
        let value = value.clamp(0.0, 1.0);
        let srgb = if value <= 0.003_130_8 {
            value * 12.92
        } else {
            1.055_f64.mul_add(value.powf(1.0 / 2.4), -0.055)
        };
        to_u8(srgb)
    })
}

fn linear_to_oklab(linear: [f64; 3]) -> Oklab {
    let [red, green, blue] = linear;
    let long = 0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue;
    let medium = 0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue;
    let short = 0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue;
    let (long, medium, short) = (long.cbrt(), medium.cbrt(), short.cbrt());
    Oklab {
        lightness: 0.210_454_255_3 * long + 0.793_617_785 * medium - 0.004_072_046_8 * short,
        green_red: 1.977_998_495_1 * long - 2.428_592_205 * medium + 0.450_593_709_9 * short,
        blue_yellow: 0.025_904_037_1 * long + 0.782_771_766_2 * medium - 0.808_675_766 * short,
    }
}

fn oklab_to_linear(color: Oklab) -> [f64; 3] {
    let long =
        color.lightness + 0.396_337_777_4 * color.green_red + 0.215_803_757_3 * color.blue_yellow;
    let medium =
        color.lightness - 0.105_561_345_8 * color.green_red - 0.063_854_172_8 * color.blue_yellow;
    let short =
        color.lightness - 0.089_484_177_5 * color.green_red - 1.291_485_548 * color.blue_yellow;
    let (long, medium, short) = (long.powi(3), medium.powi(3), short.powi(3));
    [
        (4.076_741_662_1 * long - 3.307_711_591_3 * medium + 0.230_969_929_2 * short)
            .clamp(0.0, 1.0),
        (-1.268_438_004_6 * long + 2.609_757_401_1 * medium - 0.341_319_396_5 * short)
            .clamp(0.0, 1.0),
        (-0.004_196_086_3 * long - 0.703_418_614_7 * medium + 1.707_614_701 * short)
            .clamp(0.0, 1.0),
    ]
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn to_u8(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
#[path = "contact_tests.rs"]
mod tests;
