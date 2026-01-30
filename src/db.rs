use sqlx::{SqlitePool, Row};
use crate::models::*;
use chrono::{Utc, Duration};

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Create database file if it doesn't exist
    let db_path = "library.db";
    
    // Ensure the file can be created by touching it first
    if !std::path::Path::new(db_path).exists() {
        std::fs::File::create(db_path).expect("Failed to create database file");
    }
    
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await?;
    
    // Create tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('admin', 'lender')),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&pool).await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            isbn TEXT UNIQUE NOT NULL,
            publication_year INTEGER,
            genre TEXT,
            total_copies INTEGER NOT NULL,
            available_copies INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&pool).await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS lending_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            book_id INTEGER NOT NULL,
            borrowed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            due_date TIMESTAMP NOT NULL,
            returned_at TIMESTAMP,
            status TEXT NOT NULL CHECK(status IN ('borrowed', 'returned', 'overdue')),
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (book_id) REFERENCES books(id)
        )"
    ).execute(&pool).await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token TEXT UNIQUE NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )"
    ).execute(&pool).await?;
    
    // Insert default admin user (password: 123)
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO users (username, email, password_hash, role) 
         VALUES ('admin', 'admin@library.com', '$2a$12$rfyRaXCM.mNJgnV6t9pOI.EPDV5UhgezjOirtlqBDD2lIyR5BhWIG', 'admin')"
    ).execute(&pool).await;
    
    Ok(pool)
}

// User operations
pub async fn create_user(pool: &SqlitePool, username: &str, email: &str, password_hash: &str, role: &str) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash, role) VALUES (?, ?, ?, ?)"
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .execute(pool)
    .await?;
    
    Ok(result.last_insert_rowid())
}

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, role, created_at FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    
    Ok(user)
}

pub async fn get_user_by_id(pool: &SqlitePool, id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, role, created_at FROM users WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    
    Ok(user)
}

pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, role, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(users)
}

// Session operations
pub async fn create_session(pool: &SqlitePool, user_id: i64, token: &str) -> Result<(), sqlx::Error> {
    let expires_at = Utc::now() + Duration::hours(24);
    
    sqlx::query(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES (?, ?, ?)"
    )
    .bind(user_id)
    .bind(token)
    .bind(expires_at.to_rfc3339())
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_user_by_token(pool: &SqlitePool, token: &str) -> Result<Option<User>, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    
    let user = sqlx::query_as::<_, User>(
        "SELECT u.id, u.username, u.email, u.password_hash, u.role, u.created_at 
         FROM users u 
         INNER JOIN sessions s ON u.id = s.user_id 
         WHERE s.token = ? AND s.expires_at > ?"
    )
    .bind(token)
    .bind(now)
    .fetch_optional(pool)
    .await?;
    
    Ok(user)
}

pub async fn delete_session(pool: &SqlitePool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;
    
    Ok(())
}

// Book operations
pub async fn create_book(pool: &SqlitePool, req: &CreateBookRequest) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO books (title, author, isbn, publication_year, genre, total_copies, available_copies) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.title)
    .bind(&req.author)
    .bind(&req.isbn)
    .bind(req.publication_year)
    .bind(&req.genre)
    .bind(req.total_copies)
    .bind(req.total_copies)
    .execute(pool)
    .await?;
    
    Ok(result.last_insert_rowid())
}

pub async fn get_all_books(pool: &SqlitePool) -> Result<Vec<Book>, sqlx::Error> {
    let books = sqlx::query_as::<_, Book>(
        "SELECT id, title, author, isbn, publication_year, genre, total_copies, available_copies, created_at 
         FROM books ORDER BY title"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(books)
}

pub async fn get_book_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Book>, sqlx::Error> {
    let book = sqlx::query_as::<_, Book>(
        "SELECT id, title, author, isbn, publication_year, genre, total_copies, available_copies, created_at 
         FROM books WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    
    Ok(book)
}

pub async fn update_book(pool: &SqlitePool, id: i64, req: &UpdateBookRequest) -> Result<(), sqlx::Error> {
    let book = get_book_by_id(pool, id).await?;
    if book.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }
    let book = book.unwrap();
    
    let title = req.title.as_ref().unwrap_or(&book.title);
    let author = req.author.as_ref().unwrap_or(&book.author);
    let isbn = req.isbn.as_ref().unwrap_or(&book.isbn);
    let publication_year = req.publication_year.or(book.publication_year);
    let genre = req.genre.as_ref().or(book.genre.as_ref());
    let total_copies = req.total_copies.unwrap_or(book.total_copies);
    
    // Update available copies if total copies changed
    let available_diff = total_copies - book.total_copies;
    let available_copies = book.available_copies + available_diff;
    
    sqlx::query(
        "UPDATE books SET title = ?, author = ?, isbn = ?, publication_year = ?, 
         genre = ?, total_copies = ?, available_copies = ? WHERE id = ?"
    )
    .bind(title)
    .bind(author)
    .bind(isbn)
    .bind(publication_year)
    .bind(genre)
    .bind(total_copies)
    .bind(available_copies)
    .bind(id)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn delete_book(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn search_books(pool: &SqlitePool, query: &str) -> Result<Vec<Book>, sqlx::Error> {
    let search_pattern = format!("%{}%", query);
    
    let books = sqlx::query_as::<_, Book>(
        "SELECT id, title, author, isbn, publication_year, genre, total_copies, available_copies, created_at 
         FROM books 
         WHERE title LIKE ? OR author LIKE ? OR isbn LIKE ? OR genre LIKE ?
         ORDER BY title"
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;
    
    Ok(books)
}

// Lending operations
pub async fn borrow_book(pool: &SqlitePool, user_id: i64, book_id: i64) -> Result<i64, sqlx::Error> {
    // Check if book is available
    let book = get_book_by_id(pool, book_id).await?;
    if book.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }
    let book = book.unwrap();
    
    if book.available_copies <= 0 {
        return Err(sqlx::Error::RowNotFound); // Use as "not available" error
    }
    
    // Create lending record
    let borrowed_at = Utc::now();
    let due_date = borrowed_at + Duration::days(14);
    
    let result = sqlx::query(
        "INSERT INTO lending_records (user_id, book_id, borrowed_at, due_date, status) 
         VALUES (?, ?, ?, ?, 'borrowed')"
    )
    .bind(user_id)
    .bind(book_id)
    .bind(borrowed_at.to_rfc3339())
    .bind(due_date.to_rfc3339())
    .execute(pool)
    .await?;
    
    // Decrease available copies
    sqlx::query("UPDATE books SET available_copies = available_copies - 1 WHERE id = ?")
        .bind(book_id)
        .execute(pool)
        .await?;
    
    Ok(result.last_insert_rowid())
}

pub async fn return_book(pool: &SqlitePool, record_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
    // Get lending record
    let record = sqlx::query(
        "SELECT id, user_id, book_id, status FROM lending_records WHERE id = ?"
    )
    .bind(record_id)
    .fetch_optional(pool)
    .await?;
    
    if record.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }
    
    let record = record.unwrap();
    let record_user_id: i64 = record.get("user_id");
    let book_id: i64 = record.get("book_id");
    let status: String = record.get("status");
    
    if record_user_id != user_id {
        return Err(sqlx::Error::RowNotFound); // Not authorized
    }
    
    if status != "borrowed" && status != "overdue" {
        return Err(sqlx::Error::RowNotFound); // Already returned
    }
    
    // Update lending record
    let returned_at = Utc::now();
    sqlx::query(
        "UPDATE lending_records SET returned_at = ?, status = 'returned' WHERE id = ?"
    )
    .bind(returned_at.to_rfc3339())
    .bind(record_id)
    .execute(pool)
    .await?;
    
    // Increase available copies
    sqlx::query("UPDATE books SET available_copies = available_copies + 1 WHERE id = ?")
        .bind(book_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn get_user_borrowed_books(pool: &SqlitePool, user_id: i64) -> Result<Vec<LendingRecordWithDetails>, sqlx::Error> {
    let records = sqlx::query_as::<_, LendingRecordWithDetails>(
        "SELECT lr.id, lr.user_id, u.username, lr.book_id, b.title, b.author, 
                lr.borrowed_at, lr.due_date, lr.returned_at, lr.status
         FROM lending_records lr
         INNER JOIN users u ON lr.user_id = u.id
         INNER JOIN books b ON lr.book_id = b.id
         WHERE lr.user_id = ? AND lr.status IN ('borrowed', 'overdue')
         ORDER BY lr.borrowed_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    
    Ok(records)
}

pub async fn get_all_active_lending(pool: &SqlitePool) -> Result<Vec<LendingRecordWithDetails>, sqlx::Error> {
    let records = sqlx::query_as::<_, LendingRecordWithDetails>(
        "SELECT lr.id, lr.user_id, u.username, lr.book_id, b.title, b.author, 
                lr.borrowed_at, lr.due_date, lr.returned_at, lr.status
         FROM lending_records lr
         INNER JOIN users u ON lr.user_id = u.id
         INNER JOIN books b ON lr.book_id = b.id
         WHERE lr.status IN ('borrowed', 'overdue')
         ORDER BY lr.borrowed_at DESC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(records)
}

pub async fn get_overdue_books(pool: &SqlitePool) -> Result<Vec<LendingRecordWithDetails>, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    
    // First update overdue status
    sqlx::query(
        "UPDATE lending_records SET status = 'overdue' 
         WHERE status = 'borrowed' AND due_date < ?"
    )
    .bind(&now)
    .execute(pool)
    .await?;
    
    let records = sqlx::query_as::<_, LendingRecordWithDetails>(
        "SELECT lr.id, lr.user_id, u.username, lr.book_id, b.title, b.author, 
                lr.borrowed_at, lr.due_date, lr.returned_at, lr.status
         FROM lending_records lr
         INNER JOIN users u ON lr.user_id = u.id
         INNER JOIN books b ON lr.book_id = b.id
         WHERE lr.status = 'overdue'
         ORDER BY lr.due_date ASC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(records)
}

// Implement FromRow for custom types
impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for User {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            role: row.try_get("role")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Book {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Book {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            author: row.try_get("author")?,
            isbn: row.try_get("isbn")?,
            publication_year: row.try_get("publication_year")?,
            genre: row.try_get("genre")?,
            total_copies: row.try_get("total_copies")?,
            available_copies: row.try_get("available_copies")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for LendingRecordWithDetails {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(LendingRecordWithDetails {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            username: row.try_get("username")?,
            book_id: row.try_get("book_id")?,
            title: row.try_get("title")?,
            author: row.try_get("author")?,
            borrowed_at: row.try_get("borrowed_at")?,
            due_date: row.try_get("due_date")?,
            returned_at: row.try_get("returned_at")?,
            status: row.try_get("status")?,
        })
    }
}