use crate::state::State;
use egui::Vec2;

use super::confirm::ConfirmAction;
use crate::theme::icon;

#[derive(PartialEq)]
pub enum ZoomType {
    In,
    Out,
}

pub fn zoomratio(i: f32, s: f32) -> f32 {
    i * s * 0.1
}

fn mouse_wheel_zoom(delta: f32, pointer_delta: Vec2, state: &mut State) {
    let delta = zoomratio((delta - 1.0) * 2.0, state.img_zoom);
    let new_scale = state.img_zoom + delta;
    // limit scale
    if new_scale > 0.01 && new_scale < 40. {
        state.offset -= scale_pt(state.offset, pointer_delta, state.img_zoom, delta);
        state.img_zoom += delta;
    }
}

fn scale_pt(origin: Vec2, pt: Vec2, scale: f32, scale_inc: f32) -> Vec2 {
    ((pt - origin) * scale_inc) / scale
}

fn zoom_action(win_size: egui::Vec2, state: &mut State, zoom_type: ZoomType) {
    let i = if zoom_type == ZoomType::In { 3.5 } else { -3.5 };
    let delta = zoomratio(i, state.img_zoom);
    let new_scale = state.img_zoom + delta;
    // limit scale
    if new_scale > 0.05 && new_scale < 40. {
        // We want to zoom towards the center
        let center = Vec2::new(win_size.x / 2., win_size.y / 2.);
        state.offset -= scale_pt(state.offset, center, state.img_zoom, delta);
        state.img_zoom += delta;
    }
}

pub fn file_view_ui(ctx: &egui::Context, state: &mut State) {
    if state.current_object.key().is_empty() || !state.current_object.is_file() {
        return;
    }

    let win_size = ctx.input(|i| i.screen_rect).size();
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..state.cc_ui.bottom_panel_frame()
    };
    egui::SidePanel::right("preview_panel")
        // .min_width(200.0)
        .resizable(true)
        .frame(frame)
        .max_width(400.0)
        .show_animated(ctx, state.is_preview, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui
                    .button(icon::CLOSE)
                    .on_hover_text("Close panel")
                    .clicked()
                {
                    state.is_preview = false;
                }
            });
            let mut is_image = false;
            if let Some(file) = state.file_cache.check(state.current_object.key()) {
                if file.is_image() {
                    is_image = true;
                    let mut size = file.size_vec2();
                    size = if state.img_zoom != 1.0 {
                        size * state.img_zoom
                    } else {
                        size * (ui.available_width() / size.x).min(1.0)
                    };
                    let resp = egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .max_height(win_size.y - 110.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                file.show_size(ui, size);
                            });
                        });
                    if ui.rect_contains_pointer(resp.inner_rect) {
                        let (zoom, pointer_delta, _pointer_down, _modifiers) = ui.input(|i| {
                            let zoom = i.events.iter().find_map(|e| match e {
                                egui::Event::Zoom(v) => Some(*v),
                                _ => None,
                            });
                            (
                                zoom,
                                i.pointer.interact_pos(),
                                i.pointer.primary_down(),
                                i.modifiers,
                            )
                        });
                        if let Some(zoom) = zoom {
                            // tracing::info!("zoom: {:?}, pointer: {:?}", zoom, pointer_delta,);
                            if let Some(pointer_delta) = pointer_delta {
                                mouse_wheel_zoom(zoom, pointer_delta.to_vec2(), state);
                            }
                        }
                    }
                } else if file.is_unknown() {
                    is_image = false;
                    ui.centered_and_justified(|ui| {
                        ui.set_height(win_size.y - 130.0);
                        ui.heading("Couldn't Preview File");
                    });
                } else {
                    is_image = false;
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .max_height(win_size.y - 88.0)
                        .show(ui, |ui| {
                            file.show(ui);
                        });
                }
            } else {
                ui.centered_and_justified(|ui| ui.spinner());
            }

            if is_image {
                ui.horizontal(|ui| {
                    ui.label(format!("{} Zoom: ", icon::CROSS_HAIR));
                    if ui.button(icon::ZOOM_IN).on_hover_text("Zoom In").clicked() {
                        zoom_action(win_size, state, ZoomType::In);
                    }
                    if ui
                        .button(icon::ZOOM_ACTUAL)
                        .on_hover_text("Zoom to window size")
                        .clicked()
                    {
                        state.img_zoom = 1.0;
                    }
                    if ui
                        .button(icon::ZOOM_OUT)
                        .on_hover_text("Zoom Out")
                        .clicked()
                    {
                        zoom_action(win_size, state, ZoomType::Out);
                    }
                });
            }
            ui.horizontal(|ui| {
                let mut url = state.current_object.url();
                ui.label(format!("{} Link:", icon::LINK));
                ui.add(
                    egui::TextEdit::singleline(&mut url).desired_width(ui.available_width() - 50.0),
                );
                if state.bucket_is_private()
                    && ui
                        .button(icon::REFRESH)
                        .on_hover_text("Generate Link")
                        .clicked()
                {
                    state.confirm.prompt(
                        "Please enter the link expiration (in seconds):",
                        ConfirmAction::GenerateUrl(3600),
                    );
                }
                if ui
                    .button(icon::CLIPBOARD)
                    .on_hover_text("Copy Link")
                    .clicked()
                {
                    ui.output_mut(|o| o.copied_text = url.to_string());
                    state.toasts.success("Copied!");
                }
            });
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} Size: {}",
                    icon::SIZE,
                    state.current_object.size_string()
                ));
                ui.label(format!(
                    "{} Last Modified: {}",
                    icon::DATE,
                    state.current_object.date_string()
                ));
            });
        });
}
