use crate::config::ClientConfig;
use crate::Result;
use opendal::services::S3;
use opendal::Operator;
use std::sync::Arc;

pub(crate) fn create(config: &Arc<ClientConfig>) -> Result<Operator> {
    let mut builder = S3::default();
    builder.bucket(&config.bucket);
    builder.endpoint(&config.endpoint);
    builder.region("us-east-1");
    builder.access_key_id(&config.access_key_id);
    builder.secret_access_key(&config.access_key_secret);
    builder.disable_config_load();
    builder.disable_ec2_metadata();
    // OSS need enable virtual host style
    if config.endpoint.contains("aliyuncs.com") {
        builder.enable_virtual_host_style();
    }
    let operator: Operator = Operator::new(builder)?.finish();

    Ok(operator)
}
