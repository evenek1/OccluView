//! Tests for the occlusal contact view, split out of `occlusion.rs` to hold the
//! workspace's 800-line file budget.

use super::{antagonist_for, ContactPair, ContactReading, OcclusionView};
use glam::{Affine3A, Vec3};
use occluview_align::{LOAD_MAX_MM, LOAD_MIN_MM, TIGHTNESS};
use occluview_core::{Mesh, Scene, SceneMesh, SceneMeshId, Vertex};

/// A one-triangle surface, so a layer has a real bounding box to sit in.
fn surface() -> Mesh {
    let corner = |position: [f32; 3]| Vertex {
        position,
        normal: [0.0, 0.0, 1.0],
        color: [255, 255, 255, 255],
        uv: [0.0, 0.0],
    };
    Mesh::new(
        None,
        vec![
            corner([0.0, 0.0, 0.0]),
            corner([1.0, 0.0, 0.0]),
            corner([0.0, 1.0, 0.0]),
        ],
        vec![0, 1, 2],
    )
    .unwrap_or_else(|_| Mesh::empty())
}

fn layer_at(scene: &mut Scene, x: f32) -> SceneMeshId {
    let entry = SceneMesh::new(surface())
        .with_transform(Affine3A::from_translation(Vec3::new(x, 0.0, 0.0)));
    let id = entry.id();
    scene.add(entry);
    id
}

#[test]
fn the_antagonist_is_the_other_scan() {
    let mut scene = Scene::new();
    let lower = layer_at(&mut scene, 0.0);
    let upper = layer_at(&mut scene, 1.0);

    assert_eq!(antagonist_for(&scene, lower), Some(upper));
    assert_eq!(antagonist_for(&scene, upper), Some(lower));
}

#[test]
fn a_lone_scan_has_nothing_to_bite_against() {
    let mut scene = Scene::new();
    let only = layer_at(&mut scene, 0.0);

    assert_eq!(antagonist_for(&scene, only), None);
}

#[test]
fn a_hidden_scan_is_not_an_antagonist() {
    // Measuring against it would put the reading on a surface nobody can see,
    // and the operator would be looking at marks with no visible cause.
    let mut scene = Scene::new();
    let subject = layer_at(&mut scene, 0.0);
    let hidden = layer_at(&mut scene, 1.0);
    let visible = layer_at(&mut scene, 40.0);
    if let Some(entry) = scene
        .meshes_mut()
        .iter_mut()
        .find(|entry| entry.id() == hidden)
    {
        entry.visible = false;
    }

    assert_eq!(antagonist_for(&scene, subject), Some(visible));
}

#[test]
fn the_nearest_scan_wins_when_a_case_carries_more_than_two() {
    // A waxup or a preoperative copy in the same scene must not steal the
    // reading from the arch actually opposite.
    let mut scene = Scene::new();
    let subject = layer_at(&mut scene, 0.0);
    let opposite = layer_at(&mut scene, 2.0);
    let _far_copy = layer_at(&mut scene, 80.0);

    assert_eq!(antagonist_for(&scene, subject), Some(opposite));
}

#[test]
fn a_scan_that_left_the_scene_has_no_antagonist() {
    let mut scene = Scene::new();
    let gone = layer_at(&mut scene, 0.0);
    let _other = layer_at(&mut scene, 1.0);
    scene.remove(0);

    assert_eq!(antagonist_for(&scene, gone), None);
}

#[test]
fn a_reading_opens_on_the_law_its_own_depth() {
    // Not on wherever a previous case left the slider: the map an operator
    // first sees has to be the one the law was designed around.
    let mut scene = Scene::new();
    let painted = layer_at(&mut scene, 0.0);
    let antagonist = layer_at(&mut scene, 1.0);
    let mut view = OcclusionView::default();
    view.set_load_mm(LOAD_MAX_MM);

    view.open(ContactPair {
        painted,
        antagonist,
    });

    assert!((view.load_mm() - TIGHTNESS.load_mm).abs() < 1e-9);
    assert!(view.is_open());
}

#[test]
fn the_slider_is_bounded_and_survives_nonsense() {
    let mut view = OcclusionView::default();

    assert!(view.set_load_mm(-4.0));
    assert!((view.load_mm() - LOAD_MIN_MM).abs() < 1e-9);
    assert!(view.set_load_mm(900.0));
    assert!((view.load_mm() - LOAD_MAX_MM).abs() < 1e-9);
    assert!(
        !view.set_load_mm(f64::NAN),
        "a non-finite setting is not a move"
    );
    assert!((view.load_mm() - LOAD_MAX_MM).abs() < 1e-9);
    assert!(
        !view.set_load_mm(LOAD_MAX_MM),
        "the same value is not a move"
    );
}

#[test]
fn switching_reading_takes_the_new_law_own_depth() {
    // The two laws call different depths loaded. Carrying a number across
    // would silently re-scale the map the operator was just looking at.
    let mut view = OcclusionView::default();
    view.set_load_mm(0.4);

    assert!(view.set_reading(ContactReading::Approach));

    assert!((view.load_mm() - ContactReading::Approach.law().load_mm).abs() < 1e-9);
    assert!(
        !view.set_reading(ContactReading::Approach),
        "re-selecting the same reading is not a change"
    );
}

#[test]
fn a_reading_whose_scan_left_the_scene_is_dropped() {
    let mut scene = Scene::new();
    let painted = layer_at(&mut scene, 0.0);
    let antagonist = layer_at(&mut scene, 1.0);
    let mut view = OcclusionView::default();
    view.open(ContactPair {
        painted,
        antagonist,
    });

    // The antagonist goes: the reading is meaningless, but the marks are still
    // on a layer that is still there and have to be taken off it.
    scene.remove(1);
    let orphaned = view.forget_missing(&scene);

    assert_eq!(orphaned, Some(painted));
    assert!(!view.is_open());
}

#[test]
fn a_reading_whose_painted_scan_left_the_scene_leaves_nothing_to_clean() {
    let mut scene = Scene::new();
    let painted = layer_at(&mut scene, 0.0);
    let antagonist = layer_at(&mut scene, 1.0);
    let mut view = OcclusionView::default();
    view.open(ContactPair {
        painted,
        antagonist,
    });

    scene.remove(0);

    assert_eq!(view.forget_missing(&scene), None);
    assert!(!view.is_open());
}

#[test]
fn an_intact_reading_is_left_alone() {
    let mut scene = Scene::new();
    let painted = layer_at(&mut scene, 0.0);
    let antagonist = layer_at(&mut scene, 1.0);
    let mut view = OcclusionView::default();
    view.open(ContactPair {
        painted,
        antagonist,
    });

    assert_eq!(view.forget_missing(&scene), None);
    assert!(view.is_open());
}
