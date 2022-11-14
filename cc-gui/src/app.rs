use crate::images::NetworkImages;
use crate::widgets::item_ui;
use crate::OssFile;
use bytesize::ByteSize;
use cc_core::{tracing, ObjectList, OssConfig, OssError};
use egui_extras::{Size, TableBuilder};
use std::sync::mpsc;
use tokio::runtime;

enum Update {
    Uploaded(Result<String, OssError>),
    List(Result<ObjectList, OssError>),
}

#[derive(PartialEq)]
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
    update_tx: mpsc::SyncSender<Update>,
    update_rx: mpsc::Receiver<Update>,
    state: State,
    err: Option<String>,
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    net_images: NetworkImages,
    show_type: ShowType,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let oss = OssConfig::new();
        let (update_tx, update_rx) = mpsc::sync_channel(1);
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let mut this = Self {
            oss,
            rt,
            list: vec![],
            update_tx,
            update_rx,
            state: State::Idle(Route::List),
            err: None,
            dropped_files: vec![],
            picked_path: None,
            net_images: NetworkImages::new(cc.egui_ctx.clone()),
            show_type: ShowType::List,
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

        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::remainder().at_least(60.0))
            .column(Size::initial(60.0).at_least(40.0))
            .column(Size::initial(120.0).at_least(80.0))
            .resizable(true);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("Size");
                });
                header.col(|ui| {
                    ui.heading("Last Modified");
                });
            })
            .body(|body| {
                body.rows(text_height, num_rows, |row_index, mut row| {
                    let data = self.list.get(row_index).unwrap();
                    row.col(|ui| {
                        ui.label(&data.name).on_hover_text(&data.url);
                    });
                    row.col(|ui| {
                        ui.label(&data.size);
                    });
                    row.col(|ui| {
                        ui.label(&data.last_modified);
                    });
                });
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
                                //
                            }
                        }
                    },
                );
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
                        for d in str.object_list {
                            let url = self.oss.get_file_url(d.key.clone());
                            self.list.push(OssFile {
                                name: d.key.replace(&self.oss.path, "").replace("/", ""),
                                url,
                                size: format!("{}", ByteSize(d.size)),
                                last_modified: d
                                    .last_modified
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Upload file...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.picked_path = Some(path.display().to_string());
                    }
                }
                let enabled = self.state != State::Busy(Route::List)
                    && self.state != State::Busy(Route::Upload);
                ui.add_enabled_ui(enabled, |ui| {
                    if ui.button("\u{1f503}").clicked() {
                        self.get_list(ctx);
                    }
                });
                if let Some(err) = &self.err {
                    ui.set_min_width(100.0);
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.selectable_value(&mut self.show_type, ShowType::Thumb, "\u{25a3}");
                    ui.selectable_value(&mut self.show_type, ShowType::List, "\u{2630}");
                });
            });
            ui.separator();
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
