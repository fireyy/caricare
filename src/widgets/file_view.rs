use crate::state::{State, Update};
use eframe::emath;
use egui::Vec2;

use super::confirm::ConfirmAction;
use crate::global;
use crate::util;
use cc_ui::icon;

#[derive(PartialEq)]
pub enum ZoomType {
    In,
    Out,
}

pub fn zoomratio(i: f32, s: f32) -> f32 {
    i * s * 0.1
}

fn mouse_wheel_zoom(delta: f32, pointer_delta: Vec2, state: &mut State) {
    let delta = zoomratio((delta / 10.).max(-5.0).min(5.0), state.img_zoom);
    let new_scale = state.img_zoom + delta;
    // limit scale
    if new_scale > 0.01 && new_scale < 40. {
        state.img_zoom_offset -=
            scale_pt(state.img_zoom_offset, pointer_delta, state.img_zoom, delta);
        state.img_zoom += delta;
    }
}

fn scale_pt(origin: Vec2, pt: Vec2, scale: f32, scale_inc: f32) -> Vec2 {
    ((pt - origin) * scale_inc) / scale
}

fn zoom_action(state: &mut State, zoom_type: ZoomType) {
    let i = if zoom_type == ZoomType::In { 3.5 } else { -3.5 };
    let delta = zoomratio(i, state.img_zoom);
    let new_scale = state.img_zoom + delta;
    // limit scale
    if new_scale > 0.05 && new_scale < 40. {
        // We want to zoom towards the center
        let new_zoom = state.img_zoom + delta;
        let pos: emath::Pos2 = state.disp_rect.pos.into();
        let size: emath::Vec2 = state.disp_rect.size.into();
        let offset = size * 0.5;
        let ratio = new_zoom / state.img_zoom;
        let x = ratio * (pos.x + offset.x) - offset.x;
        let y = ratio * (pos.y + offset.y) - offset.y;

        state.img_scroll = Some(emath::Pos2::new(x, y));

        state.img_zoom = new_zoom;
        println!(
            "offset: {:?}, zoom: {}",
            state.img_zoom_offset, state.img_zoom
        )
    }
}

pub fn file_view_ui(ctx: &egui::Context, state: &mut State) {
    if state.current_object.key().is_empty() || !state.current_object.is_file() {
        return;
    }

    let win_size = ctx.input(|i| i.screen_rect).size();
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..global().cc_ui.bottom_panel_frame()
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
                    state.close_preview();
                }
            });
            let mut is_image = false;
            if let Some(file) = state.file_cache.check(state.current_object.key()) {
                if file.is_image() {
                    is_image = true;
                    let mut size = file.size_vec2();
                    state.img_default_zoom = if size.x > ui.available_width() {
                        ui.available_width() / size.x
                    } else {
                        1.0
                    };
                    if state.img_zoom == 1.0 {
                        state.img_zoom = state.img_default_zoom;
                    }
                    size = size * state.img_zoom;
                    let scroll = state.img_scroll.take();
                    let widget = if let Some(pos) = &scroll {
                        egui::ScrollArea::both()
                            .auto_shrink([false; 2])
                            .scroll_offset(pos.to_vec2())
                            .max_height(win_size.y - 110.0)
                    } else {
                        egui::ScrollArea::both()
                            .max_height(win_size.y - 110.0)
                            .auto_shrink([false; 2])
                    };
                    ui.spacing_mut().scroll_bar_inner_margin = 0.0;
                    let resp = widget.show(ui, |ui| {
                        file.show_size(ui, size);
                    });
                    let pos = resp.state.offset;
                    let display_rect = util::Rect {
                        pos: pos.into(),
                        size: resp.inner_rect.size().into(),
                    };
                    state.disp_rect = display_rect;
                    if ui.rect_contains_pointer(resp.inner_rect) {
                        let (zoom, pointer_delta, _pointer_down, modifiers) = ui.input(|i| {
                            let zoom = i.events.iter().find_map(|e| match e {
                                // egui::Event::Zoom(v) => Some(*v),
                                egui::Event::MouseWheel {
                                    unit: _unit,
                                    delta,
                                    modifiers: _modifiers,
                                } => Some(delta.y),
                                _ => None,
                            });
                            (
                                zoom,
                                i.pointer.interact_pos(),
                                i.pointer.primary_down(),
                                i.modifiers,
                            )
                        });
                        if modifiers.ctrl {
                            if let Some(zoom) = zoom {
                                tracing::info!("zoom: {:?}, pointer: {:?}", zoom, pointer_delta,);
                                if let Some(pointer_delta) = pointer_delta {
                                    mouse_wheel_zoom(zoom, pointer_delta.to_vec2(), state);
                                }
                            }
                        }
                    }
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
                        zoom_action(state, ZoomType::In);
                    }
                    if ui
                        .button(icon::ZOOM_ACTUAL)
                        .on_hover_text("Zoom to window size")
                        .clicked()
                    {
                        state.img_zoom = state.img_default_zoom;
                    }
                    if ui
                        .button(icon::ZOOM_OUT)
                        .on_hover_text("Zoom Out")
                        .clicked()
                    {
                        zoom_action(state, ZoomType::Out);
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
                    global()
                        .update_tx
                        .send(Update::Prompt((
                            "Please enter the link expiration (in seconds):".to_string(),
                            ConfirmAction::GenerateUrl(3600),
                        )))
                        .unwrap();
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
