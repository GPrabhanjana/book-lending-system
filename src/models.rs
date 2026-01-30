use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub publication_year: Option<i32>,
    pub genre: Option<String>,
    pub total_copies: i32,
    pub available_copies: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LendingRecordWithDetails {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub book_id: i64,
    pub title: String,
    pub author: String,
    pub borrowed_at: String,
    pub due_date: String,
    pub returned_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub publication_year: Option<i32>,
    pub genre: Option<String>,
    pub total_copies: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub genre: Option<String>,
    pub total_copies: Option<i32>,
}