use crate::images::NetworkImages;
use crate::widgets::item_ui;
use crate::OssFile;
use bytesize::ByteSize;
use cc_core::{tracing, GetObjectInfo, ObjectList, OssConfig, OssError};
use egui_modal::{Icon, Modal};
use std::sync::mpsc;
use tokio::runtime;

enum Update {
    Uploaded(Result<String, OssError>),
    List(Result<ObjectList, OssError>),
}

#[derive(Clone, Copy, PartialEq)]
enum ShowType {
    List,
    Thumb,
}

#[derive(PartialEq)]
enum State {
    Idle(Route),
    Busy(Route),
}

#[derive(Clone, Copy, PartialEq)]
enum Route {
    Upload,
    List,
}

pub struct App {
    oss: OssConfig,
    rt: runtime::Runtime,
    list: Vec<OssFile>,
    current_img: OssFile,
    update_tx: mpsc::SyncSender<Update>,
    update_rx: mpsc::Receiver<Update>,
    state: State,
    err: Option<String>,
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    net_images: NetworkImages,
    show_type: ShowType,
    preview_modal: Modal,
    dialog: Modal,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let oss = OssConfig::new();
        let (update_tx, update_rx) = mpsc::sync_channel(1);
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let preview_modal = Modal::new(&cc.egui_ctx, "preview");

        let dialog = Modal::new(&cc.egui_ctx, "my_dialog");

        let mut this = Self {
            oss,
            rt,
            current_img: OssFile::default(),
            list: vec![],
            update_tx,
            update_rx,
            state: State::Idle(Route::List),
            err: None,
            dropped_files: vec![],
            picked_path: None,
            net_images: NetworkImages::new(cc.egui_ctx.clone()),
            show_type: ShowType::List,
            preview_modal,
            dialog,
        };

        this.get_list(&cc.egui_ctx);

        this
    }

    fn upload_file(&mut self, ctx: &egui::Context) {
        if let Some(picked_path) = self.picked_path.clone() {
            self.picked_path = None;
            self.state = State::Busy(Route::Upload);

            let update_tx = self.update_tx.clone();
            let ctx = ctx.clone();
            let oss = self.oss.clone();

            self.rt.spawn(async move {
                let res = oss.put(picked_path).await;
                update_tx.send(Update::Uploaded(res)).unwrap();
                ctx.request_repaint();
            });
        }
    }

    fn get_list(&mut self, ctx: &egui::Context) {
        self.state = State::Busy(Route::List);
        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();
        let oss = self.oss.clone();

        self.rt.block_on(async move {
            let res = oss.get_list().await;
            update_tx.send(Update::List(res)).unwrap();
            ctx.request_repaint();
        });
    }

    fn render_list(&mut self, ui: &mut egui::Ui) {
        let num_rows = self.list.len();
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            // .enable_scrolling(false)
            .id_source("content_scroll")
            .show_rows(ui, text_height, num_rows, |ui, row_range| {
                for i in row_range {
                    let data = self.list.get(i).unwrap();
                    if ui
                        .add(egui::Label::new(&data.name).sense(egui::Sense::click()))
                        .on_hover_text(&data.url)
                        .clicked()
                    {
                        self.current_img = data.clone();
                        self.preview_modal.open();
                        ui.ctx().request_repaint();
                    }
                    // ui.label(&data.size);
                    // ui.label(&data.last_modified);
                }
            });
    }

    fn render_thumb(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            // .enable_scrolling(false)
            .id_source("content_scroll")
            .show(ui, |ui| {
                ui.with_layout(
                    egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true),
                    |ui| {
                        // ui.spacing_mut().item_spacing.x = 0.0;
                        for d in &self.list {
                            let url = d.url.clone();
                            self.net_images.add(url.clone());
                            let resp = item_ui(ui, d.clone(), &self.net_images);
                            if resp.on_hover_text(url).clicked() {
                                self.current_img = d.clone();
                                self.preview_modal.open();
                                ui.ctx().request_repaint();
                            }
                        }
                    },
                );
            });
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.button("Upload file...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_path = Some(path.display().to_string());
            }
        }
        let enabled =
            self.state != State::Busy(Route::List) && self.state != State::Busy(Route::Upload);
        ui.add_enabled_ui(enabled, |ui| {
            if ui.button("\u{1f503}").clicked() {
                self.get_list(ctx);
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            ui.style_mut().spacing.item_spacing.x = 0.0;
            egui::Frame {
                fill: ui.style().visuals.widgets.inactive.bg_fill,
                rounding: egui::Rounding::same(2.0),
                ..egui::Frame::default()
            }
            .show(ui, |ui| {
                ui.style_mut().visuals.button_frame = false;
                ui.style_mut().visuals.widgets.active.rounding = egui::Rounding::same(2.0);
                ui.selectable_value(&mut self.show_type, ShowType::Thumb, "\u{25a3}");
                ui.selectable_value(&mut self.show_type, ShowType::List, "\u{2630}");
            });
        });
    }
    fn status_bar_contents(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        egui::widgets::global_dark_light_mode_switch(ui);

        if let Some(err) = &self.err {
            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.button("\u{1f4ac}").clicked() {
                self.dialog.open_dialog(
                    Some("Info"),       // title
                    Some("Working..."), // body
                    Some(Icon::Info),   // icon
                )
            }
        });
    }

    fn show_image(&mut self, ctx: &egui::Context) {
        let Self {
            preview_modal: show_modal,
            current_img,
            ..
        } = self;

        if current_img.url.is_empty() {
            return;
        }

        self.net_images.add(current_img.url.clone());

        show_modal.show(|ui| {
            show_modal.title(ui, "Preview");
            show_modal.frame(ui, |ui| {
                ui.vertical(|ui| {
                    let url = format!("{}/{}", self.oss.url, current_img.key);

                    if ui
                        .add(
                            egui::Label::new(
                                egui::RichText::new(format!("url: {}{}", url.clone(), "\u{1f4cb}"))
                                    .monospace(),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .on_hover_text("Click to copy")
                        .clicked()
                    {
                        ui.output().copied_text = url;
                    }
                    ui.monospace(format!("size: {}", current_img.size));
                    ui.monospace(format!("last modified: {}", current_img.last_modified));
                });
                let win_size = ctx.input().screen_rect().size();
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .max_height(win_size.y - 200.0)
                    .show(ui, |ui| {
                        if let Some(img) = self.net_images.get_image(current_img.url.clone()) {
                            let mut size = img.size_vec2();
                            size *= (ui.available_width() / size.x).min(1.0);
                            img.show_size(ui, size);
                        }
                    });
            });
            show_modal.buttons(ui, |ui| {
                show_modal.suggested_button(ui, "Ok");
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                Update::Uploaded(result) => match result {
                    Ok(str) => {
                        self.state = State::Idle(Route::Upload);
                        tracing::info!("{}", str);
                    }
                    Err(err) => {
                        self.state = State::Idle(Route::Upload);
                        self.err = Some(err.message());
                    }
                },
                Update::List(result) => match result {
                    Ok(str) => {
                        for data in str.object_list {
                            let (base, last_modified, _etag, _typ, size, _storage_class) =
                                data.pieces();
                            let key = base.path().to_string();
                            let url = self.oss.get_file_url(key.clone());
                            let name = key.replace(&self.oss.path, "").replace("/", "");

                            self.list.push(OssFile {
                                name,
                                key,
                                url,
                                size: format!("{}", ByteSize(size)),
                                last_modified: last_modified
                                    .format("%Y-%m-%d %H:%M:%S")
                                    .to_string(),
                            });
                        }

                        self.state = State::Idle(Route::List);
                    }
                    Err(err) => {
                        self.state = State::Idle(Route::List);
                        self.err = Some(err.message());
                    }
                },
            }
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::style::Margin::symmetric(0.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        self.bar_contents(ui, ctx);
                    });
                });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.status_bar_contents(ui, ctx);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::style::Margin::same(0.0))
                .show(ui, |ui| {
                    match &mut self.state {
                        State::Idle(ref mut route) => match route {
                            Route::Upload => {
                                //
                            }
                            Route::List => match self.show_type {
                                ShowType::List => self.render_list(ui),
                                ShowType::Thumb => self.render_thumb(ui),
                            },
                        },
                        State::Busy(route) => {
                            ui.centered_and_justified(|ui| match route {
                                Route::Upload => {
                                    ui.spinner();
                                    ui.heading("Uploading file...");
                                }
                                Route::List => {
                                    ui.spinner();
                                    ui.heading("Getting file list...");
                                }
                            });
                        }
                    };
                });
        });

        self.show_image(ctx);
        self.dialog.show_dialog();

        if !self.dropped_files.is_empty() {
            // for file in &self.dropped_files {

            // }
            let file = self.dropped_files.first().unwrap();
            let info = if let Some(path) = &file.path {
                path.display().to_string()
            } else if !file.name.is_empty() {
                file.name.clone()
            } else {
                "???".to_owned()
            };
            self.picked_path = Some(info);
        }

        self.upload_file(ctx);

        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }

        self.net_images.try_fetch();
    }
}
