use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ClientConfig;
use crate::partial_file::PartialFile;
use crate::transfer::{TransferProgressInfo, TransferSender, TransferType};
use crate::types::{Bucket, ListObjects, Object, Params};
use crate::util::get_name;
use crate::Result;
use anyhow::Context;
use cc_core::ServiceType;

use crate::services;
use crate::stream::{
    AsyncReadProgressExt, BoxedStreamingUploader, StreamingUploader, TrackableBodyStream,
};
use futures::{AsyncReadExt, TryStreamExt};
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

    pub async fn get_object_range(&self, object: impl AsRef<str>) -> Result<(String, Vec<u8>)> {
        let object = object.as_ref();
        let result = self.operator.range_read(object, ..128).await?;

        Ok((object.to_string(), result))
    }

    pub async fn delete_object(&self, object: impl AsRef<str>) -> Result<()> {
        let object = object.as_ref();
        self.operator.delete(object).await?;
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
        let path = query.map_or("".into(), |x| format!("{x}/"));
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

        self.operator.create_dir(&path).await?;
        // TODO: use `stat` to check create dir result

        Ok(())
    }

    // TODO: The copy function is not compatible with OSS
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

    async fn streaming_read(
        &self,
        path: &str,
        start_pos: Option<usize>,
        transfer: TransferSender,
    ) -> Result<Vec<u8>> {
        let reader = match start_pos {
            Some(start_position) => {
                self.operator
                    .range_reader(path, start_position as u64..)
                    .await?
            }
            None => self.operator.reader(path).await?,
        };

        let size = self.head_object(path).await?.content_length();
        let mut body = Vec::new();

        let mut stream = reader.into_async_read().report_progress(|bytes_read| {
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

        let content = self.streaming_read(obj, None, transfer).await?;

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
