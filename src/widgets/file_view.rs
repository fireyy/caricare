use crate::state::Update;
use cc_storage::Object;

use super::confirm::ConfirmAction;
use super::toasts::ToastKind;
use crate::global;
use cc_ui::icon;

#[derive(PartialEq)]
pub enum ZoomType {
    In,
    Out,
}

fn zoomratio(i: f32, s: f32) -> f32 {
    i * s * 0.1
}

pub struct FileView {
    pub is_preview: bool,
    pub img_zoom: f32,
    pub img_default_zoom: f32,
}

impl FileView {
    pub fn new() -> Self {
        Self {
            is_preview: false,
            img_zoom: 1.0,
            img_default_zoom: 1.0,
        }
    }

    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        current_object: &Object,
        current_file: Option<&cc_files::FileType>,
    ) {
        let mut url = current_object.url();
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
            .show_animated(ctx, self.is_preview, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui
                        .button(icon::CLOSE)
                        .on_hover_text("Close panel")
                        .clicked()
                    {
                        self.close();
                    }
                });
                let mut is_image = false;
                if let Some(file) = current_file {
                    if file.is_image() {
                        is_image = true;
                        ui.spacing_mut().scroll.bar_inner_margin = 0.0;
                        let resp = egui::ScrollArea::both()
                            .max_height(win_size.y - 110.0)
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                if !url.is_empty() {
                                    let mut size: egui::Vec2 = egui::Vec2::ZERO;
                                    let mut image = egui::Image::from_uri(url);
                                    if let Ok(img) =
                                        image.load_for_size(ui.ctx(), ui.available_size())
                                    {
                                        if let Some(size2) = img.size() {
                                            self.img_default_zoom =
                                                if size2.x > ui.available_width() {
                                                    ui.available_width() / size2.x
                                                } else {
                                                    1.0
                                                };
                                            size = size2;
                                        }
                                    }
                                    if self.img_zoom == 1.0 {
                                        self.img_zoom = self.img_default_zoom;
                                    }
                                    size *= self.img_zoom;
                                    image = image.fit_to_exact_size(size);
                                    ui.add_sized(ui.available_size(), image);
                                }
                            });
                        if ui.rect_contains_pointer(resp.inner_rect) {
                            let (zoom, _pointer_delta, _pointer_down, _modifiers) = ui.input(|i| {
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
                            if let Some(zoom) = zoom {
                                // tracing::info!("zoom: {:?}", zoom);
                                self.mouse_wheel_zoom(zoom);
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
                            self.zoom_action(ZoomType::In);
                        }
                        if ui
                            .button(icon::ZOOM_ACTUAL)
                            .on_hover_text("Zoom to window size")
                            .clicked()
                        {
                            self.img_zoom = self.img_default_zoom;
                        }
                        if ui
                            .button(icon::ZOOM_OUT)
                            .on_hover_text("Zoom Out")
                            .clicked()
                        {
                            self.zoom_action(ZoomType::Out);
                        }
                    });
                }
                ui.horizontal(|ui| {
                    ui.label(format!("{} Link:", icon::LINK));
                    ui.add(
                        egui::TextEdit::singleline(&mut url)
                            .desired_width(ui.available_width() - 50.0),
                    );
                    if ui
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
                        global()
                            .update_tx
                            .send(Update::Toast(("Copied!".to_string(), ToastKind::Success)))
                            .unwrap();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} Size: {}",
                        icon::SIZE,
                        current_object.size_string()
                    ));
                    ui.label(format!(
                        "{} Last Modified: {}",
                        icon::DATE,
                        current_object.date_string()
                    ));
                });
            });
    }

    pub fn close(&mut self) {
        self.is_preview = false;
        self.reset();
        global().update_tx.send(Update::CloseObject).unwrap();
    }

    pub fn reset(&mut self) {
        self.img_zoom = 1.0;
        self.img_default_zoom = 1.0;
    }

    fn mouse_wheel_zoom(&mut self, delta: f32) {
        let delta = zoomratio((delta / 10.).clamp(-5.0, 5.0), self.img_zoom);
        let new_scale = self.img_zoom + delta;
        // limit scale
        if new_scale > 0.01 && new_scale < 40. {
            self.img_zoom = new_scale;
        }
    }

    fn zoom_action(&mut self, zoom_type: ZoomType) {
        let i = if zoom_type == ZoomType::In { 3.5 } else { -3.5 };
        let delta = zoomratio(i, self.img_zoom);
        let new_scale = self.img_zoom + delta;
        // limit scale
        if new_scale > 0.05 && new_scale < 40. {
            self.img_zoom = new_scale;
        }
    }

    pub fn show(&mut self) {
        self.is_preview = true;
    }
}
