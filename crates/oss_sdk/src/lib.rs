#[macro_use]
extern crate anyhow;

pub type Result<T> = anyhow::Result<T>;

mod bucket;
mod client;
mod config;
mod conn;
mod error;
mod types;
mod util;
mod version;

pub use bucket::Bucket;
pub use client::Client;
pub use version::VERSION;
