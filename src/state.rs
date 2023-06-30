use crate::global;
use crate::widgets::toasts::Toasts;
use crate::widgets::{
    confirm::{Confirm, ConfirmAction},
    file_view_ui, log_panel_ui, transfer_panel_ui,
};
use crate::{spawn_evs, spawn_transfer};
use cc_core::{log::LogItem, store, tracing, MemoryHistory, Session, Setting};
use cc_files::Cache as ImageCache;
use cc_storage::util::get_name_form_path;
use cc_storage::{
    Bucket, Client, ListObjects, Metadata, Object, Params, Result as ClientResult, TransferManager,
};
use std::{path::PathBuf, vec};

const MAX_BUFFER_SIZE: u64 = 2 * 1024 * 1024;

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

pub enum FileAction {
    Copy(String),
    Move(String),
}

pub enum Update {
    TransferResult,
    List(ClientResult<ListObjects>),
    Navgator(NavgatorType),
    Deleted(ClientResult<()>),
    CreateFolder(ClientResult<()>),
    ViewObject(Object),
    HeadObject(ClientResult<Metadata>),
    GetObject(ClientResult<(String, Vec<u8>)>),
    BucketInfo(ClientResult<Bucket>),
    Copied(ClientResult<(String, bool)>),
    DownloadObject(String),
    SignatureUrl(ClientResult<String>),
    Confirm((String, ConfirmAction)),
    Prompt((String, ConfirmAction)),
}

pub struct State {
    pub client: Option<Client>,
    pub list: Vec<Object>,
    pub current_object: Object,
    pub confirm_rx: crossbeam_channel::Receiver<ConfirmAction>,
    pub setting: Setting,
    pub is_preview: bool,
    pub img_zoom: f32,
    pub img_default_zoom: f32,
    pub img_zoom_offset: egui::Vec2,
    pub loading_more: bool,
    pub next_query: Option<Params>,
    pub scroll_top: bool,
    pub file_cache: ImageCache,
    pub logs: Vec<LogItem>,
    pub is_show_result: bool,
    pub current_path: String,
    pub navigator: MemoryHistory,
    confirm: Confirm,
    pub session: Session,
    pub sessions: Vec<Session>,
    pub err: Option<String>,
    pub dropped_files: Vec<egui::DroppedFile>,
    pub picked_path: Vec<PathBuf>,
    pub status: Status,
    pub toasts: Toasts,
    pub filter_str: String,
    pub selected_item: usize,
    pub ctx: egui::Context,
    pub bucket: Option<Bucket>,
    pub file_action: Option<FileAction>,
    pub transfer_manager: TransferManager,
}

impl State {
    pub fn new(ctx: &egui::Context) -> Self {
        let session = match store::get_latest_session() {
            Some(session) => session,
            None => Session::default(),
        };

        let mut client = None;
        let mut bucket = None;

        let (confirm_tx, confirm_rx) = crossbeam_channel::bounded(1);

        let mut current_path = String::from("");
        let navigator = MemoryHistory::new();
        let ctx_clone = ctx.clone();

        let images = ImageCache::create(move || ctx_clone.request_repaint());

        let mut status = Status::Idle(Route::List);

        let setting = Setting::load();

        let is_need_init = !session.is_empty() && setting.auto_login;

        if is_need_init {
            match Client::builder()
                .service(&session.service)
                .endpoint(&session.endpoint)
                .access_key(&session.key_id)
                .access_secret(&session.key_secret)
                .bucket(&session.bucket)
                .build()
            {
                Ok(cli) => {
                    bucket = Some(Bucket::default());
                    current_path = "".to_string();
                    client = Some(cli);
                    navigator.push(current_path.clone());
                }
                Err(err) => tracing::error!("{:?}", err),
            }
        } else {
            status = Status::Idle(Route::Auth);
        }

        let mut this = Self {
            setting,
            client,
            current_object: Object::default(),
            list: vec![],
            confirm_rx,
            err: None,
            is_preview: false,
            img_zoom: 1.0,
            img_default_zoom: 1.0,
            img_zoom_offset: Default::default(),
            loading_more: false,
            next_query: None,
            scroll_top: false,
            file_cache: images,
            logs: vec![],
            is_show_result: false,
            current_path,
            navigator,
            confirm: Confirm::new(confirm_tx),
            session,
            sessions: vec![],
            dropped_files: vec![],
            picked_path: vec![],
            status,
            toasts: Toasts::new(),
            filter_str: String::new(),
            selected_item: 0,
            ctx: ctx.clone(),
            bucket,
            file_action: None,
            transfer_manager: TransferManager::new(),
        };

        this.next_query = Some(this.build_query(None));
        this.sessions = this.load_all_session();

        if is_need_init {
            this.get_bucket_info();
        }

        this
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        self.file_cache.poll();
        self.init_confirm(ctx);
        let ctx_clone = ctx.clone();
        self.transfer_manager
            .poll(move || ctx_clone.request_repaint());
        self.selected_item = self.list.iter().filter(|x| x.selected).count();
        while let Ok(update) = global().update_rx.try_recv() {
            match update {
                Update::TransferResult => {
                    self.refresh();
                }
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
                        self.refresh();
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.logs
                            .push(LogItem::delete().with_error(err.to_string()));
                    }
                },
                Update::CreateFolder(result) => match result {
                    Ok(_) => {
                        //
                        self.refresh();
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
                Update::ViewObject(obj) => {
                    self.head_object(obj.key());
                    self.current_object = obj;
                    self.is_preview = true;
                }
                Update::GetObject(result) => match result {
                    Ok((_name, data)) => {
                        self.file_cache.add(self.current_object.key(), data);
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
                Update::HeadObject(result) => match result {
                    Ok(headers) => {
                        tracing::debug!(
                            "current: {:?} {}",
                            self.current_object,
                            headers.content_length()
                        );
                        if let Some(mint_type) = headers.content_type() {
                            self.current_object.set_mine_type(mint_type.to_string());
                        }
                        if headers.content_length() <= MAX_BUFFER_SIZE {
                            self.get_current_object();
                        } else {
                            self.file_cache.big_file(self.current_object.key());
                        }
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
                Update::BucketInfo(result) => match result {
                    Ok(bucket) => {
                        self.bucket = Some(bucket);
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.err = Some(err.to_string());
                    }
                },
                Update::Copied(result) => match result {
                    Ok((file, is_move)) => {
                        self.file_action = None;
                        if is_move {
                            self.delete_object(file);
                        }
                        self.refresh();
                    }
                    Err(err) => {
                        self.status = Status::Idle(Route::List);
                        self.toasts.error("Copy failed.");
                        self.logs.push(LogItem::copy().with_error(err.to_string()));
                    }
                },
                Update::DownloadObject(name) => {
                    self.download_file(name);
                }
                Update::SignatureUrl(result) => match result {
                    Ok(url) => {
                        self.current_object.set_url(url.clone());
                    }
                    Err(err) => {
                        self.toasts.error("Signature Url failed.");
                        self.logs.push(LogItem::copy().with_error(err.to_string()));
                    }
                },
                Update::Prompt((message, action)) => self.confirm.prompt(message, action),
                Update::Confirm((message, action)) => self.confirm.show(message, action),
            }
        }

        if !self.dropped_files.is_empty() {
            let mut files = vec![];
            let dropped_files = self.dropped_files.clone();
            self.dropped_files = vec![];
            for file in dropped_files {
                if let Some(path) = &file.path {
                    files.push(path.clone());
                }
            }

            self.picked_path = files;
        }

        self.upload_file();

        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            self.dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        }

        if self.client.is_some() {
            file_view_ui(ctx, self);
            log_panel_ui(ctx, self);
            transfer_panel_ui(ctx, self);
        }

        self.toasts.show(ctx);
    }

    pub fn update_tx(&self) -> crossbeam_channel::Sender<Update> {
        global().update_tx.clone()
    }

    pub fn client(&self) -> &Client {
        self.client.as_ref().expect("Oss not initialized yet")
    }

    pub fn bucket(&self) -> &Bucket {
        self.bucket.as_ref().expect("Bucket not initialized yet")
    }

    pub fn bucket_is_private(&self) -> bool {
        self.bucket().is_private()
    }

    pub fn get_signature_url(&self, name: String, expire: u64) {
        spawn_evs!(self, |evs, client, ctx| {
            let res = client.signature_url(&name, expire, None).await;
            evs.send(Update::SignatureUrl(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn upload_file(&mut self) {
        if self.picked_path.is_empty() {
            return;
        }
        let picked_path = self.picked_path.clone();
        self.picked_path = vec![];

        let dest = self.current_path.clone();
        self.transfer_manager.show("upload");

        spawn_transfer!(self, |transfer, evs, client, ctx| {
            let _ = client.put_multi(picked_path, dest, transfer).await;
            evs.send(Update::TransferResult).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn delete_object(&mut self, file: String) {
        self.status = Status::Busy(Route::List);

        spawn_evs!(self, |evs, client, ctx| {
            let res = client.delete_object(file).await;
            evs.send(Update::Deleted(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn delete_multi_object(&mut self) {
        self.status = Status::Busy(Route::List);

        let files: Vec<Object> = self.list.iter().filter(|x| x.selected).cloned().collect();

        spawn_evs!(self, |evs, client, ctx| {
            let res = client.delete_multi_object(files).await;
            evs.send(Update::Deleted(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn create_folder(&mut self, name: String) {
        self.status = Status::Busy(Route::List);

        let name = format!("{}{}", self.current_path, name);

        spawn_evs!(self, |evs, client, ctx| {
            let res = client.create_folder(name).await;
            evs.send(Update::CreateFolder(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn head_object(&mut self, name: &str) {
        // self.status = Status::Busy(Route::List);

        // let name = format!("{}{}", self.current_path, name);
        let name = name.to_string();

        spawn_evs!(self, |evs, client, ctx| {
            let res = client.head_object(name).await;
            evs.send(Update::HeadObject(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn copy_object(&mut self, src: String, dest: String, is_move: bool) {
        self.status = Status::Busy(Route::List);

        spawn_evs!(self, |evs, client, ctx| {
            let res = client.copy_object(src, dest, is_move).await;
            evs.send(Update::Copied(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn get_current_object(&mut self) {
        let name = self.current_object.key().to_string();

        spawn_evs!(self, |evs, client, ctx| {
            let res = client.get_object(name).await;
            evs.send(Update::GetObject(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn get_bucket_info(&mut self) {
        spawn_evs!(self, |evs, client, ctx| {
            let res = client.get_bucket_info().await;
            evs.send(Update::BucketInfo(res)).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn get_list(&mut self) {
        if let Some(_query) = &self.next_query {
            if !self.loading_more {
                self.status = Status::Busy(Route::List);
            }

            let query = self.current_path.clone();

            spawn_evs!(self, |evs, client, ctx| {
                let res = client.list_v2(Some(query)).await;
                evs.send(Update::List(res)).unwrap();
                ctx.request_repaint();
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
            path.push('/');
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

    pub fn login(&mut self) -> ClientResult<()> {
        tracing::debug!("Login with session: {:?}", self.session);
        let client = Client::builder()
            .endpoint(&self.session.endpoint)
            .access_key(&self.session.key_id)
            .access_secret(&self.session.key_secret)
            .bucket(&self.session.bucket)
            .build()?;

        store::put_session(&self.session)?;
        let current_path = "".to_string();
        self.current_path = current_path.clone();
        self.navigator.push(current_path);
        self.client = Some(client);
        self.get_bucket_info();
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
                    self.client = None;
                    self.current_path = String::from("");
                    self.navigator.clear();
                    self.setting.auto_login = false;
                }
                ConfirmAction::RemoveSession(session) => {
                    store::delete_session_by_name(&session.key_id);
                    self.sessions = self.load_all_session();
                }
                ConfirmAction::RemoveFile(obj) => {
                    self.delete_object(obj.key().to_string());
                }
                ConfirmAction::CreateFolder(name) => {
                    self.create_folder(name);
                }
                ConfirmAction::RemoveFiles => {
                    self.delete_multi_object();
                }
                ConfirmAction::GenerateUrl(expire) => {
                    let name = self.current_object.key();
                    self.get_signature_url(name.to_string(), expire);
                }
                ConfirmAction::RenameObject((src, name)) => {
                    let dest = format!("{}{}", self.current_path, name);
                    self.copy_object(src, dest, true);
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

    pub fn download_file(&mut self, name: String) {
        let file_name = get_name_form_path(&name);
        self.transfer_manager.show("download");
        if let Some(path) = rfd::FileDialog::new().set_file_name(&file_name).save_file() {
            spawn_transfer!(self, |transfer, evs, client, ctx| {
                let _ = client.download_file(&name, path, transfer).await;
                evs.send(Update::TransferResult).unwrap();
                ctx.request_repaint();
            });
        }
    }

    pub(crate) fn close_preview(&mut self) {
        self.is_preview = false;
        self.current_object = Object::default();
    }
}
