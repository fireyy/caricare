mod oss;
pub use aliyun_oss_client::{
    config::GetObjectInfo,
    errors::OssError,
    object::{Object, ObjectList},
    Query,
};
pub use oss::OssConfig;
pub use tracing;

use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub fn setup_tracing() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error")
    }

    println!(
        "Set logging to {}",
        std::env::var("RUST_LOG").unwrap_or("Nothing".to_string())
    );
    tracing::info!("Logging initialized");

    let collector = tracing_subscriber::registry().with(fmt::layer().with_writer(std::io::stdout));

    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
}
