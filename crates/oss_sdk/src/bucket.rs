use std::collections::HashMap;

use crate::client::Client;
use crate::util;
use crate::Result;

#[derive(Clone)]
pub struct Bucket {
    client: Client,
    name: String,
}

impl Bucket {
    pub(crate) fn new(client: Client, name: String) -> Bucket {
        Bucket { client, name }
    }

    pub async fn get_object(&self, object: impl AsRef<str>) -> Result<Vec<u8>> {
        let object = object.as_ref();
        self.do_request(reqwest::Method::GET, object).await
    }

    async fn do_request(&self, method: reqwest::Method, object: &str) -> Result<Vec<u8>> {
        // self.client
        let headers = HashMap::<String, String>::new();
        util::check_bucket_name(&self.name)?;

        self.client
            .conn
            .execute(
                method,
                &self.name,
                object,
                None,
                None,
                Default::default(),
                0,
            )
            .await
    }
}
