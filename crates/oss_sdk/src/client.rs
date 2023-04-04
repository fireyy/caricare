use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ClientConfig;
use crate::types::{Bucket, ListObjects, Object, Params};
use crate::util::{self, get_name};
use crate::Result;

use futures::TryStreamExt;
use opendal::services::Oss;
use opendal::{Metadata, Metakey, Operator};

#[derive(Clone)]
pub struct Client {
    pub(crate) config: Arc<ClientConfig>,
    operator: Operator,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            config: Default::default(),
        }
    }

    fn new(config: ClientConfig) -> Result<Client> {
        let config = Arc::new(config);

        let mut builder = Oss::default();
        builder.bucket(&config.bucket);
        builder.endpoint(&config.endpoint);
        builder.access_key_id(&config.access_key_id);
        builder.access_key_secret(&config.access_key_secret);
        let operator: Operator = Operator::new(builder)?.finish();

        Ok(Client { config, operator })
    }

    pub fn get_bucket_url(&self) -> String {
        let url = &self.config.endpoint;
        let name_str = self.config.bucket.to_string();

        let mut name = String::from("https://");
        name.push_str(&name_str);
        name.push('.');

        url.replace("https://", &name)
    }

    pub async fn get_bucket_info(&self) -> Result<Bucket> {
        //TODO: check for bucket acl
        let grant = Bucket::get_acl_from_str("private");

        Ok(Bucket::new(self.config.bucket.to_owned(), grant))
    }

    pub async fn head_object(&self, object: impl AsRef<str>) -> Result<Metadata> {
        let object = object.as_ref();
        let meta = self.operator.stat(object).await?;

        tracing::debug!("Response header: {:?}", meta);

        Ok(meta)
    }

    pub async fn get_object(&self, object: impl AsRef<str>) -> Result<(String, Vec<u8>)> {
        let object = object.as_ref();
        let result = self.operator.read(object).await?;

        Ok((object.to_string(), result))
    }

    pub async fn delete_object(&self, object: impl AsRef<str>) -> Result<()> {
        let object = object.as_ref();
        let _ = self.operator.delete(object).await?;
        // TODO: check `object` if delete success

        Ok(())
    }

    pub async fn delete_multi_object(self, obj: Vec<Object>) -> Result<()> {
        let mut paths: Vec<String> = vec![];

        for o in obj.iter() {
            paths.push(o.key().into());
        }

        self.operator.remove(paths).await?;
        // TODO: check delete result

        Ok(())
    }

    pub async fn list_v2(&self, query: Option<String>) -> Result<ListObjects> {
        tracing::debug!("List object: {:?}", query);
        let path = query.map_or("".into(), |x| format!("{}/", x));
        let mut stream = self.operator.list(&path).await?;

        let mut list_objects = ListObjects::default();
        let mut common_prefixes = Vec::new();
        let mut objects = Vec::new();

        while let Some(entry) = stream.try_next().await? {
            let meta = self
                .operator
                .metadata(
                    &entry,
                    Metakey::Mode | Metakey::ContentLength | Metakey::LastModified,
                )
                .await?;

            if meta.is_dir() {
                common_prefixes.push(Object::new_folder(entry.path()));
            } else {
                objects.push(Object::new(
                    entry.path(),
                    meta.last_modified(),
                    meta.content_length() as usize,
                ));
            }
        }

        list_objects.set_common_prefixes(common_prefixes);
        list_objects.set_objects(objects);

        Ok(list_objects)
    }

    pub async fn create_folder(&self, path: String) -> Result<()> {
        let path = if path.ends_with('/') {
            path
        } else {
            format!("{path}/")
        };
        tracing::debug!("Create folder: {}", path);

        let _ = self.operator.create_dir(&path).await?;
        // TODO: use `stat` to check create dir result

        Ok(())
    }

    pub async fn copy_object(
        &self,
        src: impl AsRef<str>,
        dest: impl AsRef<str>,
        is_move: bool,
    ) -> Result<(String, bool)> {
        let src = src.as_ref();
        let dest = dest.as_ref();

        tracing::debug!("Copy object: {}", src);

        let mut dst_w = self.operator.writer(&dest).await?;
        let reader = self.operator.reader(&src).await?;
        let buf_reader = futures::io::BufReader::with_capacity(8 * 1024 * 1024, reader);
        futures::io::copy_buf(buf_reader, &mut dst_w).await?;
        // flush data
        dst_w.close().await?;

        Ok((src.to_string(), is_move))
    }

    pub async fn put(&self, path: PathBuf, dest: &str) -> Result<()> {
        let name = get_name(&path);
        let key = format!("{dest}{name}");
        let file_content = std::fs::read(path)?;

        let _ = self.operator.write(&key, file_content).await?;
        // TODO: check if put success

        Ok(())
    }

    pub async fn put_multi(&self, paths: Vec<PathBuf>, dest: String) -> Result<Vec<String>> {
        let mut results = vec![];
        for path in paths {
            match self.put(path, &dest).await {
                Ok(_) => results.push("upload success".into()),
                Err(err) => results.push(err.to_string()),
            }
        }

        Ok(results)
    }

    pub fn signature_url(
        &self,
        object: &str,
        expire: i64,
        // TODO: join the params like: x-oss-image=
        _params: Option<Params>,
    ) -> Result<String> {
        let url = self
            .operator
            .presign_read(object, time::Duration::seconds(expire))?;

        Ok(url.uri().to_string())
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

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.config.bucket = bucket.into();
        self
    }

    pub fn build(self) -> Result<Client> {
        util::check_bucket_name(&self.config.bucket)?;
        Client::new(self.config)
    }
}
