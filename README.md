# Library Book Lending System

A full-stack book lending system with a RESTful API backend built in Rust and a simple HTML/JavaScript frontend.

## Features

- User authentication (Admin and Lender roles)
- Book management (CRUD operations)
- Borrowing and returning books
- Due date tracking and overdue detection
- Admin dashboard for system oversight
- Search functionality

## Tech Stack

**Backend:**
- Rust
- sqlx (SQLite database)
- bcrypt (password hashing)
- No web frameworks (raw HTTP handling)

**Frontend:**
- HTML5
- JavaScript (Vanilla)
- Tailwind CSS

## Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

## Setup Instructions

### 1. Clone the repository

```bash
git clone <repository-url>
cd book-lending-system
```

### 2. Build the project

```bash
cargo build --release
```

### 3. Run the application

```bash
cargo run --release
```

The server will start on `http://127.0.0.1:8080`

### 4. Access the application

Open your web browser and navigate to:
```
http://127.0.0.1:8080
```

## Default Admin Account

- **Username:** admin
- **Password:** admin123

## Project Structure

```
book-lending-system/
├── src/
│   ├── main.rs        # HTTP server and request handlers
│   ├── db.rs          # Database operations
│   ├── auth.rs        # Authentication utilities
│   └── models.rs      # Data structures
├── frontend/
│   ├── index.html     # Login/Register page
│   ├── lender.html    # Lender dashboard
│   ├── admin.html     # Admin dashboard
│   └── app.js         # Frontend JavaScript
├── Cargo.toml         # Rust dependencies
├── schema.sql         # Database schema
└── README.md          # This file
```

## API Documentation

### Base URL
```
http://127.0.0.1:8080/api
```

### Authentication Endpoints

#### Register
```
POST /api/auth/register
Content-Type: application/json

Request Body:
{
  "username": "string",
  "email": "string",
  "password": "string"
}

Response (201):
{
  "id": 1,
  "username": "string",
  "email": "string",
  "role": "lender",
  "created_at": "timestamp"
}
```

#### Login
```
POST /api/auth/login
Content-Type: application/json

Request Body:
{
  "username": "string",
  "password": "string"
}

Response (200):
{
  "token": "uuid-string",
  "user": {
    "id": 1,
    "username": "string",
    "email": "string",
    "role": "lender|admin",
    "created_at": "timestamp"
  }
}
```

#### Logout
```
POST /api/auth/logout
Authorization: Bearer <token>

Response (200):
{
  "message": "Logged out successfully"
}
```

#### Get Current User
```
GET /api/auth/me
Authorization: Bearer <token>

Response (200):
{
  "id": 1,
  "username": "string",
  "email": "string",
  "role": "lender|admin",
  "created_at": "timestamp"
}
```

### Book Endpoints

#### Get All Books
```
GET /api/books

Response (200):
[
  {
    "id": 1,
    "title": "string",
    "author": "string",
    "isbn": "string",
    "publication_year": 2024,
    "genre": "string",
    "total_copies": 5,
    "available_copies": 3,
    "created_at": "timestamp"
  }
]
```

#### Search Books
```
GET /api/books/search?q=<query>

Response (200):
[
  {
    "id": 1,
    "title": "string",
    ...
  }
]
```

#### Create Book (Admin Only)
```
POST /api/books
Authorization: Bearer <admin-token>
Content-Type: application/json

Request Body:
{
  "title": "string",
  "author": "string",
  "isbn": "string",
  "publication_year": 2024,  // optional
  "genre": "string",          // optional
  "total_copies": 5
}

Response (201):
{
  "id": 1,
  "title": "string",
  ...
}
```

#### Update Book (Admin Only)
```
PUT /api/books/:id
Authorization: Bearer <admin-token>
Content-Type: application/json

Request Body:
{
  "title": "string",          // optional
  "author": "string",         // optional
  "isbn": "string",           // optional
  "publication_year": 2024,   // optional
  "genre": "string",          // optional
  "total_copies": 5           // optional
}

Response (200):
{
  "id": 1,
  "title": "string",
  ...
}
```

#### Delete Book (Admin Only)
```
DELETE /api/books/:id
Authorization: Bearer <admin-token>

Response (200):
{
  "message": "Book deleted successfully"
}
```

### Lending Endpoints

#### Borrow Book
```
POST /api/lending/borrow/:book_id
Authorization: Bearer <token>

Response (201):
{
  "message": "Book borrowed successfully",
  "record_id": 1
}
```

#### Return Book
```
POST /api/lending/return/:record_id
Authorization: Bearer <token>

Response (200):
{
  "message": "Book returned successfully"
}
```

#### Get My Borrowed Books
```
GET /api/lending/my-books
Authorization: Bearer <token>

Response (200):
[
  {
    "id": 1,
    "user_id": 2,
    "username": "string",
    "book_id": 1,
    "title": "string",
    "author": "string",
    "borrowed_at": "timestamp",
    "due_date": "timestamp",
    "returned_at": null,
    "status": "borrowed|overdue"
  }
]
```

### Admin Endpoints

#### Get All Users (Admin Only)
```
GET /api/admin/users
Authorization: Bearer <admin-token>

Response (200):
[
  {
    "id": 1,
    "username": "string",
    "email": "string",
    "role": "lender|admin",
    "created_at": "timestamp"
  }
]
```

#### Get Active Lending Records (Admin Only)
```
GET /api/admin/lending/active
Authorization: Bearer <admin-token>

Response (200):
[
  {
    "id": 1,
    "user_id": 2,
    "username": "string",
    "book_id": 1,
    "title": "string",
    "author": "string",
    "borrowed_at": "timestamp",
    "due_date": "timestamp",
    "returned_at": null,
    "status": "borrowed|overdue"
  }
]
```

#### Get Overdue Books (Admin Only)
```
GET /api/admin/lending/overdue
Authorization: Bearer <admin-token>

Response (200):
[
  {
    "id": 1,
    "user_id": 2,
    "username": "string",
    "book_id": 1,
    "title": "string",
    "author": "string",
    "borrowed_at": "timestamp",
    "due_date": "timestamp",
    "returned_at": null,
    "status": "overdue"
  }
]
```

## Database Schema

### Users Table
- `id` (INTEGER PRIMARY KEY)
- `username` (TEXT UNIQUE NOT NULL)
- `email` (TEXT UNIQUE NOT NULL)
- `password_hash` (TEXT NOT NULL)
- `role` (TEXT NOT NULL) - 'admin' or 'lender'
- `created_at` (TIMESTAMP)

### Books Table
- `id` (INTEGER PRIMARY KEY)
- `title` (TEXT NOT NULL)
- `author` (TEXT NOT NULL)
- `isbn` (TEXT UNIQUE NOT NULL)
- `publication_year` (INTEGER)
- `genre` (TEXT)
- `total_copies` (INTEGER NOT NULL)
- `available_copies` (INTEGER NOT NULL)
- `created_at` (TIMESTAMP)

### Lending Records Table
- `id` (INTEGER PRIMARY KEY)
- `user_id` (INTEGER FOREIGN KEY)
- `book_id` (INTEGER FOREIGN KEY)
- `borrowed_at` (TIMESTAMP NOT NULL)
- `due_date` (TIMESTAMP NOT NULL)
- `returned_at` (TIMESTAMP)
- `status` (TEXT NOT NULL) - 'borrowed', 'returned', or 'overdue'

### Sessions Table
- `id` (INTEGER PRIMARY KEY)
- `user_id` (INTEGER FOREIGN KEY)
- `token` (TEXT UNIQUE NOT NULL)
- `expires_at` (TIMESTAMP NOT NULL)
- `created_at` (TIMESTAMP)

## Business Rules

- Books are borrowed for 14 days
- Users can borrow multiple books simultaneously
- Books cannot be borrowed if no copies are available
- Overdue status is automatically updated when fetching overdue books
- Sessions expire after 24 hours
- Passwords are hashed using bcrypt

## Security Features

- Password hashing with bcrypt (cost factor 12)
- Session-based authentication with token expiration
- Role-based access control (Admin vs Lender)
- SQL injection prevention through parameterized queries
- Input validation on all endpoints

## Error Handling

The API returns standard HTTP status codes:
- `200 OK` - Successful request
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid input
- `401 Unauthorized` - Authentication required or failed
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict (e.g., duplicate ISBN)
- `500 Internal Server Error` - Server error

Error responses include a JSON body:
```json
{
  "error": "Error message"
}
```

## Testing

### Manual Testing with curl

#### Register a new user:
```bash
curl -X POST http://127.0.0.1:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"password123"}'
```

#### Login:
```bash
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

#### Get all books:
```bash
curl http://127.0.0.1:8080/api/books
```

#### Add a book (admin):
```bash
curl -X POST http://127.0.0.1:8080/api/books \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-token>" \
  -d '{"title":"The Great Gatsby","author":"F. Scott Fitzgerald","isbn":"9780743273565","publication_year":1925,"genre":"Fiction","total_copies":5}'
```

#### Borrow a book:
```bash
curl -X POST http://127.0.0.1:8080/api/lending/borrow/1 \
  -H "Authorization: Bearer <your-token>"
```

## Development

### Building for Development
```bash
cargo build
```

### Running in Development Mode
```bash
cargo run
```

### Checking Code
```bash
cargo check
```

### Formatting Code
```bash
cargo fmt
```

## Troubleshooting

### Database locked error
If you encounter a "database is locked" error, make sure only one instance of the server is running.

### Connection refused
Make sure the server is running on port 8080 and no firewall is blocking the connection.

### Session expired
If you get unauthorized errors, your session may have expired. Log in again to get a new token.

## License

This project is created for educational purposes as part of an internship screening project.
