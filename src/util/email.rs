use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::io::Error as IoError;
use std::io::ErrorKind;

// Outlook 邮箱配置, 后续将其密码放置到 .env 环境变量中
const SMTP_SERVER: &str = "smtp-mail.outlook.com";
const SMTP_PORT: u16 = 587;
const SMTP_USERNAME: &str = "zhangguangxun1@outlook.com";
const SMTP_PASSWORD: &str = "icjyimtooulnqlnp";
const SMTP_FROM_EMAIL: &str = "zhangguangxun1@outlook.com";

pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<(), IoError> {
    let email = Message::builder()
        .from(
            SMTP_FROM_EMAIL
                .parse()
                .map_err(|e| IoError::new(ErrorKind::InvalidInput, e))?,
        )
        .to(to
            .parse()
            .map_err(|e| IoError::new(ErrorKind::InvalidInput, e))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| IoError::new(ErrorKind::Other, e))?;

    let creds = Credentials::new(SMTP_USERNAME.to_string(), SMTP_PASSWORD.to_string());

    let tls_params = TlsParameters::new(SMTP_SERVER.to_string())
        .map_err(|e| IoError::new(ErrorKind::Other, e))?;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(SMTP_SERVER)
        .map_err(|e| IoError::new(ErrorKind::Other, e))?
        .port(SMTP_PORT)
        .credentials(creds)
        .tls(Tls::Required(tls_params))
        .build();

    mailer
        .send(email)
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, e))?;

    Ok(())
}

#[cfg(test)]
#[actix_web::test]
async fn test_send_email() {
    send_email(
        "978771018@qq.com",
        "这是一封测试邮件, 看是否能发送成功",
        "这是一封测试邮件内容, 内容为空没有实际填充",
    )
    .await
    .expect("邮件发送失败");
}
