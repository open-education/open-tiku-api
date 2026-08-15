use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String, // QQ邮箱授权码
    pub from_name: String,
    pub from_email: String,
}

// 生成班级学生账户的邮件模板
pub fn get_student_account_html(accounts: &HashMap<String, String>) -> String {
    let mut items = String::new();
    for (name, password) in accounts {
        // 简单转义，防止用户名或密码包含 < > & 等（如有需要）
        // 这里假设数据安全，直接拼接
        items.push_str(&format!("<li>{}: {}</li>", name, password));
    }

    // 完整的 HTML 模板
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>班级学生账户</title>
</head>
<body>
    <div>
        <h4>导入学生账户成功, 下面是生成的账户和对应的登录密码, 请你妥善保管, 不要遗失或者泄露给不相关的人</h4>
        <ul>
            {}
        </ul>
    </div>
</body>
</html>"#,
        items
    )
}

// 异步发送邮件
pub async fn send_html_email(
    config: &EmailConfig,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), Error> {
    let from = if config.from_name.trim().is_empty() {
        format!("<{}>", config.from_email)
            .parse::<Mailbox>()
            .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("发件人格式错误: {}", e)))?
    } else {
        format!("{} <{}>", config.from_name, config.from_email)
            .parse::<Mailbox>()
            .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("发件人格式错误: {}", e)))?
    };

    let to = to
        .parse::<Mailbox>()
        .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("收件人格式错误: {}", e)))?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("构建邮件失败: {}", e)))?;

    let creds = Credentials::new(config.username.clone(), config.password.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
        .map_err(|e| Error::new(ErrorKind::ConnectionRefused, format!("SMTP连接失败: {}", e)))?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    mailer
        .send(email)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("邮件发送失败: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::util::email::{EmailConfig, get_student_account_html, send_html_email};
    use std::collections::HashMap;

    #[actix_web::test]
    async fn test_send_email() {
        let config = EmailConfig {
            smtp_server: "smtp.qq.com".to_string(),
            smtp_port: 587,
            username: "978771018@qq.com".to_string(),
            password: "xxx".to_string(), // 必须使用 QQ 邮箱授权码, 不提供在代码中, 去配置 .env 中查找
            from_name: "zhangguangxun".to_string(),
            from_email: "978771018@qq.com".to_string(),
        };

        let mut map: HashMap<String, String> = HashMap::new();
        map.entry("zhangsan".to_string())
            .or_insert("q2323232332".to_string());
        map.entry("lisi".to_string())
            .or_insert("q2323232sdse332".to_string());
        map.entry("wangwu".to_string())
            .or_insert("q23232342343432332".to_string());

        // 调用异步函数，使用 .await
        if let Err(e) = send_html_email(
            &config,
            "zhangguangxun1@outlook.com",
            "学生账户相关信息",
            get_student_account_html(&map).as_str(),
        )
        .await
        {
            println!("发送失败: {}", e);
        } else {
            println!("发送成功");
        }
    }
}
