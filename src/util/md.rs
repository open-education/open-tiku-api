use crate::api::question::QuestionUploadReq;
use crate::constant::meta;
use crate::service::index::QuestionIndex;
use crate::util::string;
use log::{error, info, warn};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use tokio::sync::broadcast;
use tokio::task;
use tokio::task::JoinHandle;

/// 写入单个 Markdown 文件（支持取消）
async fn write_single_markdown_file_with_cancel(
    filepath: &str,
    markdown_content: &str,
    mut cancel_rx: broadcast::Receiver<()>,
) -> Result<String, Error> {
    // 检查是否已经收到取消信号
    if cancel_rx.try_recv().is_ok() {
        return Err(Error::new(ErrorKind::Interrupted, "task canceled"));
    }

    if !string::has_content(markdown_content) {
        return Ok("".to_string()); // empty file name -> no file
    }

    // 使用 tokio 的异步文件写入
    tokio::fs::write(&filepath, &markdown_content).await?;

    // 再次检查取消信号（在写入后）
    if cancel_rx.try_recv().is_ok() {
        // 如果收到取消信号，删除刚刚创建的文件
        let _ = tokio::fs::remove_file(&filepath).await;
        return Err(Error::new(ErrorKind::Interrupted, "task canceled"));
    }

    Ok(filepath.to_string())
}

// title.md
fn write_title_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) -> Result<bool, Error> {
    if !string::has_content(markdown_content.as_str()) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "markdown title content is missing",
        ));
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_TITLE_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
    Ok(true)
}

// mention.md
fn write_mention_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_MENTION_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select a.md
fn write_select_a_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_A_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select b.md
fn write_select_b_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_B_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select c.md
fn write_select_c_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_C_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select d.md
fn write_select_d_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_D_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select e.md
fn write_select_e_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_E_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select answer.md
fn write_answer_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_ANSWER_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select knowledge.md
fn write_knowledge_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_KNOWLEDGE_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select analyze.md
fn write_analyze_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_ANALYZE_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select process.md
fn write_process_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_PROCESS_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

// select remark.md
fn write_remark_md(
    file_path: &str,
    markdown_content: String,
    cancel_tx: &broadcast::Sender<()>,
    handles: &mut Vec<JoinHandle<Result<String, Error>>>,
) {
    if !string::has_content(markdown_content.as_str()) {
        return;
    }

    // 每个任务获取一个接收器
    let cancel_rx = cancel_tx.subscribe();
    let file_path = format!("{}/{}", &file_path, meta::QUESTION_REMARK_NAME);
    let handle = task::spawn(async move {
        write_single_markdown_file_with_cancel(&file_path, &markdown_content, cancel_rx).await
    });
    handles.push(handle);
}

/// 原子性并发写入 Markdown 文件
/// 如果任何文件写入失败，会取消所有任务并清理已创建的文件
pub async fn write_markdown_files(
    meta_path: &str,
    req: QuestionUploadReq,
    index: &QuestionIndex,
) -> Result<bool, Error> {
    // md directory: id_left
    let file_path = format!(
        "{}/{}/{}/{}_{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        index.id,
        index.left
    );
    info!("writing markdown file path: {}", &file_path);
    if !Path::new(&file_path).exists() {
        fs::create_dir_all(&file_path)?;
    }

    // 使用 broadcast 通道进行取消信号
    let (cancel_tx, _) = broadcast::channel(1);
    let mut handles = Vec::new();
    let mut successful_files = Vec::new();

    // 启动所有写入任务
    write_title_md(&file_path, req.title_val, &cancel_tx, &mut handles)?; // because title is required
    if let Some(mention_val) = req.mention_val {
        write_mention_md(&file_path, mention_val, &cancel_tx, &mut handles);
    }
    if let Some(a_val) = req.a_val {
        write_select_a_md(&file_path, a_val, &cancel_tx, &mut handles);
    }
    if let Some(b_val) = req.b_val {
        write_select_b_md(&file_path, b_val, &cancel_tx, &mut handles);
    }
    if let Some(c_val) = req.c_val {
        write_select_c_md(&file_path, c_val, &cancel_tx, &mut handles);
    }
    if let Some(d_val) = req.d_val {
        write_select_d_md(&file_path, d_val, &cancel_tx, &mut handles);
    }
    if let Some(e_val) = req.e_val {
        write_select_e_md(&file_path, e_val, &cancel_tx, &mut handles);
    }
    if let Some(answer_val) = req.answer_val {
        write_answer_md(&file_path, answer_val, &cancel_tx, &mut handles);
    }
    if let Some(knowledge_val) = req.knowledge_val {
        write_knowledge_md(&file_path, knowledge_val, &cancel_tx, &mut handles);
    }
    if let Some(analyze_val) = req.analyze_val {
        write_analyze_md(&file_path, analyze_val, &cancel_tx, &mut handles);
    }
    if let Some(process_val) = req.process_val {
        write_process_md(&file_path, process_val, &cancel_tx, &mut handles);
    }
    if let Some(remark_val) = req.remark_val {
        write_remark_md(&file_path, remark_val, &cancel_tx, &mut handles);
    }

    // 收集结果，遇到第一个错误就取消所有任务
    let mut has_failure = false;
    let mut error_message = None;

    for handle in handles {
        // 检查是否已经收到取消信号
        if has_failure {
            handle.abort(); // 取消未完成的任务
            warn!("writing markdown file path task canceled");
            continue;
        }

        match handle.await {
            Ok(Ok(filename)) => {
                if filename.len() > 0 {
                    info!("successful markdown file path : {}", &filename);
                    successful_files.push(filename);
                }
            }
            Ok(Err(e)) => {
                // 第一个失败的任务
                has_failure = true;
                error_message = Some(e.to_string());
                error!("writing markdown file path error: {}", e);
                // 发送取消信号给所有任务
                let _ = cancel_tx.send(());
            }
            Err(e) if e.is_cancelled() => {
                warn!("writing markdown file path task canceled {}", e);
            }
            Err(e) => {
                has_failure = true;
                error_message = Some(format!("任务执行错误: {}", e));
                error!("writing markdown file path exec error: {}", e);
                let _ = cancel_tx.send(());
            }
        }
    }

    // 如果有失败，删除新建文件夹即可
    if has_failure {
        let path = Path::new(file_path.as_str());
        if path.exists() && path.is_dir() {
            match fs::remove_dir_all(path) {
                Ok(_) => {}
                Err(e) => Err(Error::new(
                    ErrorKind::Interrupted,
                    format!("writing markdown file path remove error: {}", e),
                ))?,
            }
        }

        return Err(Error::new(
            ErrorKind::Interrupted,
            format!("题目上传失败: {}", error_message.unwrap()),
        ));
    }

    info!(
        "writing markdown file path successful: {:?}",
        successful_files
    );

    Ok(true)
}
