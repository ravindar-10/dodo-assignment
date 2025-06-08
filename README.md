# dodo-assignment
This Repo contains the details of dodo assignnmens

# Dodo Assignment - Transaction Management System

A robust transaction management system built with Rust, Actix-web, and PostgreSQL, featuring user authentication, OTP verification, and transaction management.

## Features

- 👤 User Authentication with JWT
- 📱 OTP-based Registration and Verification
- 💰 Transaction Management (Credit/Debit)
- 💵 Balance Tracking
- 🔒 Secure Password Handling

## Tech Stack

- **Backend**: Rust with Actix-web framework
- **Database**: PostgreSQL
- **ORM**: Diesel
- **Authentication**: JWT (JSON Web Tokens)
- **Containerization**: Docker & Docker Compose
- **Testing**: Rust's built-in testing framework with mockall

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Diesel CLI (`cargo install diesel_cli --no-default-features --features postgres`)

## Installation Diesel
- cargo install diesel_cli --no-default-features --features postgres
- diesel print-schema > src/schema.rs

### Local Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/dodo-assignment.git
   cd dodo-assignment
   ```

2. Create a `.env` file in the project root:
   ```env
    DATABASE_URL=postgres://ravindar:postgres@localhost:5432/dodo_payments
    SOCKET_URL="0.0.0.0:8080"
    SUBJECT="Your Dodo Login OTP"
    JWT_SECRET="your-secret-key"
    ADMIN_PASS="" (Please add your pass and email credential)
    ADMIN_EMAIL="work.ravindar@gmail.com"
    ADMIN_USERNAME="work.ravindar@gmail.com"
    RUST_LOG=info
   ```

3. Install dependencies and set up the database:
   ```bash
   # Install Diesel CLI if you haven't already
   cargo install diesel_cli --no-default-features --features postgres

   # Create and set up the database
   diesel setup
   diesel Docker migration run
   ```

4. Build and run the project:
   ```bash
   cargo build
   cargo run
   ```

### local Deployment

1. Build and start the services:
   ```bash
   cargo run
   ```

2. The application will be available at `http://localhost:8080`

## API Endpoints

### User Management

- POST `/register` - Register a new user
  ```json
  {
    "email": "user@example.com",
  }
  ```

- POST `/login` - Verify OTP
  ```json
  {
    "email": "user@example.com",
    "otp": "123456"
  }
  ```

### Transaction Management

- POST `/transactions` - Create a new transaction
  ```json
  {
    "user_id":12,
    "amount": 100.00,
    "transaction_type": "credit",
    "description": "Sample transaction"
  }
  ```

- GET `/transactions` - List all transactions (with pagination)
  - Query params: `page` and `per_page`

- GET `/transactions/{id}` - Get transaction details

- GET `/balance` - Get current balance

## Testing

Run the test suite:
```bash
cargo test
```

For test coverage:
```bash
cargo test
```

## Project Structure

- `src/`
  - `main.rs` - Application entry point
  - `handler.rs` - Request handlers
  - `models.rs` - Data models
  - `schema.rs` - Database schema
  - `db.rs` - Database connection management
  - `repo.rs` - Repository layer
  - `midware/` - Middleware (JWT, etc.)
  - `tests/` - Test modules

- `migrations/` - Database migrations
- `docker/` - Docker configuration files

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
