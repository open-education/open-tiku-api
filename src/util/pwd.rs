use crate::constant::meta::STUDENT_LOGIN_TIME_WINDOW_MS;
use crate::util::error::AppError;
use base64::{Engine, prelude::BASE64_STANDARD};
use rsa::sha2::Sha256;
use rsa::{Oaep, RsaPrivateKey, pkcs8::DecodePrivateKey};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, warn};

// 用私钥解密密码字段

// 获取用户明文密码
pub fn get_pwd(password: &str, absolute_path: &str) -> Result<String, AppError> {
    let private_key_content = load_private_key(absolute_path).map_err(|err| {
        error!("Get private key error: {}", err);
        AppError::param_error("私钥文件读取失败")
    })?;

    let d_pwd = decrypt_pwd(password, &private_key_content).map_err(|err| {
        error!("Decrypt error: {}", err);
        AppError::param_error("密码解密失败")
    })?;

    verify_and_get_pwd(&d_pwd)
}

// 根据绝对路径读取私钥内容
fn load_private_key(absolute_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 校验路径是否存在，方便精准排查问题
    let path = Path::new(absolute_path);
    if !path.exists() {
        error!("Load private key file:  {} not exist", absolute_path);
        return Err(format!("私钥文件不存在，请检查路径: {}", absolute_path).into());
    }

    // 读取绝对路径的文件内容并转换为 String
    let private_key_content = fs::read_to_string(path)?;

    Ok(private_key_content)
}

// rsa 解密密码字段
fn decrypt_pwd(
    encrypted_base64: &str,
    pem_private_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 解析 PEM 格式的 PKCS#8 私钥
    let private_key = RsaPrivateKey::from_pkcs8_pem(pem_private_key)?;

    // 将前端传来的 Base64 密文解码为字节数组
    let encrypted_bytes = BASE64_STANDARD.decode(encrypted_base64)?;

    // 配置与前端一致的 RSA-OAEP (SHA-256) 填充模式
    let padding = Oaep::new::<Sha256>();

    // 使用私钥解密
    let decrypted_bytes = private_key.decrypt(padding, &encrypted_bytes)?;

    // 将结果转回明文密码字符串
    let password = String::from_utf8(decrypted_bytes)?;

    Ok(password)
}

// 验证密码完整和安全性
fn verify_and_get_pwd(d_pwd: &str) -> Result<String, AppError> {
    let parts: Vec<&str> = d_pwd.split('|').collect();
    if parts.len() != 3 {
        error!("Verify d_pwd format err: {}", d_pwd);
        return Err(AppError::param_error("密码格式生成错误"));
    }

    let password = parts[0];
    let timestamp = parts[1];
    let nonce = parts[2];

    // 校验时间戳
    let client_time = timestamp.parse::<u64>().map_err(|err| {
        error!("Verify pwd timestamp error: {}", err);
        AppError::param_error("密码时间戳生成错误")
    })?;
    let server_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // 检查请求是否超时 或者是否是未来的请求 防止客户端时钟严重不准
    if client_time > server_time + STUDENT_LOGIN_TIME_WINDOW_MS
        || server_time > client_time + STUDENT_LOGIN_TIME_WINDOW_MS
    {
        return Err(AppError::param_error("登录时间过长, 请刷新后重新登录"));
    }

    // nonce 字段是为了防止加密后的密文泄露被重放, 所以该值设置为登录时间窗口内仅使用一次即废弃, 但是目前没有 redis 等类似的缓存中间件
    // 内存存储又会增加维护和清理负担, 故暂时保留该字段并为做校验, 所以登录时间窗口内能被重放

    warn!("Verify d_pwd nonce is missing: {}", nonce);

    Ok(password.to_string())
}
