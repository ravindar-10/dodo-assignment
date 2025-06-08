mod constants;
mod db;
mod handler;
mod midware;
mod models;
mod repo;
mod schema;
mod tests;
mod transaction_routes;
mod user_routes;
use actix_cors::Cors;
use actix_web::{
	http::header,
	web::{self},
	App, HttpServer,
};

use dotenv::dotenv;
use env_logger::Env;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv().ok();
	env_logger::init_from_env(Env::default().default_filter_or("info"));
	let pool = db::get_db_pool();
	let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET can not be found in .env file");
	let sock_url = env::var("SOCKET_URL").expect("SOCKET_URL");
	if let Err(e) = db::init(&pool).await {
		panic!("Unable to initialize the db. Err: {:?}", e);
	}
	println!("Listening on: {}..", sock_url);

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(pool.clone()))
			.app_data(web::Data::new(jwt_secret.to_string()))
			.wrap(
				Cors::default()
					.allow_any_origin()
					.allow_any_method()
					.allow_any_header()
					.expose_headers(vec![
						header::CONTENT_DISPOSITION,
						header::HeaderName::from_static("x-file-iv"),
					])
					.supports_credentials()
					.max_age(3600),
			)
			.wrap(actix_web::middleware::Logger::default())
			.configure(user_routes::init)
			.configure(transaction_routes::init)
	})
	.bind(&sock_url)?
	.run()
	.await
}
