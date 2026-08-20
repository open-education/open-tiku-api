use argon2::password_hash::rand_core::OsRng;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::RngExt;
use rand::prelude::{IndexedRandom, SliceRandom};

// 学生账户密码管理

// 生成一个随机密码, 长度在 8~10 之间, 包含数字, 大小写字母和特殊字符
pub fn generate_random_password() -> String {
    const DIGITS: &[char] = &['2', '3', '4', '5', '6', '7', '8', '9'];
    const UPPERCASE: &[char] = &[
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T',
        'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    const LOWERCASE: &[char] = &[
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't',
        'u', 'v', 'w', 'x', 'y', 'z',
    ];
    const SPECIAL: &[char] = &[
        '!', '@', '#', '$', '%', '^', '*', '(', ')', '_', '+', '-', '=', '[', ']', '{', '}', '|',
        ';', ':', ',', '.', '?', '/', '~',
    ];

    // 全集
    let all_chars: Vec<char> = [DIGITS, UPPERCASE, LOWERCASE, SPECIAL]
        .iter()
        .flat_map(|&set| set.iter().copied())
        .collect();

    let mut rng = rand::rng();
    let len = rng.random_range(8..=10);

    // 每类各取 2 个, 共 8 个
    let mut password_chars = Vec::with_capacity(len);
    for _ in 0..2 {
        password_chars.push(*DIGITS.choose(&mut rng).unwrap());
        password_chars.push(*UPPERCASE.choose(&mut rng).unwrap());
        password_chars.push(*LOWERCASE.choose(&mut rng).unwrap());
        password_chars.push(*SPECIAL.choose(&mut rng).unwrap());
    }

    // 剩余字符从全集中随机取
    for _ in 8..len {
        password_chars.push(*all_chars.choose(&mut rng).unwrap());
    }

    // 打乱顺序
    password_chars.shuffle(&mut rng);

    password_chars.into_iter().collect()
}

// 给用户生成一个密码, 随机盐值和hash值存储在一起
pub fn hash_password(pepper: &str, password: &str) -> Result<String, argon2::password_hash::Error> {
    // SaltString::generate 内部使用 OsRng（系统安全随机数源）
    let salt = SaltString::generate(OsRng);
    let password_with_pepper = format!("{}{}", password, pepper);

    // 使用 Argon2id 默认参数（内存 19 MiB，迭代 3 次，并行 1）
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password_with_pepper.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

// 验证用户密码是否一致
pub fn verify_password(
    pepper: &str,
    password: &str,
    hashed: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hashed)?;
    let password_with_pepper = format!("{}{}", password, pepper);
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password_with_pepper.as_bytes(), &parsed_hash)
        .is_ok())
}
