use crate::util::error::AppError;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::HashMap;
use std::fmt::Write;
use tracing::error;

// 生成班级学生账户的邮件模板
pub fn get_student_account_html(accounts: &HashMap<String, String>) -> String {
    // 预估 HTML 静态模板的总长度大约 450 字节
    let template_len = 450;

    // 流式计算所有学生数据的总长度
    // <li>: </li> 一共 10 字节 加上 name 和 password 的实际长度
    let items_len: usize = accounts
        .iter()
        .map(|(name, password)| 10 + name.len() + password.len())
        .sum();

    // 整张大 HTML 页面 只在堆上分配内存空间
    let mut html_buf = String::with_capacity(template_len + items_len);

    // 将 HTML 前半部分写入缓冲区
    html_buf.push_str(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>班级学生账户</title>
</head>
<body>
    <div>
        <h4>下面是生成的账户和对应的登录密码, 请你妥善保管, 不要遗失或者泄露给不相关的人</h4>
        <ul>
            "#,
    );

    // 5. 使用 write! 直接将数据写进 html_buf
    for (name, password) in accounts {
        let _ = write!(html_buf, "<li>{}: {}</li>", name, password);
    }

    // 将 HTML 后半部分写入缓冲区
    html_buf.push_str(
        r"
        </ul>
    </div>
</body>
</html>",
    );

    html_buf
}

// 异步发送邮件
pub async fn send_html_email(
    mailer: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    to: &str,
    subject: String,
    html_body: String,
) -> Result<(), AppError> {
    let from = from.parse::<Mailbox>().map_err(|e| {
        error!("send mail from name err: {}", e);
        AppError::param_error("发件人格式错误")
    })?;

    let to = to.parse::<Mailbox>().map_err(|e| {
        error!("send mail to email err: {}", e);
        AppError::param_error("收件人格式错误")
    })?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| {
            error!("send email content err: {}", e);
            AppError::internal_error("构建邮件失败")
        })?;

    mailer.send(email).await.map_err(|e| {
        error!("send email mailer err: {}", e);
        AppError::internal_error("邮件发送失败")
    })?;

    Ok(())
}
