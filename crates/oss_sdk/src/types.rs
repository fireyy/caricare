use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use reqwest::Response;

use crate::Result;

pub(crate) type Params = BTreeMap<String, Option<String>>;
pub(crate) type Headers = HashMap<String, String>;

pub(crate) struct Request {
    pub(crate) url: String,
    pub(crate) method: reqwest::Method,
    pub(crate) headers: Headers,
    pub(crate) params: Params,
    pub(crate) body: Vec<u8>,
}

impl Request {
    pub(crate) async fn send(self, client: &reqwest::Client) -> Result<Response> {
        let Self {
            url,
            method,
            headers,
            params,
            body,
        } = self;

        let mut req = client.request(method, url);
        for (k, v) in headers {
            req = req.header(&k, &v);
        }

        if !body.is_empty() {
            req = req.body(body);
        }

        Ok(req.send().await?)
    }
}

pub(crate) trait Credentials: Send + Sync {
    fn access_key_id(&self) -> &str;
    fn access_key_secret(&self) -> &str;
    fn security_token(&self) -> &str;
}

impl Debug for dyn Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("access_key_id", &self.access_key_id().to_string())
            .field("access_key_secret", &self.access_key_secret().to_string())
            .field("security_token", &self.security_token().to_string())
            .finish()
    }
}
