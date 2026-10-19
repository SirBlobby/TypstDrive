use sha2::{Digest, Sha256};

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn sha256_hex(data: &[u8]) -> String {
    to_hex(Sha256::digest(data).as_slice())
}
