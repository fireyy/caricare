use std::sync::Arc;

use crate::config::ClientConfig;
use crate::conn::{Conn, UrlMaker};
use crate::util;
use crate::{bucket::Bucket, Result};

#[derive(Clone)]
pub struct Client {
    pub(crate) config: Arc<ClientConfig>,
    pub(crate) conn: Conn,
    client: reqwest::Client,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            config: Default::default(),
        }
    }

    fn new(config: ClientConfig) -> Result<Client> {
        let client = reqwest::Client::builder()
            .http1_only()
            .timeout(config.timeout)
            .build()?;
        /*
        reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap() */

        let um = UrlMaker::new(&config.endpoint, config.cname, config.http_proxy.is_some())?;
        let config = Arc::new(config);
        let conn = Conn::new(config.clone(), Arc::new(um), client.clone());

        Ok(Client {
            conn,
            config,
            client,
        })
    }

    pub fn bucket(&self, bucket: impl Into<String>) -> Result<Bucket> {
        let bucket: String = bucket.into();
        util::check_bucket_name(&bucket)?;
        Ok(Bucket::new(self.clone(), bucket))
    }
}

pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    pub fn access_key(mut self, key: impl Into<String>) -> Self {
        self.config.access_key_id = key.into();
        self
    }

    pub fn access_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.access_key_secret = secret.into();
        self
    }

    pub fn build(self) -> Result<Client> {
        Client::new(self.config)
    }
}
