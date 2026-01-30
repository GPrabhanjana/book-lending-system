use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use sqlx::SqlitePool;
use serde_json::json;

mod models;
mod db;
mod auth;

use models::*;

#[tokio::main]
async fn main() {
    println!("Initializing database...");
    let pool = db::init_db().await.expect("Failed to initialize database");
    println!("Database initialized successfully");
    
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port 8080");
    println!("Server running on http://127.0.0.1:8080");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    handle_connection(stream, pool_clone).await;
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, pool: SqlitePool) {
    let mut buffer = [0; 8192];
    
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            let response = route_request(&request, &pool).await;
            
            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Failed to write response: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}

async fn route_request(request: &str, pool: &SqlitePool) -> String {
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return error_response(400, "Bad Request");
    }
    
    let request_line: Vec<&str> = lines[0].split_whitespace().collect();
    if request_line.len() < 2 {
        return error_response(400, "Bad Request");
    }
    
    let method = request_line[0];
    let path = request_line[1];
    
    // Extract body
    let body = if let Some(pos) = request.find("\r\n\r\n") {
        &request[pos + 4..]
    } else {
        ""
    };
    
    // Extract token from Authorization header
    let token = extract_token(request);
    
    println!("{} {}", method, path);
    
    // Route matching
    match (method, path) {
        // Serve frontend files
        ("GET", "/") => serve_file("frontend/index.html", "text/html"),
        ("GET", "/lender.html") => serve_file("frontend/lender.html", "text/html"),
        ("GET", "/admin.html") => serve_file("frontend/admin.html", "text/html"),
        ("GET", "/app.js") => serve_file("frontend/app.js", "application/javascript"),
        
        // Auth endpoints
        ("POST", "/api/auth/register") => handle_register(pool, body).await,
        ("POST", "/api/auth/login") => handle_login(pool, body).await,
        ("POST", "/api/auth/logout") => handle_logout(pool, token.as_deref()).await,
        ("GET", "/api/auth/me") => handle_get_current_user(pool, token.as_deref()).await,
        
        // Book endpoints
        ("GET", "/api/books") => handle_get_books(pool).await,
        ("POST", "/api/books") => handle_create_book(pool, token.as_deref(), body).await,
        ("PUT", path) if path.starts_with("/api/books/") => {
            let id = path.trim_start_matches("/api/books/").parse::<i64>().unwrap_or(0);
            handle_update_book(pool, token.as_deref(), id, body).await
        },
        ("DELETE", path) if path.starts_with("/api/books/") => {
            let id = path.trim_start_matches("/api/books/").parse::<i64>().unwrap_or(0);
            handle_delete_book(pool, token.as_deref(), id).await
        },
        ("GET", path) if path.starts_with("/api/books/search?") => {
            let query = path.split("q=").nth(1).unwrap_or("");
            let decoded = urlencoding::decode(query).unwrap_or_default();
            handle_search_books(pool, &decoded).await
        },
        
        // Lending endpoints
        ("POST", path) if path.starts_with("/api/lending/borrow/") => {
            let book_id = path.trim_start_matches("/api/lending/borrow/").parse::<i64>().unwrap_or(0);
            handle_borrow_book(pool, token.as_deref(), book_id).await
        },
        ("POST", path) if path.starts_with("/api/lending/return/") => {
            let record_id = path.trim_start_matches("/api/lending/return/").parse::<i64>().unwrap_or(0);
            handle_return_book(pool, token.as_deref(), record_id).await
        },
        ("GET", "/api/lending/my-books") => handle_get_my_books(pool, token.as_deref()).await,
        
        // Admin endpoints
        ("GET", "/api/admin/users") => handle_get_all_users(pool, token.as_deref()).await,
        ("GET", "/api/admin/lending/active") => handle_get_active_lending(pool, token.as_deref()).await,
        ("GET", "/api/admin/lending/overdue") => handle_get_overdue_books(pool, token.as_deref()).await,
        
        _ => error_response(404, "Not Found"),
    }
}

fn extract_token(request: &str) -> Option<String> {
    for line in request.lines() {
        let lower_line = line.to_lowercase();
        if lower_line.starts_with("authorization:") {
            // Get everything after "Authorization: "
            if let Some(auth_value) = line.split(':').nth(1) {
                let auth_value = auth_value.trim();
                // Remove "Bearer " prefix if present
                let token = if auth_value.to_lowercase().starts_with("bearer ") {
                    auth_value[7..].trim().to_string()
                } else {
                    auth_value.to_string()
                };
                println!("Token extracted: {}", &token[..token.len().min(10)]); // Print first 10 chars
                return Some(token);
            }
        }
    }
    None
}

fn serve_file(path: &str, content_type: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                content_type,
                content.len(),
                content
            )
        }
        Err(_) => error_response(404, "File not found"),
    }
}

fn success_response(data: serde_json::Value) -> String {
    let body = data.to_string();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn created_response(data: serde_json::Value) -> String {
    let body = data.to_string();
    format!(
        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn error_response(code: u16, message: &str) -> String {
    let body = json!({ "error": message }).to_string();
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        code,
        message,
        body.len(),
        body
    )
}

async fn authenticate(pool: &SqlitePool, token: Option<&str>) -> Result<User, String> {
    if token.is_none() {
        println!("Authentication failed: No token provided");
        return Err("Unauthorized".to_string());
    }
    
    let token = token.unwrap();
    println!("Authenticating with token: {}...", &token[..token.len().min(10)]);
    
    match db::get_user_by_token(pool, token).await {
        Ok(Some(user)) => {
            println!("Authentication successful for user: {}", user.username);
            Ok(user)
        },
        Ok(None) => {
            println!("Authentication failed: Token not found or expired");
            Err("Unauthorized".to_string())
        },
        Err(e) => {
            println!("Authentication failed: Database error: {:?}", e);
            Err("Unauthorized".to_string())
        }
    }
}

async fn authenticate_admin(pool: &SqlitePool, token: Option<&str>) -> Result<User, String> {
    let user = authenticate(pool, token).await?;
    if user.role != "admin" {
        return Err("Forbidden".to_string());
    }
    Ok(user)
}

// Auth handlers
async fn handle_register(pool: &SqlitePool, body: &str) -> String {
    let req: RegisterRequest = match serde_json::from_str(body) {
        Ok(req) => req,
        Err(_) => return error_response(400, "Invalid request body"),
    };
    
    if req.username.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return error_response(400, "Missing required fields");
    }
    
    let password_hash = match auth::hash_password(&req.password) {
        Ok(hash) => hash,
        Err(_) => return error_response(500, "Failed to hash password"),
    };
    
    match db::create_user(pool, &req.username, &req.email, &password_hash, "lender").await {
        Ok(user_id) => {
            let user = db::get_user_by_id(pool, user_id).await.ok().flatten();
            if let Some(user) = user {
                created_response(serde_json::to_value(user).unwrap())
            } else {
                error_response(500, "Failed to retrieve user")
            }
        }
        Err(_) => error_response(409, "Username or email already exists"),
    }
}

async fn handle_login(pool: &SqlitePool, body: &str) -> String {
    let req: LoginRequest = match serde_json::from_str(body) {
        Ok(req) => req,
        Err(_) => return error_response(400, "Invalid request body"),
    };
    
    let user = match db::get_user_by_username(pool, &req.username).await {
        Ok(Some(user)) => user,
        _ => return error_response(401, "Invalid credentials"),
    };
    
    let valid = match auth::verify_password(&req.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => return error_response(500, "Authentication error"),
    };
    
    if !valid {
        return error_response(401, "Invalid credentials");
    }
    
    let token = auth::generate_token();
    if let Err(_) = db::create_session(pool, user.id, &token).await {
        return error_response(500, "Failed to create session");
    }
    
    let response = LoginResponse {
        token,
        user,
    };
    
    success_response(serde_json::to_value(response).unwrap())
}

async fn handle_logout(pool: &SqlitePool, token: Option<&str>) -> String {
    if let Some(token) = token {
        let _ = db::delete_session(pool, token).await;
    }
    success_response(json!({ "message": "Logged out successfully" }))
}

async fn handle_get_current_user(pool: &SqlitePool, token: Option<&str>) -> String {
    match authenticate(pool, token).await {
        Ok(user) => success_response(serde_json::to_value(user).unwrap()),
        Err(msg) => error_response(401, &msg),
    }
}

// Book handlers
async fn handle_get_books(pool: &SqlitePool) -> String {
    match db::get_all_books(pool).await {
        Ok(books) => success_response(serde_json::to_value(books).unwrap()),
        Err(_) => error_response(500, "Failed to fetch books"),
    }
}

async fn handle_create_book(pool: &SqlitePool, token: Option<&str>, body: &str) -> String {
    if let Err(msg) = authenticate_admin(pool, token).await {
        return error_response(if msg == "Unauthorized" { 401 } else { 403 }, &msg);
    }
    
    let req: CreateBookRequest = match serde_json::from_str(body) {
        Ok(req) => req,
        Err(_) => return error_response(400, "Invalid request body"),
    };
    
    if req.title.is_empty() || req.author.is_empty() || req.isbn.is_empty() || req.total_copies < 0 {
        return error_response(400, "Invalid book data");
    }
    
    match db::create_book(pool, &req).await {
        Ok(book_id) => {
            let book = db::get_book_by_id(pool, book_id).await.ok().flatten();
            if let Some(book) = book {
                created_response(serde_json::to_value(book).unwrap())
            } else {
                error_response(500, "Failed to retrieve book")
            }
        }
        Err(_) => error_response(409, "ISBN already exists"),
    }
}

async fn handle_update_book(pool: &SqlitePool, token: Option<&str>, id: i64, body: &str) -> String {
    if let Err(msg) = authenticate_admin(pool, token).await {
        return error_response(if msg == "Unauthorized" { 401 } else { 403 }, &msg);
    }
    
    let req: UpdateBookRequest = match serde_json::from_str(body) {
        Ok(req) => req,
        Err(_) => return error_response(400, "Invalid request body"),
    };
    
    match db::update_book(pool, id, &req).await {
        Ok(_) => {
            let book = db::get_book_by_id(pool, id).await.ok().flatten();
            if let Some(book) = book {
                success_response(serde_json::to_value(book).unwrap())
            } else {
                error_response(500, "Failed to retrieve updated book")
            }
        }
        Err(_) => error_response(404, "Book not found"),
    }
}

async fn handle_delete_book(pool: &SqlitePool, token: Option<&str>, id: i64) -> String {
    if let Err(msg) = authenticate_admin(pool, token).await {
        return error_response(if msg == "Unauthorized" { 401 } else { 403 }, &msg);
    }
    
    match db::delete_book(pool, id).await {
        Ok(_) => success_response(json!({ "message": "Book deleted successfully" })),
        Err(_) => error_response(404, "Book not found"),
    }
}

async fn handle_search_books(pool: &SqlitePool, query: &str) -> String {
    match db::search_books(pool, query).await {
        Ok(books) => success_response(serde_json::to_value(books).unwrap()),
        Err(_) => error_response(500, "Failed to search books"),
    }
}

// Lending handlers
async fn handle_borrow_book(pool: &SqlitePool, token: Option<&str>, book_id: i64) -> String {
    let user = match authenticate(pool, token).await {
        Ok(user) => user,
        Err(msg) => return error_response(401, &msg),
    };
    
    match db::borrow_book(pool, user.id, book_id).await {
        Ok(record_id) => {
            created_response(json!({ "message": "Book borrowed successfully", "record_id": record_id }))
        }
        Err(_) => error_response(409, "Book not available"),
    }
}

async fn handle_return_book(pool: &SqlitePool, token: Option<&str>, record_id: i64) -> String {
    let user = match authenticate(pool, token).await {
        Ok(user) => user,
        Err(msg) => return error_response(401, &msg),
    };
    
    match db::return_book(pool, record_id, user.id).await {
        Ok(_) => success_response(json!({ "message": "Book returned successfully" })),
        Err(_) => error_response(404, "Lending record not found or already returned"),
    }
}

async fn handle_get_my_books(pool: &SqlitePool, token: Option<&str>) -> String {
    let user = match authenticate(pool, token).await {
        Ok(user) => user,
        Err(msg) => return error_response(401, &msg),
    };
    
    match db::get_user_borrowed_books(pool, user.id).await {
        Ok(records) => success_response(serde_json::to_value(records).unwrap()),
        Err(_) => error_response(500, "Failed to fetch borrowed books"),
    }
}

// Admin handlers
async fn handle_get_all_users(pool: &SqlitePool, token: Option<&str>) -> String {
    if let Err(msg) = authenticate_admin(pool, token).await {
        return error_response(if msg == "Unauthorized" { 401 } else { 403 }, &msg);
    }
    
    match db::get_all_users(pool).await {
        Ok(users) => success_response(serde_json::to_value(users).unwrap()),
        Err(_) => error_response(500, "Failed to fetch users"),
    }
}

async fn handle_get_active_lending(pool: &SqlitePool, token: Option<&str>) -> String {
    if let Err(msg) = authenticate_admin(pool, token).await {
        return error_response(if msg == "Unauthorized" { 401 } else { 403 }, &msg);
    }
    
    match db::get_all_active_lending(pool).await {
        Ok(records) => success_response(serde_json::to_value(records).unwrap()),
        Err(_) => error_response(500, "Failed to fetch lending records"),
    }
}

async fn handle_get_overdue_books(pool: &SqlitePool, token: Option<&str>) -> String {
    if let Err(msg) = authenticate_admin(pool, token).await {
        return error_response(if msg == "Unauthorized" { 401 } else { 403 }, &msg);
    }
    
    match db::get_overdue_books(pool).await {
        Ok(records) => success_response(serde_json::to_value(records).unwrap()),
        Err(_) => error_response(500, "Failed to fetch overdue books"),
    }
}