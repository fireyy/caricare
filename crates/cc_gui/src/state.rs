use crate::spawn_evs;
use crate::theme;
use crate::widgets::{
    confirm::{Confirm, ConfirmAction},
    image_view_ui, log_panel_ui,
};
use crate::SUPPORT_EXTENSIONS;
use cc_core::{log::LogItem, store, tracing, util::get_extension, MemoryHistory, Session, Setting};
use cc_images::Cache as ImageCache;
use egui_notify::Toasts;
use oss_sdk::{Client as OssClient, HeaderMap, ListObjects, Object, Params, Result as OssResult};
use std::{path::PathBuf, sync::mpsc, vec};
// ImageCache, ImageFetcher,

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
    Uploaded(OssResult<Vec<String>>),
    List(OssResult<ListObjects>),
    Navgator(NavgatorType),
    Deleted(OssResult<()>),
    CreateFolder(OssResult<()>),
    ViewObject(Object),
    HeadObject(OssResult<HeaderMap>),
    GetObject(OssResult<Vec<u8>>),
}

pub struct State {
    pub oss: Option<OssClient>,
    pub list: Vec<Object>,
    pub current_img: Object,
    pub update_tx: mpsc::SyncSender<Update>,
    pub update_rx: mpsc::Receiver<Update>,
    pub confirm_rx: mpsc::Receiver<ConfirmAction>,
    pub setting: Setting,
    pub is_preview: bool,
    pub img_zoom: f32,
    pub offset: egui::Vec2,
    pub loading_more: bool,
    pub next_query: Option<Params>,
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
    pub selected_item: usize,
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
        let ctx_clone = ctx.clone();

        let images = ImageCache::create(move || ctx_clone.request_repaint());

        let mut status = Status::Idle(Route::List);

        let setting = Setting::load();

        if !session.is_empty() && setting.auto_login {
            match OssClient::builder()
                .endpoint(&session.endpoint)
                .access_key(&session.key_id)
                .access_secret(&session.key_secret)
                .bucket(&session.bucket)
                .build()
            {
                Ok(client) => {
                    current_path = "".to_string();
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
            current_img: Object::default(),
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
            selected_item: 0,
        };

        this.next_query = Some(this.build_query(None));
        this.sessions = this.load_all_session();

        this
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        self.images.poll();
        self.init_confirm(ctx);
        self.selected_item = self.list.iter().filter(|x| x.selected).count();
        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                Update::Uploaded(result) => match result {
                    Ok(str) => {
                        self.status = Status::Idle(Route::List);
                        for s in str {
                            self.logs.push(LogItem::upload().with_info(s));
                        }
                        self.refresh();
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::Upload);
                        self.err = Some(err.to_string());
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
                        self.err = Some(err.to_string());
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
                    self.refresh();
                }
                Update::Deleted(result) => match result {
                    Ok(_) => {
                        //
                        self.toasts.success("Delete Successed");
                        self.refresh();
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
                        self.refresh();
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
                Update::ViewObject(obj) => {
                    // self.current_img = obj;
                    self.get_object(obj.key());
                }
                Update::GetObject(result) => match result {
                    Ok(headers) => {
                        //TODO: show object
                        self.is_preview = true;
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
                Update::HeadObject(result) => match result {
                    Ok(headers) => {
                        let url = self.get_signature_url(self.current_img.key());
                        self.current_img.set_url(url);
                        if let Some(mint_type) = headers.get("content-type") {
                            self.current_img
                                .set_mine_type(mint_type.to_str().unwrap().to_string());
                        }
                        self.is_preview = true;
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
                    if SUPPORT_EXTENSIONS.contains(&get_extension(&path).as_str()) {
                        files.push(path.clone());
                    }
                }
            }

            self.picked_path = files;
        }

        self.upload_file();

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

    pub fn get_signature_url(&self, name: &str) -> String {
        self.oss().signature_url(name, None).unwrap()
    }

    pub fn get_thumb_url(&self, name: &str, size: u16) -> String {
        let mut params = Params::new();
        params.insert(
            "x-oss-process".into(),
            Some(format!("image/resize,w_{size}")),
        );
        self.oss().signature_url(name, Some(params)).unwrap()
    }

    pub fn upload_file(&mut self) {
        if self.picked_path.is_empty() {
            return;
        }
        let picked_path = self.picked_path.clone();
        self.picked_path = vec![];
        self.status = Status::Busy(Route::Upload);

        let dest = self.current_path.clone();

        spawn_evs!(self, |evs, client| {
            let res = client.put_multi(picked_path, dest).await;
            evs.send(Update::Uploaded(res)).unwrap();
        });
    }

    pub fn delete_object(&mut self, file: Object) {
        self.status = Status::Busy(Route::List);

        spawn_evs!(self, |evs, client| {
            let res = client.delete_object(file.key()).await;
            evs.send(Update::Deleted(res)).unwrap();
        });
    }

    pub fn delete_multi_object(&mut self) {
        self.status = Status::Busy(Route::List);

        let files: Vec<Object> = self.list.iter().filter(|x| x.selected).cloned().collect();

        spawn_evs!(self, |evs, client| {
            let res = client.delete_multi_object(files).await;
            evs.send(Update::Deleted(res)).unwrap();
        });
    }

    pub fn create_folder(&mut self, name: String) {
        self.status = Status::Busy(Route::List);

        let name = format!("{}{}", self.current_path, name);

        spawn_evs!(self, |evs, client| {
            let res = client.create_folder(name).await;
            evs.send(Update::CreateFolder(res)).unwrap();
        });
    }

    pub fn head_object(&mut self, name: &str) {
        // self.status = Status::Busy(Route::List);

        // let name = format!("{}{}", self.current_path, name);
        let name = name.to_string();

        spawn_evs!(self, |evs, client| {
            let res = client.head_object(name).await;
            evs.send(Update::HeadObject(res)).unwrap();
        });
    }

    pub fn get_object(&mut self, name: &str) {
        // self.status = Status::Busy(Route::List);

        // let name = format!("{}{}", self.current_path, name);
        let name = name.to_string();

        spawn_evs!(self, |evs, client| {
            let res = client.get_object(name).await;
            evs.send(Update::GetObject(res)).unwrap();
        });
    }

    pub fn get_list(&mut self) {
        if let Some(query) = &self.next_query {
            if !self.loading_more {
                self.status = Status::Busy(Route::List);
            }

            let query = query.clone();

            spawn_evs!(self, |evs, client| {
                let res = client.list_v2(Some(query)).await;
                evs.send(Update::List(res)).unwrap();
            });
        }
    }

    pub fn set_list(&mut self, obj: ListObjects) {
        let mut dirs = obj.common_prefixes;
        let mut files: Vec<Object> = obj
            .objects
            .into_iter()
            .filter(|x| !x.key().ends_with('/'))
            .collect();
        self.list.append(&mut dirs);
        self.list.append(&mut files);
    }

    pub fn load_more(&mut self) {
        tracing::debug!("load more!");
        if self.next_query.is_some() {
            self.get_list();
        } else {
            //no more
        }
    }

    pub fn refresh(&mut self) {
        self.err = None;
        self.scroll_top = true;
        let current_path = self.navigator.location();
        self.current_path = current_path;
        self.next_query = Some(self.build_query(None));
        self.list = vec![];
        self.get_list();
    }

    pub fn filter(&mut self) {
        self.refresh();
    }

    fn build_query(&self, next_token: Option<String>) -> Params {
        let mut path = self.current_path.clone();
        if !path.ends_with('/') && !path.is_empty() {
            path.push_str("/");
        }
        if !self.filter_str.is_empty() {
            path.push_str(&self.filter_str);
        }
        let mut query = Params::new();
        query.insert("list-type".into(), Some("2".to_string()));
        query.insert("prefix".into(), Some(path));
        query.insert("delimiter".into(), Some("/".into()));
        query.insert("max-keys".into(), Some(self.setting.page_limit.to_string()));
        if let Some(token) = next_token {
            query.insert("continuation-token".into(), Some(token));
        }
        query
    }

    pub fn login(&mut self) -> OssResult<()> {
        tracing::debug!("Login with session: {:?}", self.session);
        let client = OssClient::builder()
            .endpoint(&self.session.endpoint)
            .access_key(&self.session.key_id)
            .access_secret(&self.session.key_secret)
            .bucket(&self.session.bucket)
            .build()?;

        let _ = store::put_session(&self.session)?;
        let current_path = "".to_string();
        self.current_path = current_path.clone();
        self.navigator.push(current_path);
        self.oss = Some(client);
        self.refresh();
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
                    self.delete_object(obj);
                }
                ConfirmAction::CreateFolder(name) => {
                    self.create_folder(name);
                }
                ConfirmAction::RemoveFiles => {
                    self.delete_multi_object();
                }
            }
        }
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
