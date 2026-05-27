use crate::AppConfig;
use crate::service::question_upload;
use log::error;

/// 批量上传题目

pub async fn upload(config: &AppConfig) {
    if let Err(e) = question_upload::batch(config).await {
        error!("Upload question failed err: {}", e);
    }
}
