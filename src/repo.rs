use crate::{
	db::DbPool,
	midware::jwt::Claims,
	models::{ErrorResponse, NewOtp, NewUser, Otp, User},
	schema::{otps, users},
};
use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use lettre::{
	message::header::ContentType, transport::smtp::authentication::Credentials, Message,
	SmtpTransport, Transport,
};
use rand::Rng;
pub struct UserRepo<'a> {
	_pool: &'a DbPool,
}

impl<'a> UserRepo<'a> {
	// pub fn new(pool: &'a DbPool) -> Self {
	// 	Self { pool }
	// }

	pub fn send_otp_email(
		to_email: &str,
		user_otp: &str,
	) -> Result<(), Box<dyn std::error::Error>> {
		let expiration_time = 10;
		let splitted_email = to_email.split("@").collect::<Vec<&str>>();
		let first_name = splitted_email.get(0).expect("not able to get the first name");
		let email_body: String = format!(
			"Hi {} ,

Welcome to Dodo Payment! We’re excited to have you join us.

To complete your login, please use the following One-Time Password (OTP):

OTP: {}

This OTP is valid for the next {} minutes. Please do not share this code with anyone.

If you did not request this, please ignore this message or contact our support team immediately.

Thank you,
The Dodo Payment Team",
			first_name, user_otp, expiration_time,
		);
		let admin_email: String =
			std::env::var("ADMIN_EMAIL").expect("ADMIN_EMAIL can not be found in .env file");
		let admin_pass =
			std::env::var("ADMIN_PASS").expect("ADMIN_PASS can not be found in .env file");
		let subject = std::env::var("SUBJECT").expect("SUBJECT can not be found in .env file");
		let admin_username =
			std::env::var("ADMIN_USERNAME").expect("ADMIN_USERNAME can not be found in .env file");
		let user_email = Message::builder()
			.from(admin_email.parse().expect("admin email is not correct"))
			.reply_to(admin_email.parse().expect("admin password is not correct"))
			.to(to_email.parse().expect("user email is not correct"))
			.subject(subject)
			.header(ContentType::TEXT_PLAIN)
			.body(email_body)?;
		let creds = Credentials::new(admin_username, admin_pass.to_owned());
		// let mailer =
		// SmtpTransport::relay("smtp.zeptomail.in").unwrap().credentials(creds).build();
		let mailer = SmtpTransport::relay("smtp.gmail.com").unwrap().credentials(creds).build();

		match mailer.send(&user_email) {
			Ok(_) => {
				log::info!("OTP sent successfully to {}", to_email);
				Ok(())
			},
			Err(e) => {
				log::info!("Could not send OTP: {:?}", e);
				Err(Box::new(e))
			},
		}
	}

	pub fn generate_otp() -> String {
		let other_otp: i32 = rand::thread_rng().gen_range(100_000..1_000_000);
		other_otp.to_string()
	}
	pub async fn store_otp_in_db(
		pool: web::Data<DbPool>,
		new_user_email: &str,
		new_user: NewUser<'a>,
		otp_code: &str,
	) -> Result<(), Box<dyn std::error::Error>> {
		let mut conn = pool.get()?;
		let existing_user = users::dsl::users
			.filter(users::dsl::email.eq(new_user_email))
			.select(User::as_select())
			.first::<User>(&mut conn)
			.optional()
			.unwrap();
		if existing_user.is_none() || new_user_email != existing_user.unwrap().email {
			diesel::insert_into(users::dsl::users).values(&new_user).execute(&mut conn)?;
		}
		let user_id_res = users::dsl::users
			.filter(users::dsl::email.eq(&new_user_email))
			.select(users::dsl::id)
			.first::<i32>(&mut conn)
			.map_err(|e| {
				log::error!("User fetch error: {:?}", e);
				HttpResponse::NotFound().json(ErrorResponse { error: "User not found".to_string() })
			});
		let new_user_id = match user_id_res {
			Ok(id) => id,
			Err(_) => return Err("User not found".into()),
		};
		let new_otp = NewOtp { user_id: new_user_id, otp: otp_code, is_valid: true };

		diesel::insert_into(otps::dsl::otps).values(&new_otp).execute(&mut conn)?;
		Ok(())
	}
	pub fn verify_otp_in_db(
		pool: web::Data<DbPool>,
		user_email: &str,
		otp_code: &i64,
	) -> Result<(), Box<dyn std::error::Error>> {
		let mut conn = pool.get()?;

		// First get the user_id from email
		let other_user_id: i32 = users::dsl::users
			.filter(users::dsl::email.eq(user_email))
			.select(users::dsl::id)
			.first::<i32>(&mut conn)?;

		// Then get the OTP using the user_id
		let db_otp = otps::dsl::otps
			.filter(otps::dsl::user_id.eq(&other_user_id))
			.filter(otps::dsl::is_valid.eq(&true))
			.first::<Otp>(&mut conn)?;

		if db_otp.otp == otp_code.to_string() {
			diesel::update(otps::table)
				.filter(otps::user_id.eq(other_user_id))
				.set(otps::is_valid.eq(false))
				.execute(&mut conn)
				.map_err(|_| "Failed to update OTP validity")?;

			Ok(())
		} else {
			Err("OTP is not valid or expired".into())
		}
	}
}
pub async fn authenticate(req: &HttpRequest) -> Result<String, HttpResponse> {
	let auth_header = req
		.headers()
		.get("Authorization")
		.and_then(|h| h.to_str().ok())
		.and_then(|h| h.strip_prefix("Bearer "))
		.ok_or_else(|| {
			HttpResponse::Unauthorized()
				.json(ErrorResponse { error: "Missing or invalid token".to_string() })
		})?;

	let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not found in .env");
	let claims = decode::<Claims>(
		auth_header,
		&DecodingKey::from_secret(jwt_secret.as_bytes()),
		&Validation::new(Algorithm::HS256),
	)
	.map_err(|e| {
		log::error!("Token validation error: {:?}", e);
		HttpResponse::Unauthorized().json(ErrorResponse { error: "Invalid token".to_string() })
	})?;

	Ok(claims.claims.sub) // Return the user ID from claims
}
