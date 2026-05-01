use sha2::{Sha256,Digest};
pub fn hash_content(content: &str) -> String {
    format!("{:x}", Sha256::digest(content.as_bytes()))
}
