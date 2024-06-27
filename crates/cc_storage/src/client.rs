use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ClientConfig;
use crate::transfer::{TransferProgressInfo, TransferSender, TransferType};
use crate::types::{Bucket, ListObjects, ListObjectsV2Params, Object, Params};
use crate::util::get_name;
use crate::Result;
use cc_core::ServiceType;
use futures::StreamExt;

use crate::services;
use opendal::{Metadata, Metakey, Operator};
use tokio::{
    fs,
    io::{self, AsyncWriteExt as _},
};

const DEFAULT_BUF_SIZE: usize = 8 * 1024 * 1024;

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
        // let result = self.operator.is_exist(object).await?;

        Ok(true)
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
        // let path = query.prefix.map_or("".into(), |x| format!("{x}/"));
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

    pub async fn download_file(
        &self,
        obj: &str,
        target: PathBuf,
        transfer: TransferSender,
    ) -> Result<()> {
        let remote_op = self.operator.clone();
        let progress_tx = transfer.clone();
        let oid = obj.to_string();
        let total_bytes = self.meta_data(obj).await?.content_length();

        tokio::spawn(async move {
            let _: Result<Option<String>> = async {
                fs::create_dir_all(target.parent().unwrap()).await?;
                let mut reader = remote_op.reader_with(&oid).buffer(DEFAULT_BUF_SIZE).await?;
                let mut writer = io::BufWriter::new(fs::File::create(&target).await?);
                copy_with_progress(
                    "download",
                    &progress_tx,
                    &oid,
                    total_bytes,
                    &mut reader,
                    &mut writer,
                )
                .await?;
                writer.shutdown().await?;
                Ok(Some(target.to_string_lossy().into()))
            }
            .await;
        });

        Ok(())
    }

    pub async fn put(&self, path: PathBuf, dest: &str, transfer: &TransferSender) -> Result<()> {
        let name = get_name(&path);
        let key = format!("{dest}{name}");
        let remote_op = self.operator.clone();
        let progress_tx = transfer.clone();
        let total_bytes = fs::metadata(&path).await?.len();

        tokio::spawn(async move {
            let _: Result<Option<String>> = async {
                let mut reader = io::BufReader::new(fs::File::open(path).await?);
                let mut writer = remote_op.writer_with(&key).buffer(DEFAULT_BUF_SIZE).await?;
                copy_with_progress(
                    "upload",
                    &progress_tx,
                    &key,
                    total_bytes,
                    &mut reader,
                    &mut writer,
                )
                .await?;
                writer.close().await?;
                Ok(None)
            }
            .await;
        });

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

async fn copy_with_progress<R, W>(
    tp: &str,
    progress_sender: &TransferSender,
    key: &str,
    total_bytes: u64,
    mut reader: R,
    mut writer: W,
) -> io::Result<usize>
where
    R: io::AsyncReadExt + Unpin,
    W: io::AsyncWriteExt + Unpin,
{
    let mut bytes_so_far: usize = 0;
    let mut buf = vec![0; DEFAULT_BUF_SIZE];

    loop {
        let bytes_since_last = reader.read(&mut buf).await?;
        if bytes_since_last == 0 {
            break;
        }
        writer.write_all(&buf[..bytes_since_last]).await?;
        bytes_so_far += bytes_since_last;
        let msg = if tp == "download" {
            TransferType::Download(
                key.to_string(),
                TransferProgressInfo {
                    total_bytes: total_bytes as usize,
                    transferred_bytes: bytes_so_far,
                },
            )
        } else {
            TransferType::Upload(
                key.to_string(),
                TransferProgressInfo {
                    total_bytes: total_bytes as usize,
                    transferred_bytes: bytes_so_far,
                },
            )
        };
        send_response(progress_sender, msg).await;
    }

    Ok(bytes_so_far)
}

async fn send_response(sender: &TransferSender, msg: TransferType) {
    // tracing::debug!("response: {}", &msg);
    sender.send(msg).unwrap();
}
