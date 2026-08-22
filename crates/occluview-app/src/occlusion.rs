//! OCCLUSION: one scan painted by how hard it meets the scan it bites against.
//!
//! The measurement is the deviation map the align worker already computes, and
//! the colour is [`occluview_align::ContactScale`]. This module is the part in
//! between: which two scans the reading runs across, which law it is read
//! under, and the one number the operator drives.
//!
//! One number, because judging a bite is a question about a threshold — where
//! does close stop being contact and start being pressure — and the honest way
//! to answer it is to move the threshold and watch the map, not to type a value
//! and wait. Nothing the slider touches changes a distance, so nothing it
//! touches re-measures: the worker's cache is keyed on what the distances
//! depend on, and the scale is not one of those things.

use occluview_align::{ContactLaw, ContactScale, CLINICAL, LOAD_MAX_MM, LOAD_MIN_MM, TIGHTNESS};
use occluview_core::{Scene, SceneMesh, SceneMeshId};

/// Which reading the map is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContactReading {
    /// Where the arches touch, and how hard. Digital articulating paper.
    Marks,
    /// How close the antagonist is everywhere, load included.
    Approach,
}

impl ContactReading {
    /// The colour law for this reading.
    pub(crate) fn law(self) -> &'static ContactLaw {
        match self {
            Self::Marks => &TIGHTNESS,
            Self::Approach => &CLINICAL,
        }
    }

    /// What the button says.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Marks => "Contacts",
            Self::Approach => "Approach",
        }
    }
}

/// Which two scans a contact reading runs between.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContactPair {
    /// The scan wearing the marks.
    pub(crate) painted: SceneMeshId,
    /// The scan it is measured against.
    pub(crate) antagonist: SceneMeshId,
}

/// The occlusal contact view: what it is showing, and how it is set.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OcclusionView {
    pair: Option<ContactPair>,
    reading: ContactReading,
    load_mm: f64,
}

impl Default for OcclusionView {
    fn default() -> Self {
        Self {
            pair: None,
            reading: ContactReading::Marks,
            load_mm: TIGHTNESS.load_mm,
        }
    }
}

impl OcclusionView {
    /// Which two scans the reading runs between, or `None` when it is closed.
    pub(crate) fn pair(self) -> Option<ContactPair> {
        self.pair
    }

    /// Whether a reading is on screen.
    pub(crate) fn is_open(self) -> bool {
        self.pair.is_some()
    }

    /// The reading currently shown.
    pub(crate) fn reading(self) -> ContactReading {
        self.reading
    }

    /// The load depth the slider is at, in millimetres.
    pub(crate) fn load_mm(self) -> f64 {
        self.load_mm
    }

    /// The scale the map is painted with.
    pub(crate) fn scale(self) -> ContactScale {
        ContactScale::new(self.reading.law(), self.load_mm)
    }

    /// Open a reading on `painted` against `antagonist`.
    ///
    /// The load depth resets to the law's own, so opening a reading always
    /// starts from the number the law was designed around rather than from
    /// wherever a previous case left the slider.
    pub(crate) fn open(&mut self, pair: ContactPair) {
        self.pair = Some(pair);
        self.load_mm = self.reading.law().load_mm;
    }

    /// Close the reading. Returns the scan that was wearing the marks, so the
    /// caller can take them off it.
    pub(crate) fn close(&mut self) -> Option<SceneMeshId> {
        self.pair.take().map(|pair| pair.painted)
    }

    /// Move the slider. Returns whether it actually moved.
    pub(crate) fn set_load_mm(&mut self, load_mm: f64) -> bool {
        let next = if load_mm.is_finite() {
            load_mm.clamp(LOAD_MIN_MM, LOAD_MAX_MM)
        } else {
            return false;
        };
        let moved = (next - self.load_mm).abs() > f64::EPSILON;
        self.load_mm = next;
        moved
    }

    /// Switch reading. Returns whether it actually changed.
    ///
    /// The load depth follows the law, because the two laws call different
    /// depths "loaded" and carrying a number across would silently re-scale
    /// the map the operator was just looking at.
    pub(crate) fn set_reading(&mut self, reading: ContactReading) -> bool {
        if self.reading == reading {
            return false;
        }
        self.reading = reading;
        self.load_mm = reading.law().load_mm;
        true
    }

    /// Drop a reading that names a layer no longer in the scene, or one whose
    /// partner has gone. Returns the layer to clean the marks off, if any.
    pub(crate) fn forget_missing(&mut self, scene: &Scene) -> Option<SceneMeshId> {
        let pair = self.pair?;
        let present = |id: SceneMeshId| scene.meshes().iter().any(|entry| entry.id() == id);
        if present(pair.painted) && present(pair.antagonist) {
            return None;
        }
        self.pair = None;
        present(pair.painted).then_some(pair.painted)
    }
}

/// The scan `layer` bites against, or `None` when nothing in the scene can be.
///
/// The nearest visible surface, and deliberately nothing cleverer. With the two
/// scans a bite is made of there is only one answer and any rule finds it; with
/// more than two — a waxup, a preoperative copy — nearest is the one an
/// operator can predict without being told the rule. Hidden layers and point
/// clouds are not candidates: a point cloud has no surface to measure to, and a
/// hidden scan would put the reading on a surface nobody can see.
pub(crate) fn antagonist_for(scene: &Scene, layer: SceneMeshId) -> Option<SceneMeshId> {
    let subject = scene.meshes().iter().find(|entry| entry.id() == layer)?;
    let subject_centre = subject.world_bbox().center();
    scene
        .meshes()
        .iter()
        .filter(|entry| entry.id() != layer)
        .filter(|entry| entry.visible && !entry.mesh.is_point_cloud())
        .filter(|entry| !entry.world_bbox().is_empty())
        .min_by(|left, right| {
            let left_gap = left.world_bbox().center().distance_squared(subject_centre);
            let right_gap = right.world_bbox().center().distance_squared(subject_centre);
            left_gap.total_cmp(&right_gap)
        })
        .map(SceneMesh::id)
}

#[cfg(test)]
#[path = "occlusion_tests.rs"]
mod tests;
