use cc_core::{tracing, OssError, OSS};
use std::sync::mpsc;

enum Update {
    Uploaded(Result<String, OssError>),
}

enum State {
    Idle(Route),
    Busy(Route),
}

#[derive(Clone, Copy, PartialEq)]
enum Route {
    Upload,
}

pub struct App {
    oss: OSS,
    update_tx: mpsc::SyncSender<Update>,
    update_rx: mpsc::Receiver<Update>,
    state: State,
    err: Option<String>,
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let oss = OSS::new();
        let (update_tx, update_rx) = mpsc::sync_channel(1);

        Self {
            oss,
            update_tx,
            update_rx,
            state: State::Idle(Route::Upload),
            err: None,
            dropped_files: vec![],
            picked_path: None,
        }
    }

    fn upload_file(&mut self, ctx: &egui::Context) {
        if let Some(picked_path) = self.picked_path.clone() {
            self.picked_path = None;
            self.state = State::Busy(Route::Upload);

            let update_tx = self.update_tx.clone();
            let ctx = ctx.clone();

            self.oss.put(picked_path, move |res| {
                update_tx.send(Update::Uploaded(res)).unwrap();
                ctx.request_repaint();
            });

            // let update_tx = self.update_tx.clone();
            // let ctx = ctx.clone();
            // let path = picked_path.clone();

            // thread::spawn(move || {
            //     let result = self.oss.put(path);
            //     update_tx.send(Update::Uploaded(result)).unwrap();
            //     ctx.request_repaint();
            // });
        }
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
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| match &mut self.state {
                State::Idle(ref mut route) => match route {
                    Route::Upload => {
                        ui.centered_and_justified(|ui| {
                            if ui.button("Open file...").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    self.picked_path = Some(path.display().to_string());
                                }
                            }
                        });
                        if let Some(err) = &self.err {
                            ui.label(err);
                        }
                    }
                },
                State::Busy(route) => match route {
                    Route::Upload => {
                        ui.spinner();
                        ui.heading("Uploading file...");
                    }
                },
            });
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
    }
}
