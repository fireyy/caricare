pub type Result<T> = anyhow::Result<T>;

mod client;
mod config;
mod error;
mod partial_file;
mod stream;
mod types;
pub mod util;
mod version;

pub use client::Client;
pub use error::OSSError;
pub use opendal::Metadata;
pub use types::{Bucket, Headers, ListObjects, Object, ObjectType, Params};
pub use version::VERSION;
