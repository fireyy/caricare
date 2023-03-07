mod error;
mod history;
mod image_cache;
pub mod log;
// mod object;
mod oss;
pub mod runtime;
mod session;
mod setting;
pub mod store;
pub mod util;
pub use error::CoreError;
pub use history::MemoryHistory;
pub use image_cache::{ImageCache, ImageFetcher};
// pub use object::{OssBucket, OssObject, OssObjectType};
pub use oss::OssClient;
pub use session::Session;
pub use setting::{Setting, ShowType};
pub use tokio;
pub use tracing;

const LOG_LEVEL: &str = "debug";

pub fn init_core() {
    let mut rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| LOG_LEVEL.to_owned());

    const LOUD_CRATES: [&str; 7] = [
        // wgpu crates spam a lot on info level, which is really annoying
        "naga",
        "wgpu_core",
        "wgpu_hal",
        // These are quite spammy on debug, drowning out what we care about:
        "h2",
        "hyper",
        "reqwest",
        "rustls",
    ];
    for loud_crate in LOUD_CRATES {
        if !rust_log.contains(&format!("{loud_crate}=")) {
            rust_log += &format!(",{loud_crate}=warn");
        }
    }

    std::env::set_var("RUST_LOG", rust_log);

    if std::env::var("RUST_BACKTRACE").is_err() {
        // Make sure we always produce backtraces for the (hopefully rare) cases when we crash!
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    tracing_subscriber::fmt::init();

    runtime::start().unwrap();

    let content_store = store::ContentStore::default();
    content_store.create_req_dirs().unwrap();
}
