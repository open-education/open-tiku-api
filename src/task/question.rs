use crate::AppConfig;
use crate::service::question_upload;
use log::error;

/// 批量上传题目
///
/// 部署定时任务时避免任务重复启动, 使用文件锁的方式控制
/// */5 * * * * /usr/bin/flock -n /tmp/my_task.lock -c './a/ddd task question-upload'
/// 具体的文件需要查看自己的部署机器使用一个统一的文件即可
/// 执行命令 -c 后面需要使用单或者双引号包裹
///

pub async fn upload(config: &AppConfig) {
    if let Err(e) = question_upload::batch(config).await {
        error!("Upload question failed err: {}", e);
    }
}
