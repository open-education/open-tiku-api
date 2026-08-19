use crate::app::config::SmtpEmailConfig;
use crate::util::error::AppError;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::HashMap;
use tracing::error;

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
        <h4>下面是生成的账户和对应的登录密码, 请你妥善保管, 不要遗失或者泄露给不相关的人</h4>
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
    config: &SmtpEmailConfig,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), AppError> {
    let from = if config.from_name.trim().is_empty() {
        format!("<{}>", config.from_email)
            .parse::<Mailbox>()
            .map_err(|e| {
                error!("send mail from email err: {}", e);
                AppError::param_error("发件人格式错误")
            })?
    } else {
        format!("{} <{}>", config.from_name, config.from_email)
            .parse::<Mailbox>()
            .map_err(|e| {
                error!("send mail from name err: {}", e);
                AppError::param_error("发件人格式错误")
            })?
    };

    let to = to.parse::<Mailbox>().map_err(|e| {
        error!("send mail to email err: {}", e);
        AppError::param_error("收件人格式错误")
    })?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| {
            error!("send email content err: {}", e);
            AppError::internal_error("构建邮件失败")
        })?;

    let creds = Credentials::new(config.username.clone(), config.password.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.server)
        .map_err(|e| {
            error!("send email transport err: {}", e);
            AppError::internal_error("SMTP连接失败")
        })?
        .port(config.port)
        .credentials(creds)
        .build();

    mailer.send(email).await.map_err(|e| {
        error!("send email mailer err: {}", e);
        AppError::internal_error("邮件发送失败")
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::app::config::SmtpEmailConfig;
    use crate::util::email::{get_student_account_html, send_html_email};
    use std::collections::HashMap;

    #[actix_web::test]
    async fn test_qq_send_email() {
        let config = SmtpEmailConfig {
            server: "smtp.qq.com".to_string(),
            port: 587,
            username: "978771018@qq.com".to_string(),
            password: "xxx".to_string(), // 必须使用 QQ 邮箱授权码, 不提供在代码中, 去配置中查找
            from_name: "OpenTiku".to_string(),
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
            "测试QQ邮箱-学生账户相关信息",
            get_student_account_html(&map).as_str(),
        )
        .await
        {
            println!("发送失败: {}", e.msg);
        } else {
            println!("发送成功");
        }
    }

    #[actix_web::test]
    async fn test_smarter_mail_send_email() {
        let config = SmtpEmailConfig {
            server: "mail.oef.org.cn".to_string(),
            port: 587,
            username: "z@oef.org.cn".to_string(),
            password: "xxx".to_string(), // 邮箱账户登录密码
            from_name: "OpenTiku".to_string(),
            from_email: "z@oef.org.cn".to_string(),
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
            "978771018@qq.com",
            "测试 SmarterMail-学生账户相关信息",
            get_student_account_html(&map).as_str(),
        )
        .await
        {
            println!("发送失败: {}", e.msg);
        } else {
            println!("发送成功");
        }
    }

    // 容量限制 15000 emails/month 500 emails/day 1 email/2 secs
    #[actix_web::test]
    async fn test_oniqi_send_email() {
        let config = SmtpEmailConfig {
            server: "mail.cyberpersons.com".to_string(),
            port: 587,
            username: "smtp_c74179e9f58058c9".to_string(),
            password: "xxx".to_string(), // 邮箱账户登录密码
            from_name: "OpenTiku".to_string(),
            from_email: "tiku@oniqi.com".to_string(),
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
            "978771018@qq.com",
            "测试 onigi-学生账户相关信息",
            get_student_account_html(&map).as_str(),
        )
        .await
        {
            println!("发送失败: {}", e.msg);
        } else {
            println!("发送成功");
        }
    }
}
