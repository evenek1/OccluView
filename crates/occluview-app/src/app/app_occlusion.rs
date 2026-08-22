//! The occlusal contact reading: opening it, keeping it honest, and its panel.
//!
//! The panel carries one control that matters. Everything else on it is there
//! to say what the colours mean, because a heat map without a legend is a
//! picture rather than a measurement.

use eframe::egui;
use occluview_align::{ContactScale, LOAD_MAX_MM, LOAD_MIN_MM};
use occluview_core::{Scene, SceneMeshId};

use super::OccluViewApp;
use crate::occlusion::{antagonist_for, ContactPair, ContactReading};
use crate::ui_theme;

/// Width of the panel, in points.
const PANEL_WIDTH_PX: f32 = 268.0;
/// Height of the legend bar, in points.
const LEGEND_HEIGHT_PX: f32 = 12.0;
/// How many samples the legend bar is drawn from. One per point of its width is
/// wasteful; this is enough that no band boundary lands visibly on a step.
const LEGEND_STEPS: usize = 96;

impl OccluViewApp {
    /// Open a contact reading on `layer` against whatever it bites.
    pub(super) fn begin_contacts_from_layer(&mut self, scene: &Scene, layer: SceneMeshId) {
        let Some(antagonist) = antagonist_for(scene, layer) else {
            self.status_message =
                Some("A contact reading needs a second visible scan to measure against".into());
            return;
        };
        // Whatever was wearing marks stops wearing them the moment the reading
        // moves, or the operator is looking at two maps and one legend.
        self.clear_contact_marks();
        self.occlusion.open(ContactPair {
            painted: layer,
            antagonist,
        });
        self.occlusion_status = Some("Reading contacts…".into());
        self.submit_contacts_job();
    }

    /// Close the reading and take its marks off the scan.
    pub(super) fn close_contacts(&mut self) {
        self.clear_contact_marks();
        self.occlusion.close();
        self.occlusion_status = None;
    }

    /// Drop a reading whose scans are no longer both in the scene.
    ///
    /// Called from the frame loop rather than from every edit that could
    /// invalidate it: a layer can leave through a removal, an undo, a crop or a
    /// separate, and a reading left pointing at a missing scan would go on
    /// showing marks measured against something that is not there.
    pub(super) fn sync_contacts_with_scene(&mut self) {
        let Some(scene) = self.scene.clone() else {
            if self.occlusion.close().is_some() {
                self.occlusion_status = None;
            }
            return;
        };
        if let Some(orphaned) = self.occlusion.forget_missing(&scene) {
            self.strip_overlay_colors(orphaned);
            self.occlusion_status = None;
        }
    }

    /// Take the marks off whichever scan is wearing them.
    fn clear_contact_marks(&mut self) {
        if let Some(pair) = self.occlusion.pair() {
            self.strip_overlay_colors(pair.painted);
        }
    }

    /// The contact panel. Returns whether it took the pointer.
    pub(super) fn show_contact_overlay(
        &mut self,
        ui: &mut egui::Ui,
        viewport_rect: egui::Rect,
    ) -> bool {
        if !self.occlusion.is_open() {
            return false;
        }
        let rect = panel_rect(viewport_rect);
        let mut close = false;
        let mut resubmit = false;
        let mut reading = self.occlusion.reading();
        let mut load_mm = self.occlusion.load_mm();
        let title = self.contact_title();
        let status = self.occlusion_status.clone();
        let scale = self.occlusion.scale();

        let response = ui
            .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui_theme::overlay_frame().show(ui, |ui| {
                    ui.set_width(PANEL_WIDTH_PX - 20.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(title).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            close = ui
                                .small_button("✕")
                                .on_hover_text("Close reading")
                                .clicked();
                        });
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        for choice in [ContactReading::Marks, ContactReading::Approach] {
                            if ui
                                .selectable_label(reading == choice, choice.label())
                                .on_hover_text(hint_for(choice))
                                .clicked()
                            {
                                reading = choice;
                            }
                        }
                    });
                    ui.add_space(6.0);
                    // The one control. Named for what it does to the picture
                    // rather than for the field it sets, because the operator
                    // is choosing where heavy starts, not editing a parameter.
                    let slider = ui.add(
                        egui::Slider::new(&mut load_mm, LOAD_MIN_MM..=LOAD_MAX_MM)
                            .text("heavy at")
                            .suffix(" mm")
                            .fixed_decimals(2),
                    );
                    if slider.changed() {
                        resubmit = true;
                    }
                    ui.add_space(6.0);
                    paint_legend(ui, scale);
                    if let Some(status) = status {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(status)
                                .color(ui_theme::TEXT_WEAK)
                                .size(11.0),
                        );
                    }
                });
            })
            .response;

        if close {
            self.close_contacts();
            return true;
        }
        if self.occlusion.set_reading(reading) || (resubmit && self.occlusion.set_load_mm(load_mm))
        {
            self.submit_contacts_job();
        }
        response.hovered() || ui.rect_contains_pointer(rect)
    }

    /// What the panel calls the reading it is showing.
    fn contact_title(&self) -> String {
        let Some(pair) = self.occlusion.pair() else {
            return "Contacts".into();
        };
        let painted = self
            .layer_display_name(pair.painted)
            .unwrap_or_else(|| "this scan".to_owned());
        let against = self
            .layer_display_name(pair.antagonist)
            .unwrap_or_else(|| "the other scan".to_owned());
        format!("{painted} against {against}")
    }
}

/// What each reading is for, in one line.
fn hint_for(reading: ContactReading) -> &'static str {
    match reading {
        ContactReading::Marks => {
            "Where the scans meet, coloured by how hard — the rest stays bare, as paper leaves it"
        }
        ContactReading::Approach => "How close the other scan is everywhere, load included",
    }
}

/// Where the panel sits: bottom-left of the viewport, clear of the layer list
/// on the right and the scale bar along the bottom edge.
fn panel_rect(viewport_rect: egui::Rect) -> egui::Rect {
    let width = PANEL_WIDTH_PX.min(viewport_rect.width() - 24.0).max(0.0);
    egui::Rect::from_min_size(
        egui::pos2(
            viewport_rect.left() + 12.0,
            (viewport_rect.bottom() - 168.0).max(viewport_rect.top() + 12.0),
        ),
        egui::vec2(width, 0.0),
    )
}

/// The ramp as a bar, with the two numbers that make it readable.
///
/// Drawn from the scale itself rather than from a copy of its colours, so a
/// legend can never describe a ramp the surface is not wearing.
fn paint_legend(ui: &mut egui::Ui, scale: ContactScale) {
    let law = scale.law();
    let deepest = -scale.load_mm() * (law.clamp_mm / law.load_mm);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), LEGEND_HEIGHT_PX),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    #[allow(clippy::cast_precision_loss)]
    let steps = LEGEND_STEPS as f32;
    for step in 0..LEGEND_STEPS {
        #[allow(clippy::cast_precision_loss)]
        let fraction = step as f32 / steps;
        let signed = f64::from(fraction).mul_add(deepest - law.paint_far_mm, law.paint_far_mm);
        let [red, green, blue, alpha] = scale.color_at(signed);
        let cell = egui::Rect::from_min_size(
            egui::pos2(rect.left() + fraction * rect.width(), rect.top()),
            egui::vec2(rect.width() / steps + 1.0, rect.height()),
        );
        // Over the panel, not over the scan: the bar has to show the colour the
        // ramp gives, including where that colour is barely there at all.
        painter.rect_filled(cell, 0.0, ui_theme::panel_fill());
        painter.rect_filled(
            cell,
            0.0,
            egui::Color32::from_rgba_unmultiplied(red, green, blue, alpha),
        );
    }
    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, ui_theme::hairline()));
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("touch")
                .color(ui_theme::TEXT_WEAK)
                .size(10.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.2} mm into the bite", -deepest))
                    .color(ui_theme::TEXT_WEAK)
                    .size(10.0),
            );
        });
    });
}
