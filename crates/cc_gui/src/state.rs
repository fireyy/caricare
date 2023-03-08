use crate::theme;
use crate::widgets::{
    confirm::{Confirm, ConfirmAction},
    image_view_ui, log_panel_ui,
};
use crate::SUPPORT_EXTENSIONS;
use cc_core::{
    log::LogItem, store, tokio, tracing, util::get_extension, CoreError, ImageCache, ImageFetcher,
    MemoryHistory, OssClient, Session, Setting,
};
use cc_oss::{
    errors::Error as OssError,
    object::{ListObjects, Object as OssObject},
    query::Query,
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
    Uploaded(Result<Vec<LogItem>, OssError>),
    List(Result<ListObjects, OssError>),
    Navgator(NavgatorType),
    Deleted(Result<(), OssError>),
    CreateFolder(Result<(), OssError>),
}

pub struct State {
    pub oss: Option<OssClient>,
    pub list: Vec<OssObject>,
    pub current_img: OssObject,
    pub update_tx: mpsc::SyncSender<Update>,
    pub update_rx: mpsc::Receiver<Update>,
    pub confirm_rx: mpsc::Receiver<ConfirmAction>,
    pub setting: Setting,
    pub is_preview: bool,
    pub img_zoom: f32,
    pub offset: egui::Vec2,
    pub loading_more: bool,
    pub next_query: Option<Query>,
    pub scroll_top: bool,
    pub images: ImageCache,
    pub logs: Vec<LogItem>,
    pub is_show_result: bool,
    pub current_path: String,
    pub navigator: MemoryHistory,
    pub confirm: Confirm,
    pub session: Session,
    pub sessions: Vec<Session>,
    pub err: Option<String>,
    pub dropped_files: Vec<egui::DroppedFile>,
    pub picked_path: Vec<PathBuf>,
    pub status: Status,
    pub toasts: Toasts,
    pub cc_ui: theme::CCUi,
    pub filter_str: String,
}

impl State {
    pub fn new(ctx: &egui::Context) -> Self {
        let cc_ui = theme::CCUi::load_and_apply(ctx);
        let session = match store::get_latest_session() {
            Some(session) => session,
            None => Session::default(),
        };

        let mut oss = None;

        let (update_tx, update_rx) = mpsc::sync_channel(1);
        let (confirm_tx, confirm_rx) = mpsc::sync_channel(1);

        let mut current_path = String::from("");
        let navigator = MemoryHistory::new();

        let images = ImageCache::new(ImageFetcher::spawn(ctx.clone()));

        let mut status = Status::Idle(Route::List);

        let setting = Setting::load();

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

        let mut this = Self {
            setting,
            oss,
            current_img: OssObject::default(),
            list: vec![],
            update_tx,
            update_rx,
            confirm_rx,
            err: None,
            is_preview: false,
            img_zoom: 1.0,
            offset: Default::default(),
            loading_more: false,
            next_query: None,
            scroll_top: false,
            images,
            logs: vec![],
            is_show_result: false,
            current_path: current_path.clone(),
            navigator,
            confirm: Confirm::new(confirm_tx),
            session,
            sessions: vec![],
            dropped_files: vec![],
            picked_path: vec![],
            status,
            toasts: Toasts::new(),
            cc_ui,
            filter_str: String::new(),
        };

        this.next_query = Some(this.build_query(None));
        this.sessions = this.load_all_session();

        this
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        self.images.poll();
        self.init_confirm(ctx);
        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                Update::Uploaded(result) => match result {
                    Ok(mut str) => {
                        self.status = Status::Idle(Route::List);
                        self.logs.append(&mut str);
                        self.is_show_result = true;
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::Upload);
                        self.err = Some(err.message());
                    }
                },
                Update::List(result) => match result {
                    Ok(str) => {
                        if let Some(token) = str.next_continuation_token() {
                            self.next_query = Some(self.build_query(Some(token.clone())));
                        } else {
                            self.next_query = None;
                        }
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
                Update::Deleted(result) => match result {
                    Ok(_) => {
                        //
                        self.toasts.success("Delete Successed");
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
                Update::CreateFolder(result) => match result {
                    Ok(_) => {
                        //
                        self.toasts.success("Create Successed");
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
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
            log_panel_ui(ctx, self);
        }

        self.toasts.show(ctx);
    }

    pub fn oss(&self) -> &OssClient {
        self.oss.as_ref().expect("Oss not initialized yet")
    }

    pub fn get_oss_url(&self, path: &str) -> String {
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

    pub fn delete_object(&mut self, ctx: &egui::Context, file: OssObject) {
        self.status = Status::Busy(Route::List);

        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();
        let oss = self.oss().clone();

        cc_core::runtime::spawn(async move {
            tokio::spawn(async move {
                let res = oss.delete_object(file).await;
                update_tx.send(Update::Deleted(res)).unwrap();
                ctx.request_repaint();
            });
        });
    }

    pub fn create_folder(&mut self, ctx: &egui::Context, name: String) {
        self.status = Status::Busy(Route::List);

        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();
        let oss = self.oss().clone();

        cc_core::runtime::spawn(async move {
            tokio::spawn(async move {
                let res = oss.create_object(name).await;
                update_tx.send(Update::CreateFolder(res)).unwrap();
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

    pub fn set_list(&mut self, obj: ListObjects) {
        let mut dirs = obj.common_prefixes;
        let mut files: Vec<OssObject> = obj
            .objects
            .into_iter()
            .filter(|x| !x.key().ends_with('/'))
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
        self.current_path = current_path;
        self.next_query = Some(self.build_query(None));
        self.list = vec![];
        self.get_list(ctx);
    }

    pub fn filter(&mut self, ctx: &egui::Context) {
        self.refresh(ctx);
    }

    fn build_query(&self, next_token: Option<String>) -> Query {
        let mut path = self.current_path.clone();
        if !path.ends_with('/') && !path.is_empty() {
            path.push_str("/");
        }
        if !self.filter_str.is_empty() {
            path.push_str(&self.filter_str);
        }
        let mut query = Query::new();
        query.insert("prefix", path);
        query.insert("delimiter", "/");
        query.insert("max-keys", self.setting.page_limit);
        if let Some(token) = next_token {
            query.insert("continuation-token", token);
        }
        query
    }

    pub fn save_auth(&mut self, ctx: &egui::Context) -> Result<(), CoreError> {
        let client = OssClient::new(&self.session)?;
        let _ = store::put_session(&self.session)?;
        let current_path = client.get_path().to_string();
        self.current_path = current_path.clone();
        self.navigator.push(current_path);
        self.oss = Some(client);
        self.refresh(ctx);
        self.setting.auto_login = true;
        self.sessions = self.load_all_session();
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
                    self.sessions = self.load_all_session();
                }
                ConfirmAction::RemoveFile(obj) => {
                    self.delete_object(ctx, obj);
                }
            }
        }
    }

    pub fn confirm(&mut self, message: impl Into<String>, action: ConfirmAction) {
        self.confirm.show(message, action);
    }

    pub fn load_all_session(&mut self) -> Vec<Session> {
        let mut sessions = vec![];
        match store::get_all_session() {
            Ok(list) => {
                sessions = list;
            }
            Err(err) => tracing::debug!("{:?}", err),
        }

        sessions
    }
}
