use diesel::{
	prelude::*,
	r2d2::{self, ConnectionManager},
};
use dotenv::dotenv;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn get_db_pool() -> DbPool {
	dotenv().ok();
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let manager = ConnectionManager::<PgConnection>::new(database_url);
	r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

pub async fn init(pool: &DbPool) -> Result<(), diesel::result::Error> {
	let mut conn = pool.get().expect("can not get the pool address");
	diesel::sql_query(
		"CREATE TABLE IF NOT EXISTS users (
            email VARCHAR(255) PRIMARY KEY
        );",
	)
	.execute(&mut conn)?;
	diesel::sql_query(
		"CREATE TABLE IF NOT EXISTS otps (
		id SERIAL PRIMARY KEY,
		email VARCHAR(255) NOT NULL,
		otp VARCHAR(6) NOT NULL,
		is_valid BOOLEAN DEFAULT TRUE,
		FOREIGN KEY (email) REFERENCES users(email) ON DELETE CASCADE
	);",
	)
	.execute(&mut conn)?;

	Ok(())
}
