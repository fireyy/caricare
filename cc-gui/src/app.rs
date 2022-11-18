use crate::theme::text_ellipsis;
use crate::widgets::item_ui;
use crate::OssFile;
use bytesize::ByteSize;
use cc_core::{tracing, GetObjectInfo, ObjectList, OssConfig, OssError, Query};
use cc_image_cache::{ImageCache, ImageFetcher};
use egui_modal::{Icon, Modal};
use std::{sync::mpsc, vec};
use tokio::runtime;

static THUMB_LIST_WIDTH: f32 = 200.0;
static THUMB_LIST_HEIGHT: f32 = 50.0;

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
    show_type: ShowType,
    preview_modal: Modal,
    dialog: Modal,
    loading_more: bool,
    next_query: Option<Query>,
    scroll_top: bool,
    images: ImageCache,
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

        let query = Self::build_query(oss.path.clone());

        let images = ImageCache::new(ImageFetcher::spawn(cc.egui_ctx.clone()));

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
            show_type: ShowType::List,
            preview_modal,
            dialog,
            loading_more: false,
            next_query: Some(query),
            scroll_top: false,
            images,
        };

        this.get_list(&cc.egui_ctx);

        this
    }

    fn build_query(path: String) -> Query {
        let mut query = Query::new();
        query.insert("prefix", path);
        query.insert("max-keys", "40");
        query
    }

    fn upload_file(&mut self, ctx: &egui::Context) {
        if let Some(picked_path) = self.picked_path.clone() {
            self.picked_path = None;
            self.state = State::Busy(Route::Upload);

            let update_tx = self.update_tx.clone();
            let ctx = ctx.clone();
            let oss = self.oss.clone();

            self.rt.block_on(async move {
                let res = oss.put(picked_path).await;
                update_tx.send(Update::Uploaded(res)).unwrap();
                ctx.request_repaint();
            });
        }
    }

    fn get_list(&mut self, ctx: &egui::Context) {
        if let Some(query) = self.next_query.clone() {
            if !self.loading_more {
                self.state = State::Busy(Route::List);
            }
            let update_tx = self.update_tx.clone();
            let ctx = ctx.clone();
            let oss = self.oss.clone();
            self.rt.block_on(async move {
                let res = oss.get_list(query).await;
                update_tx.send(Update::List(res)).unwrap();
                ctx.request_repaint();
            });
        }
    }

    fn set_list(&mut self, obj: ObjectList) {
        let mut list = vec![];
        for data in obj.object_list {
            let (base, last_modified, _etag, _typ, size, _storage_class) = data.pieces();
            let key = base.path().to_string();
            let url = self.oss.get_file_url(key.clone());
            let name = key.replace(&self.oss.path, "").replace("/", "");

            list.push(OssFile {
                name,
                key,
                url,
                size: format!("{}", ByteSize(size)),
                last_modified: last_modified.format("%Y-%m-%d %H:%M:%S").to_string(),
            });
        }
        self.list.append(&mut list);
    }

    fn load_more(&mut self, ctx: &egui::Context) {
        tracing::info!("load more!");
        if self.next_query.is_some() {
            self.get_list(ctx);
        } else {
            //no more
        }
    }

    fn render_content(&mut self, ui: &mut egui::Ui) {
        let num_cols = match self.show_type {
            ShowType::List => 1,
            ShowType::Thumb => {
                let w = ui.ctx().input().screen_rect().size();
                (w.x / THUMB_LIST_WIDTH) as usize
            }
        };
        let num_rows = match self.show_type {
            ShowType::List => self.list.len(),
            ShowType::Thumb => (self.list.len() as f32 / num_cols as f32).ceil() as usize,
        };
        // tracing::info!("num_rows: {}", num_rows);
        let row_height = match self.show_type {
            ShowType::List => ui.text_style_height(&egui::TextStyle::Body),
            ShowType::Thumb => THUMB_LIST_HEIGHT,
        };

        let mut scroller = egui::ScrollArea::vertical()
            .id_source("scroller_".to_owned() + &row_height.to_string())
            .auto_shrink([false; 2])
            // .enable_scrolling(false)
            // .hscroll(self.show_type == ShowType::List)
            .id_source("content_scroll");

        if self.scroll_top {
            self.scroll_top = false;
            scroller = scroller.scroll_offset(egui::Vec2::ZERO);
        }

        let (current_scroll, max_scroll) = scroller
            .show_rows(ui, row_height, num_rows, |ui, row_range| {
                // tracing::info!("row_range: {:?}", row_range);
                match self.show_type {
                    ShowType::List => self.render_list(ui, row_range),
                    ShowType::Thumb => self.render_thumb(ui, row_range, num_cols),
                }
                let margin = ui.visuals().clip_rect_margin;
                let current_scroll = ui.clip_rect().top() - ui.min_rect().top() + margin;
                let max_scroll = ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin;
                (current_scroll, max_scroll)
            })
            .inner;

        // tracing::info!(
        //     "current_scroll: {}, max_scroll: {}",
        //     current_scroll,
        //     max_scroll
        // );

        if self.next_query.is_some() && current_scroll >= max_scroll && !self.loading_more {
            self.loading_more = true;
            self.load_more(ui.ctx());
        }
    }

    fn render_list(&mut self, ui: &mut egui::Ui, row_range: std::ops::Range<usize>) {
        for i in row_range {
            let data = self.list.get(i).unwrap();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::Frame::none().show(ui, |ui| {
                    ui.set_width(120.);
                    ui.label(&data.last_modified);
                });
                egui::Frame::none().show(ui, |ui| {
                    ui.set_width(60.);
                    ui.label(&data.size);
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.vertical(|ui| {
                        if ui
                            .add(
                                egui::Label::new(text_ellipsis(&data.name, 1))
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text(&data.url)
                            .clicked()
                        {
                            self.current_img = data.clone();
                            self.preview_modal.open();
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });
        }
    }

    fn render_thumb(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        num_cols: usize,
    ) {
        egui::Grid::new(format!("grid"))
            .num_columns(num_cols)
            .max_col_width(THUMB_LIST_WIDTH)
            .min_col_width(THUMB_LIST_WIDTH)
            .min_row_height(THUMB_LIST_HEIGHT)
            .spacing(egui::Vec2::ZERO)
            .start_row(row_range.start)
            .show(ui, |ui| {
                for i in row_range {
                    for j in 0..num_cols {
                        if let Some(d) = self.list.get(j + i * num_cols) {
                            let url = d.url.clone();
                            let resp = item_ui(ui, d.clone(), &mut self.images);
                            if resp.on_hover_text(url).clicked() {
                                self.current_img = d.clone();
                                self.preview_modal.open();
                                ui.ctx().request_repaint();
                            }
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui) {
        if ui.button("Upload file...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_path = Some(path.display().to_string());
            }
        }
        ui.horizontal(|ui| {
            ui.set_width(25.0);
            let enabled = self.state != State::Busy(Route::List)
                && self.state != State::Busy(Route::Upload)
                && !self.loading_more;

            ui.add_enabled_ui(enabled, |ui| {
                if ui.button("\u{1f503}").clicked() {
                    self.scroll_top = true;
                    let query = Self::build_query(self.oss.path.clone());
                    self.next_query = Some(query);
                    self.list = vec![];
                    self.get_list(ui.ctx());
                }
            });
        });
        ui.label(format!("List({})", self.list.len()));
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
                if ui
                    .selectable_value(&mut self.show_type, ShowType::Thumb, "\u{25a3}")
                    .clicked()
                {
                    self.scroll_top = true;
                }
                if ui
                    .selectable_value(&mut self.show_type, ShowType::List, "\u{2630}")
                    .clicked()
                {
                    self.scroll_top = true;
                }
            });
        });
    }
    fn status_bar_contents(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        if self.loading_more {
            ui.add(egui::Spinner::new().size(12.0));
        }

        if self.next_query.is_none() && !self.loading_more {
            ui.label("No More Data.");
        }

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
                        if let Some(img) = self.images.get(&current_img.url) {
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
        self.images.poll();
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
                        self.next_query = str.next_query();
                        self.set_list(str);
                        self.loading_more = false;
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
                        self.bar_contents(ui);
                    });
                });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.status_bar_contents(ui);
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
                            Route::List => self.render_content(ui),
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
    }
}
