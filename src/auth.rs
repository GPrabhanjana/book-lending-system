use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn generate_token() -> String {
    Uuid::new_v4().to_string()
}
