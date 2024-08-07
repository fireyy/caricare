use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ClientConfig;
use crate::partial_file::PartialFile;
use crate::transfer::{TransferProgressInfo, TransferSender, TransferType};
use crate::types::{Bucket, ListObjects, ListObjectsV2Params, Object, Params};
use crate::util::get_name;
use crate::Result;
use anyhow::Context;
use cc_core::ServiceType;

use crate::services;
use crate::stream::{
    AsyncReadProgressExt, BoxedStreamingUploader, StreamingUploader, TrackableBodyStream,
};
use futures::{AsyncReadExt, StreamExt, TryStreamExt};
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

        let operator = match &config.service {
            ServiceType::S3 => services::s3::create(&config)?,
            ServiceType::Oss => services::oss::create(&config)?,
            ServiceType::Gcs => services::gcs::create(&config)?,
            ServiceType::Azblob => services::azblob::create(&config)?,
            ServiceType::S3Compatible => services::s3_compatible::create(&config)?,
            // v => {
            //     return Err(anyhow::anyhow!("Unsupported storage type: {:?}", v));
            // }
        };

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

    pub async fn meta_data(&self, object: impl AsRef<str>) -> Result<Metadata> {
        let object = object.as_ref();
        let meta = self.operator.stat(object).await?;

        tracing::debug!("Response header: {:?}", meta);

        Ok(meta)
    }

    pub async fn head_object(&self, object: impl AsRef<str>) -> Result<(Metadata, Vec<u8>)> {
        let object = object.as_ref();
        let meta = self.operator.stat(object).await?;
        let result = self.operator.read_with(object).range(0..256).await?;

        tracing::debug!("Response header: {:?}", meta);

        Ok((meta, result.to_vec()))
    }

    pub async fn get_object(&self, object: impl AsRef<str>) -> Result<(String, Vec<u8>)> {
        let object = object.as_ref();
        let result = self.operator.read(object).await?;

        Ok((object.to_string(), result.to_vec()))
    }

    pub async fn get_object_range(&self, object: impl AsRef<str>) -> Result<(String, Vec<u8>)> {
        let object = object.as_ref();
        let result = self.operator.read_with(object).range(..128).await?;

        Ok((object.to_string(), result.to_vec()))
    }

    pub async fn delete_object(&self, object: impl AsRef<str>) -> Result<bool> {
        let object = object.as_ref();
        self.operator.delete(object).await?;
        let result = self.operator.is_exist(object).await?;

        Ok(result)
    }

    pub async fn delete_multi_object(self, obj: Vec<Object>) -> Result<bool> {
        let mut paths: Vec<String> = vec![];

        for o in obj.iter() {
            paths.push(o.key().into());
        }

        self.operator.remove(paths).await?;
        // TODO: check delete result

        Ok(true)
    }

    pub async fn list_v2(&self, query: ListObjectsV2Params) -> Result<ListObjects> {
        tracing::debug!("List object: {:?}", query);
        let mut path = query.prefix;
        if !path.is_empty() && !path.ends_with('/') {
            path.push('/');
        }
        //TODO 分页功能
        let mut stream = self
            .operator
            .lister_with(&path)
            .start_after(&query.start_after)
            .metakey(Metakey::Mode | Metakey::ContentLength | Metakey::LastModified)
            .await?
            .chunks(100);

        let (mut common_prefixes, mut objects) = (vec![], vec![]);

        let page = stream.next().await.unwrap_or_default();

        let is_truncated = page.len() >= 100;

        for v in page {
            let entry = v?;
            let meta = entry.metadata();
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

        let mut list_objects = ListObjects::default();
        list_objects.set_start_after(if objects.is_empty() {
            common_prefixes.last().unwrap().key().to_owned()
        } else {
            objects.last().unwrap().key().to_owned()
        });
        list_objects.set_is_truncated(is_truncated);
        list_objects.set_common_prefixes(common_prefixes);
        list_objects.set_objects(objects);

        Ok(list_objects)
    }

    pub async fn create_folder(&self, path: String) -> Result<bool> {
        let path = if path.ends_with('/') {
            path
        } else {
            format!("{path}/")
        };
        tracing::debug!("Create folder: {}", path);

        self.operator.create_dir(&path).await?;
        let result = self.operator.is_exist(&path).await?;

        Ok(result)
    }

    pub async fn copy_object(
        &self,
        src: impl AsRef<str>,
        dest: impl AsRef<str>,
        is_move: bool,
    ) -> Result<(String, bool)> {
        let src = src.as_ref();
        let dest = dest.as_ref();

        tracing::debug!("Copy object: {} to: {}", src, dest);

        self.operator.copy(src, dest).await?;

        Ok((src.to_string(), is_move))
    }

    fn streaming_upload(&self, path: &str) -> Result<BoxedStreamingUploader> {
        Ok(Box::new(StreamingUploader::new(
            self.operator.clone(),
            path.to_string(),
        )))
    }

    async fn streaming_read(&self, path: &str, transfer: TransferSender) -> Result<Vec<u8>> {
        let reader = self.operator.reader(path).await?;

        let size = self.meta_data(path).await?.content_length();
        let mut body = Vec::new();

        let mut stream = reader
            .into_futures_async_read(0..)
            .await?
            .report_progress(|bytes_read| {
                transfer
                    .send(TransferType::Download(
                        path.to_string(),
                        TransferProgressInfo {
                            total_bytes: size,
                            transferred_bytes: bytes_read as u64,
                        },
                    ))
                    .unwrap();
            });

        stream
            .read_to_end(&mut body)
            .await
            .context("failed to read object content into buffer")?;

        Ok(body)
    }

    pub async fn download_file(
        &self,
        obj: &str,
        target: PathBuf,
        transfer: TransferSender,
    ) -> Result<()> {
        let mut new_file = PartialFile::create(&target)
            .with_context(|| format!("create `{}`", target.display()))?;

        let content = self.streaming_read(obj, transfer).await?;

        new_file
            .write_all(&content)
            .context("write content of file")?;
        new_file.finish().context("finish writing to new file")?;

        Ok(())
    }

    pub async fn put(&self, path: PathBuf, dest: &str, transfer: &TransferSender) -> Result<()> {
        let name = get_name(&path);
        let key = format!("{dest}{name}");

        let mut body = TrackableBodyStream::try_from(path)
            .map_err(|e| {
                panic!("Could not open sample file: {e}");
            })
            .unwrap();
        let progress_tx = transfer.clone();

        body.set_callback(
            &key,
            move |key: &str, tot_size: u64, sent: u64, _cur_buf: u64| {
                progress_tx
                    .send(TransferType::Upload(
                        key.to_string(),
                        TransferProgressInfo {
                            total_bytes: tot_size,
                            transferred_bytes: sent,
                        },
                    ))
                    .unwrap();
            },
        );

        let mut uploader = self.streaming_upload(&key)?;
        while let Ok(Some(bytes)) = body.try_next().await {
            uploader.write_bytes(bytes).await?;
        }
        uploader.finish().await?;
        // TODO: check if put success

        Ok(())
    }

    pub async fn put_multi(
        &self,
        paths: Vec<PathBuf>,
        dest: String,
        transfer: TransferSender,
    ) -> Result<Vec<String>> {
        let mut results = vec![];
        for path in paths {
            match self.put(path, &dest, &transfer).await {
                Ok(_) => results.push("upload success".into()),
                Err(err) => results.push(err.to_string()),
            }
        }

        Ok(results)
    }

    pub async fn signature_url(
        &self,
        object: &str,
        expire: u64,
        // TODO: join the params like: x-oss-image=
        _params: Option<Params>,
    ) -> Result<String> {
        let url = self
            .operator
            .presign_read(object, std::time::Duration::from_secs(expire))
            .await?;

        Ok(url.uri().to_string())
    }
}

pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn service(mut self, service: &ServiceType) -> Self {
        self.config.service = service.clone();
        self
    }

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
        Client::new(self.config)
    }
}
