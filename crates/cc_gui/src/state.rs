use crate::widgets::{
    confirm::{Confirm, ConfirmAction},
    image_view_ui, result_view_ui,
};
use crate::SUPPORT_EXTENSIONS;
use cc_core::{
    store, tokio, tracing, util::get_extension, CoreError, ImageCache, ImageFetcher, MemoryHistory,
    OssBucket, OssClient, OssError, OssObject, Query, Session, Setting, UploadResult,
};
use egui_notify::Toasts;
use std::{path::PathBuf, sync::mpsc, vec};

#[derive(PartialEq)]
pub enum Status {
    Idle(Route),
    Busy(Route),
}

impl Default for Status {
    fn default() -> Self {
        Self::Idle(Route::default())
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Route {
    Upload,
    List,
    #[default]
    Auth,
}

pub enum NavgatorType {
    Back,
    Forward,
    New(String),
}

pub enum Update {
    Uploaded(Result<Vec<UploadResult>, OssError>),
    List(Result<OssBucket, OssError>),
    Navgator(NavgatorType),
}

#[derive(serde::Serialize)]
#[serde(default)]
pub struct State {
    #[serde(skip)]
    pub oss: Option<OssClient>,
    pub list: Vec<OssObject>,
    pub current_img: OssObject,
    #[serde(skip)]
    pub update_tx: mpsc::SyncSender<Update>,
    #[serde(skip)]
    pub update_rx: mpsc::Receiver<Update>,
    #[serde(skip)]
    pub confirm_rx: mpsc::Receiver<ConfirmAction>,
    pub setting: Setting,
    pub is_preview: bool,
    pub loading_more: bool,
    #[serde(skip)]
    pub next_query: Option<Query>,
    pub scroll_top: bool,
    #[serde(skip)]
    pub images: ImageCache,
    #[serde(skip)]
    pub upload_result: Vec<UploadResult>,
    pub is_show_result: bool,
    pub current_path: String,
    pub navigator: MemoryHistory,
    #[serde(skip)]
    pub confirm: Confirm,
    pub session: Session,
    pub sessions: Vec<Session>,
    pub err: Option<String>,
    #[serde(skip)]
    pub dropped_files: Vec<egui::DroppedFile>,
    #[serde(skip)]
    pub picked_path: Vec<PathBuf>,
    #[serde(skip)]
    pub status: Status,
    #[serde(skip)]
    pub toasts: Toasts,
}

impl State {
    pub fn new(ctx: &egui::Context) -> Self {
        let session = match store::get_latest_session() {
            Some(session) => session,
            None => Session::default(),
        };
        let mut sessions = vec![];
        match store::get_all_session() {
            Ok(list) => {
                sessions = list;
            }
            Err(err) => tracing::debug!("{:?}", err),
        }
        let mut oss = None;

        let (update_tx, update_rx) = mpsc::sync_channel(1);
        let (confirm_tx, confirm_rx) = mpsc::sync_channel(1);

        let mut current_path = String::from("");
        let navigator = MemoryHistory::new();

        let images = ImageCache::new(ImageFetcher::spawn(ctx.clone()));

        let mut status = Status::Idle(Route::List);

        let setting = Setting::load();
        let limit = setting.page_limit;

        if !session.is_empty() && setting.auto_login {
            match OssClient::new(&session) {
                Ok(client) => {
                    current_path = client.get_path().to_string();
                    oss = Some(client);
                    navigator.push(current_path.clone());
                }
                Err(err) => tracing::error!("{:?}", err),
            }
        } else {
            status = Status::Idle(Route::Auth);
        }

        Self {
            setting,
            oss,
            current_img: OssObject::default(),
            list: vec![],
            update_tx,
            update_rx,
            confirm_rx,
            err: None,
            is_preview: false,
            loading_more: false,
            next_query: Some(build_query(current_path.clone(), limit)),
            scroll_top: false,
            images,
            upload_result: vec![],
            is_show_result: false,
            current_path: current_path.clone(),
            navigator,
            confirm: Confirm::new(confirm_tx),
            session,
            sessions,
            dropped_files: vec![],
            picked_path: vec![],
            status,
            toasts: Toasts::new(),
        }
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        self.images.poll();
        self.init_confirm(ctx);
        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                Update::Uploaded(result) => match result {
                    Ok(str) => {
                        self.status = Status::Idle(Route::List);
                        self.upload_result = str;
                        self.is_show_result = true;
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::Upload);
                        self.err = Some(err.message());
                    }
                },
                Update::List(result) => match result {
                    Ok(str) => {
                        self.next_query = str.next_query();
                        self.set_list(str);
                        self.loading_more = false;
                        self.status = Status::Idle(Route::List);
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.message());
                    }
                },
                Update::Navgator(nav) => {
                    match nav {
                        NavgatorType::Back => {
                            if self.navigator.can_go_back() {
                                self.navigator.go(-1);
                            }
                        }
                        NavgatorType::Forward => {
                            if self.navigator.can_go_forward() {
                                self.navigator.go(1);
                            }
                        }
                        NavgatorType::New(path) => {
                            self.navigator.push(path);
                        }
                    }
                    self.current_path = self.navigator.location();
                    self.refresh(ctx);
                }
            }
        }

        if !self.dropped_files.is_empty() {
            let mut files = vec![];
            let dropped_files = self.dropped_files.clone();
            self.dropped_files = vec![];
            for file in dropped_files {
                if let Some(path) = &file.path {
                    if SUPPORT_EXTENSIONS.contains(&get_extension(path.clone()).as_str()) {
                        files.push(path.clone());
                    }
                }
            }

            self.picked_path = files;
        }

        self.upload_file(ctx);

        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            self.dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        }

        if self.oss.is_some() {
            image_view_ui(ctx, self);
            result_view_ui(ctx, self);
        }

        self.toasts.show(ctx);
    }

    pub fn oss(&self) -> &OssClient {
        self.oss.as_ref().expect("Oss not initialized yet")
    }

    pub fn get_oss_url(&self, path: &String) -> String {
        self.oss().get_file_url(path)
    }

    pub fn upload_file(&mut self, ctx: &egui::Context) {
        if self.picked_path.is_empty() {
            return;
        }
        let picked_path = self.picked_path.clone();
        self.picked_path = vec![];
        self.status = Status::Busy(Route::Upload);

        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();
        let oss = self.oss().clone();

        cc_core::runtime::spawn(async move {
            tokio::spawn(async move {
                let res = oss.put_multi(picked_path).await;
                update_tx.send(Update::Uploaded(res)).unwrap();
                ctx.request_repaint();
            });
        });
    }

    pub fn get_list(&mut self, ctx: &egui::Context) {
        if let Some(query) = &self.next_query {
            if !self.loading_more {
                self.status = Status::Busy(Route::List);
            }

            let update_tx = self.update_tx.clone();
            let ctx = ctx.clone();
            let query = query.clone();
            let oss = self.oss().clone();

            cc_core::runtime::spawn(async move {
                tokio::spawn(async move {
                    let res = oss.get_list(query).await;
                    update_tx.send(Update::List(res)).unwrap();
                    ctx.request_repaint();
                });
            });
        }
    }

    pub fn set_list(&mut self, obj: OssBucket) {
        let mut dirs = obj.common_prefixes;
        let mut files: Vec<OssObject> = obj
            .files
            .into_iter()
            .filter(|x| !x.path.ends_with('/'))
            .collect();
        self.list.append(&mut dirs);
        self.list.append(&mut files);
    }

    pub fn load_more(&mut self, ctx: &egui::Context) {
        tracing::info!("load more!");
        if self.next_query.is_some() {
            self.get_list(ctx);
        } else {
            //no more
        }
    }

    pub fn refresh(&mut self, ctx: &egui::Context) {
        self.err = None;
        self.scroll_top = true;
        let current_path = self.navigator.location();
        self.next_query = Some(build_query(current_path, self.setting.page_limit));
        self.list = vec![];
        self.get_list(ctx);
    }

    pub fn save_auth(&mut self, ctx: &egui::Context) -> Result<(), CoreError> {
        let client = OssClient::new(&self.session)?;
        let _ = store::put_session(&self.session)?;
        let current_path = client.get_path().to_string();
        self.current_path = current_path.clone();
        self.navigator.push(current_path);
        self.oss = Some(client);
        self.refresh(ctx);
        self.setting.auto_login = false;
        Ok(())
    }

    pub fn init_confirm(&mut self, ctx: &egui::Context) {
        self.confirm.init(ctx);
        while let Ok(action) = self.confirm_rx.try_recv() {
            match action {
                ConfirmAction::Logout => {
                    self.status = Status::Idle(Route::Auth);
                    self.oss = None;
                    self.current_path = String::from("");
                    self.navigator.clear();
                    self.setting.auto_login = false;
                }
                ConfirmAction::RemoveSession(session) => {
                    store::delete_session_by_name(&session.key_id);
                }
            }
        }
    }

    pub fn confirm(&mut self, message: impl Into<String>, action: ConfirmAction) {
        self.confirm.show(message, action);
    }
}

fn build_query(path: String, limit: u16) -> Query {
    let mut path = path.clone();
    if !path.ends_with('/') && !path.is_empty() {
        path.push_str("/");
    }
    let mut query = Query::new();
    query.insert("prefix", path);
    query.insert("delimiter", "/");
    query.insert("max-keys", limit);
    query
}
