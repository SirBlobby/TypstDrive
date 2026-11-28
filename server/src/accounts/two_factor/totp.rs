use std::time::{SystemTime, UNIX_EPOCH};

use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

use crate::accounts::random_text::random_bytes;

const STEP_SECONDS: u64 = 30;
const CODE_DIGITS: u32 = 6;
const SECRET_BYTES: usize = 20;
const ALLOWED_DRIFT_STEPS: u64 = 1;

pub fn generate_secret() -> String {
    BASE32_NOPAD.encode(&random_bytes(SECRET_BYTES))
}

fn current_step() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() / STEP_SECONDS)
        .unwrap_or(0)
}

fn code_for_step(secret: &[u8], step: u64) -> Option<u32> {
    let mut mac = Hmac::<Sha1>::new_from_slice(secret).ok()?;
    mac.update(&step.to_be_bytes());
    let digest = mac.finalize().into_bytes();
    let offset = (digest[digest.len() - 1] & 0x0f) as usize;
    let number = u32::from_be_bytes([digest[offset] & 0x7f, digest[offset + 1], digest[offset + 2], digest[offset + 3]]);
    Some(number % 10u32.pow(CODE_DIGITS))
}

fn entered_number(code: &str) -> Option<u32> {
    let digits: String = code.chars().filter(char::is_ascii_digit).collect();
    if digits.len() != CODE_DIGITS as usize {
        return None;
    }
    digits.parse().ok()
}

pub fn matching_step_at(secret: &str, code: &str, last_used_step: i64, now_step: u64) -> Option<i64> {
    let entered = entered_number(code)?;
    let secret = BASE32_NOPAD.decode(secret.as_bytes()).ok()?;
    (now_step.saturating_sub(ALLOWED_DRIFT_STEPS)..=now_step + ALLOWED_DRIFT_STEPS)
        .filter(|step| *step as i64 > last_used_step)
        .find(|step| code_for_step(&secret, *step) == Some(entered))
        .map(|step| step as i64)
}

pub fn matching_step(secret: &str, code: &str, last_used_step: i64) -> Option<i64> {
    matching_step_at(secret, code, last_used_step, current_step())
}

fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

pub fn setup_uri(issuer: &str, account: &str, secret: &str) -> String {
    let label = format!("{}:{}", percent_encode(issuer), percent_encode(account));
    format!(
        "otpauth://totp/{label}?secret={secret}&issuer={}&algorithm=SHA1&digits={CODE_DIGITS}&period={STEP_SECONDS}",
        percent_encode(issuer)
    )
}

pub struct QrCodeGrid {
    pub size: usize,
    pub modules: String,
}

pub fn qr_code(text: &str) -> Option<QrCodeGrid> {
    let code = qrcode::QrCode::new(text.as_bytes()).ok()?;
    let modules = code
        .to_colors()
        .iter()
        .map(|color| if *color == qrcode::Color::Dark { '1' } else { '0' })
        .collect();
    Some(QrCodeGrid { size: code.width(), modules })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    #[test]
    fn matches_rfc_6238_sha1_vectors() {
        let secret = BASE32_NOPAD.decode(RFC_SECRET.as_bytes()).unwrap();
        assert_eq!(code_for_step(&secret, 59 / STEP_SECONDS), Some(287082));
        assert_eq!(code_for_step(&secret, 1111111109 / STEP_SECONDS), Some(81804));
        assert_eq!(code_for_step(&secret, 1234567890 / STEP_SECONDS), Some(5924));
        assert_eq!(code_for_step(&secret, 2000000000 / STEP_SECONDS), Some(279037));
    }

    #[test]
    fn accepts_neighbouring_steps_and_rejects_reuse() {
        let step = 1234567890 / STEP_SECONDS;
        assert_eq!(matching_step_at(RFC_SECRET, "005924", 0, step + 1), Some(step as i64));
        assert_eq!(matching_step_at(RFC_SECRET, "005 924", 0, step), Some(step as i64));
        assert_eq!(matching_step_at(RFC_SECRET, "005924", step as i64, step), None);
        assert_eq!(matching_step_at(RFC_SECRET, "005924", 0, step + 2), None);
        assert_eq!(matching_step_at(RFC_SECRET, "12345", 0, step), None);
    }

    #[test]
    fn builds_a_setup_uri_and_qr_code() {
        let uri = setup_uri("Typst Drive", "me@example.com", RFC_SECRET);
        assert!(uri.starts_with("otpauth://totp/Typst%20Drive:me%40example.com?secret="));
        let grid = qr_code(&uri).unwrap();
        assert_eq!(grid.modules.len(), grid.size * grid.size);
    }
}
