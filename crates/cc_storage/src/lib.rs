pub type Result<T> = anyhow::Result<T>;

mod client;
mod config;
mod custom_layer;
mod error;
mod partial_file;
mod services;
mod stream;
mod transfer;
mod types;
pub mod util;
mod version;

pub use cc_core::ServiceType;
pub use client::Client;
pub use custom_layer::CustomLayer;
pub use error::OSSError;
pub use opendal::{Lister, Metadata};
pub use transfer::TransferManager;
pub use types::{Bucket, Headers, ListObjects, Object, ObjectType, Params};
pub use version::VERSION;
