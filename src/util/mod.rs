pub mod govee;

pub fn digest_sha256(string: &str) -> String {
    sha256::digest(string)
}