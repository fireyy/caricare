#[macro_use]
extern crate anyhow;

pub type Result<T> = anyhow::Result<T>;

mod client;
mod config;
mod conn;
mod error;
mod types;
mod util;
mod version;

pub use client::Client;
pub use error::OSSError;
pub use types::{Headers, ListObjects, Object, ObjectType, Params};
pub use version::VERSION;
