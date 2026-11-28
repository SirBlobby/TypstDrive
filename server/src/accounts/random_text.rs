use uuid::Uuid;

const FULLY_RANDOM_UUID_BYTES: usize = 6;

pub const READABLE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn random_bytes(count: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(count + FULLY_RANDOM_UUID_BYTES);
    while bytes.len() < count {
        bytes.extend_from_slice(&Uuid::new_v4().as_bytes()[..FULLY_RANDOM_UUID_BYTES]);
    }
    bytes.truncate(count);
    bytes
}

pub fn random_text(length: usize) -> String {
    random_bytes(length)
        .iter()
        .map(|byte| READABLE_ALPHABET[*byte as usize % READABLE_ALPHABET.len()] as char)
        .collect()
}

pub fn grouped(text: &str, group_length: usize) -> String {
    text.as_bytes()
        .chunks(group_length)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_text_uses_the_readable_alphabet() {
        let text = random_text(40);
        assert_eq!(text.len(), 40);
        assert!(text.bytes().all(|byte| READABLE_ALPHABET.contains(&byte)));
    }

    #[test]
    fn grouped_splits_with_dashes() {
        assert_eq!(grouped("ABCDEFGH", 4), "ABCD-EFGH");
        assert_eq!(grouped("ABCDE", 4), "ABCD-E");
    }
}
