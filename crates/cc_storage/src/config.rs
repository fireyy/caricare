#![allow(dead_code)]
use std::{fmt::Debug, time::Duration};

#[derive(Debug)]
pub(crate) struct HttpTimeout {
    pub(crate) connect: Duration,
    pub(crate) read_write: Duration,
    pub(crate) header: Duration,
    pub(crate) long: Duration,
    pub(crate) idle_conn: Duration,
}

#[derive(Debug)]
pub(crate) struct HttpMaxConns {
    pub(crate) max_idle_conns: usize,
    pub(crate) max_idle_conns_per_host: usize,
}

#[derive(Debug)]
pub(crate) struct HttpProxy {
    pub(crate) host: String,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ClientConfig {
    pub(crate) endpoint: String,
    pub(crate) access_key_id: String,
    pub(crate) access_key_secret: String,
    pub(crate) bucket: String,
    pub(crate) retries: u32,
    pub(crate) debug: bool,
    pub(crate) timeout: Duration,
    pub(crate) security_token: String,
    pub(crate) cname: bool,
    pub(crate) http_timeout: Option<HttpTimeout>,
    pub(crate) http_max_conns: Option<HttpMaxConns>,
    pub(crate) http_proxy: Option<HttpProxy>,
    pub(crate) enable_md5: bool,
    pub(crate) md5_threshold: i64, // bytes
    pub(crate) enable_crc: bool,   //TODO: turn on CRC data check
    pub(crate) log_level: i8,
    pub(crate) upload_limit_speed: i64,
    //...
    pub(crate) additional_headers: Vec<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            endpoint: Default::default(),
            access_key_id: Default::default(),
            access_key_secret: Default::default(),
            bucket: Default::default(),
            retries: Default::default(),
            debug: Default::default(),
            timeout: Duration::from_secs(60),
            security_token: Default::default(),
            cname: Default::default(),
            http_timeout: Default::default(),
            http_max_conns: Default::default(),
            http_proxy: Default::default(),
            enable_md5: Default::default(),
            md5_threshold: Default::default(),
            enable_crc: Default::default(),
            log_level: Default::default(),
            upload_limit_speed: Default::default(),
            additional_headers: Default::default(),
        }
    }
}
