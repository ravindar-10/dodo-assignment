use crate::{
	db::DbPool,
	midware::jwt::JWT,
	models::{
		self, ApiResponse, Balance, ErrorResponse, LoginResp, NewTransaction, NewUser, Otp,
		ProfileUpdate, RegisterRequest, VerifyOtpRequest,
	},
	repo::{authenticate, UserRepo},
	schema::{
		balances, otps,
		transactions::{self},
		users,
	},
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;
#[derive(Debug, Serialize, Deserialize)]
pub struct UserHandler {}

impl UserHandler {
	pub async fn send_otp_handler(
		pool: web::Data<DbPool>,
		req: web::Json<RegisterRequest>,
	) -> impl Responder {
		log::info!("Attempting to send OTP for email: {}", req.email);
		let mut conn = pool
			.get()
			.map_err(|e| {
				log::error!("Database connection error: {:?}", e);
				HttpResponse::InternalServerError().json(ApiResponse::<String> {
					status: "error".to_string(),
					data: None,
					error: Some("Database connection error".to_string()),
				})
			})
			.expect("Failed to get DB connection");

		let user_id_res = users::dsl::users
			.filter(users::dsl::email.eq(&req.email))
			.select(users::dsl::id)
			.first::<i32>(&mut conn)
			.map_err(|_e| {
				log::info!("No existing user found for email: {}", req.email);
				HttpResponse::NotFound().json(ApiResponse::<String> {
					status: "error".to_string(),
					data: None,
					error: Some("User Not Found".to_string()),
				})
			});

		let new_user_id = match user_id_res {
			Ok(id) => {
				log::info!("Found existing user with ID: {} for email: {}", id, req.email);
				id
			},
			_ => {
				log::info!("No existing user found for email: {}, will create new user", req.email);
				0
			},
		};

		let existing_active_user = otps::dsl::otps
			.filter(otps::dsl::user_id.eq(new_user_id))
			.filter(otps::dsl::is_valid.eq(&true))
			.first::<Otp>(&mut conn)
			.optional()
			.map_err(|e| {
				log::error!("Database error checking existing OTP: {:?}", e);
				HttpResponse::InternalServerError().json(ApiResponse::<String> {
					status: "error".to_string(),
					data: None,
					error: Some("Database error".to_string()),
				})
			})
			.expect("failed to check existing OTP");

		if let Some(otp) = existing_active_user {
			if otp.is_valid.unwrap_or(false) {
				log::warn!("Active OTP already exists for email: {}", req.email);
				return HttpResponse::BadRequest().json(ApiResponse::<String> {
					status: "error".to_string(),
					data: None,
					error: Some("Active OTP already exists".to_string()),
				});
			}
		}

		let password = req.password.as_deref().unwrap_or("test-password");
		let hashed_password = hash(password.as_bytes(), DEFAULT_COST)
			.map_err(|e| {
				log::error!("Password hashing failed for email {}: {}", req.email, e);
				HttpResponse::InternalServerError().json(ApiResponse::<String> {
					status: "error".to_string(),
					data: None,
					error: Some("Failed to hash password".to_string()),
				})
			})
			.expect("Failed to hash password");

		let user_otp = UserRepo::generate_otp();
		log::info!("Generated OTP for email: {}", req.email);

		let new_user = NewUser { email: &req.email, password: &hashed_password };

		if let Err(e) = UserRepo::store_otp_in_db(pool, &req.email, new_user, &user_otp).await {
			log::error!("Failed to store OTP in DB for email {}: {}", req.email, e);
			return HttpResponse::InternalServerError().json(ApiResponse::<String> {
				status: "error".to_string(),
				data: None,
				error: Some("Failed to store OTP".to_string()),
			});
		}

		match UserRepo::send_otp_email(&req.email, &user_otp) {
			Ok(_) => {
				log::info!("OTP sent successfully to email: {}", req.email);
				HttpResponse::Ok().json(ApiResponse::<String> {
					status: "success".to_string(),
					data: Some("OTP sent and stored successfully".to_string()),
					error: None,
				})
			},
			Err(e) => {
				log::error!("Failed to send OTP email: {:?}", e);
				HttpResponse::InternalServerError().json(ApiResponse::<String> {
					status: "error".to_string(),
					data: None,
					error: Some("Failed to send OTP email".to_string()),
				})
			},
		}
	}

	pub async fn verify_otp_handler(
		conn: web::Data<DbPool>,
		req: web::Json<VerifyOtpRequest>,
	) -> impl Responder {
		match UserRepo::verify_otp_in_db(conn.clone(), &req.email, &req.otp) {
			Ok(_) => {
				let jwt_secret =
					std::env::var("JWT_SECRET").expect("JWT_SECRET can not be found in .env");
				let jwt_instance = JWT::new(&jwt_secret);

				let mut conn = match conn.get() {
					Ok(conn) => conn,
					Err(e) => {
						log::error!("DB connection error: {:?}", e);
						return HttpResponse::InternalServerError().json(ApiResponse::<LoginResp> {
							status: "error".to_string(),
							data: None,
							error: Some("Database error".to_string()),
						});
					},
				};

				let password = req.password.clone().unwrap_or_default();
				let _hashed_password = match hash(password.as_bytes(), DEFAULT_COST) {
					Ok(hash) => hash,
					Err(e) => {
						log::error!("Failed to hash password: {}", e);
						return HttpResponse::InternalServerError().json(ApiResponse::<LoginResp> {
							status: "error".to_string(),
							data: None,
							error: Some("Failed to hash password".to_string()),
						});
					},
				};

				// Get user ID from users table
				let user_id = users::dsl::users
					.filter(users::dsl::email.eq(&req.email.clone()))
					.select(users::dsl::id)
					.first::<i32>(&mut conn)
					.map_err(|e| {
						log::error!("User fetch error: {:?}", e);
						HttpResponse::NotFound().json(ApiResponse::<LoginResp> {
							status: "error".to_string(),
							data: None,
							error: Some("User not found".to_string()),
						})
					});

				let user_id = match user_id {
					Ok(id) => id,
					Err(res) => return res,
				};

				match jwt_instance.create_jwt(user_id.to_string()) {
					Ok(token) => {
						log::info!("Login successful for user ID: {}", user_id);
						HttpResponse::Ok().json(ApiResponse::<LoginResp> {
							status: "success".to_string(),
							data: Some(LoginResp { token, uid: user_id.to_string() }),
							error: None,
						})
					},
					Err(e) => {
						log::error!("JWT creation error: {:?}", e);
						HttpResponse::InternalServerError().json(ApiResponse::<LoginResp> {
							status: "error".to_string(),
							data: None,
							error: Some("Failed to create authentication token".to_string()),
						})
					},
				}
			},
			Err(err) => {
				log::error!("Error verifying OTP: {:?}", err);
				HttpResponse::BadRequest().json(ApiResponse::<LoginResp> {
					status: "error".to_string(),
					data: None,
					error: Some("Failed to verify OTP".to_string()),
				})
			},
		}
	}

	pub async fn update_profile_handler(
		pool: web::Data<DbPool>,
		req: web::Json<ProfileUpdate>,
		http_req: actix_web::HttpRequest,
	) -> impl Responder {
		let user_id = match authenticate(&http_req).await {
			Ok(id) => id,
			Err(resp) => return resp,
		}
		.parse::<i32>()
		.unwrap_or(0);

		if let Err(e) = req.validate() {
			log::error!("Validation error: {:?}", e);
			return HttpResponse::BadRequest().json(ApiResponse::<models::User> {
				status: "error".to_string(),
				data: None,
				error: Some(e.to_string()),
			});
		}

		let mut conn = match pool.get() {
			Ok(conn) => conn,
			Err(e) => {
				log::error!("DB connection error: {:?}", e);
				return HttpResponse::InternalServerError().json(ApiResponse::<models::User> {
					status: "error".to_string(),
					data: None,
					error: Some("Database error".to_string()),
				});
			},
		};

		let user = diesel::update(users::dsl::users.filter(users::dsl::id.eq(&user_id)))
			.set((
				req.username.as_ref().map(|u| users::dsl::username.eq(u)),
				req.email.as_ref().map(|e| users::dsl::email.eq(e)),
			))
			.returning((
				users::dsl::id,
				users::dsl::email,
				users::dsl::username,
				users::dsl::password,
			))
			.get_result::<models::User>(&mut conn)
			.map_err(|e| {
				log::error!("Profile update error: {:?}", e);
				HttpResponse::InternalServerError().json(ApiResponse::<models::User> {
					status: "error".to_string(),
					data: None,
					error: Some("Failed to update profile".to_string()),
				})
			});

		match user {
			Ok(user) => {
				log::info!("Profile updated successfully for user: {}", user_id);
				HttpResponse::Ok().json(ApiResponse::<models::User> {
					status: "success".to_string(),
					data: Some(user),
					error: None,
				})
			},
			Err(res) => res,
		}
	}
}
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TransactionHandler {}
impl TransactionHandler {
	pub async fn create_transaction_handler(
		pool: web::Data<DbPool>,
		req: web::Json<models::Transaction>,
		http_req: actix_web::HttpRequest,
	) -> impl Responder {
		let mut conn = match pool.get() {
			Ok(conn) => conn,
			Err(e) => {
				log::error!("DB connection error: {:?}", e);
				return HttpResponse::InternalServerError().json(ApiResponse::<NewTransaction> {
					status: "error".to_string(),
					data: None,
					error: Some("Database error".to_string()),
				});
			},
		};

		let user_id = match authenticate(&http_req).await {
			Ok(id) => id,
			Err(resp) => return resp,
		}
		.parse::<i32>()
		.unwrap_or(0);
		if user_id != req.user_id {
			log::error!("User ID mismatch: {} != {}", user_id, req.user_id);
			return HttpResponse::Forbidden().json(ApiResponse::<NewTransaction> {
				status: "error".to_string(),
				data: None,
				error: Some("User ID mismatch".to_string()),
			});
		}
		let amount = req.amount.clone();
		if amount <= BigDecimal::from(0) {
			log::error!("Invalid transaction amaount: {}", amount);
			return HttpResponse::BadRequest().json(ApiResponse::<NewTransaction> {
				status: "error".to_string(),
				data: None,
				error: Some("Invalid transaction amount".to_string()),
			})
		}
		let transaction_id = uuid::Uuid::new_v4();
		log::info!("Generated transaction ID: {} for user: {}", transaction_id, user_id);

		let new_transaction = NewTransaction {
			id: transaction_id,
			user_id: req.user_id,
			amount: amount.clone(),
			description: req.description.clone(),
			transaction_type: req.transaction_type.clone(),
			created_at: Some(chrono::Utc::now()),
		};

		let transaction = diesel::insert_into(transactions::table)
			.values(&new_transaction)
			.get_result::<NewTransaction>(&mut conn)
			.map_err(|e| {
				log::error!("Transaction creation error: {:?}", e);
				HttpResponse::InternalServerError().json(ApiResponse::<NewTransaction> {
					status: "error".to_string(),
					data: None,
					error: Some("Failed to create transaction".to_string()),
				})
			});

		let new_transactions = match transaction {
			Ok(transaction) => transaction,
			Err(e) => return e,
		};

		if new_transactions.transaction_type.to_lowercase() != "credit" &&
			new_transactions.transaction_type.to_lowercase() != "debit"
		{
			log::error!("Invalid transaction type: {}", new_transactions.transaction_type);
			return HttpResponse::BadRequest().json(ApiResponse::<NewTransaction> {
				status: "error".to_string(),
				data: None,
				error: Some("Invalid transaction type".to_string()),
			});
		}

		let current_balance = balances::dsl::balances
			.filter(balances::dsl::user_id.eq(req.user_id))
			.select(balances::dsl::balance)
			.first::<BigDecimal>(&mut conn)
			.map_err(|e| {
				log::error!("Balance fetch error: {:?}", e);
				HttpResponse::NotFound().json(ApiResponse::<NewTransaction> {
					status: "error".to_string(),
					data: None,
					error: Some("Balance not found".to_string()),
				})
			})
			.unwrap_or(BigDecimal::from(0));

		let amount = if new_transactions.transaction_type.to_lowercase() == "debit" {
			-amount
		} else {
			amount
		};
		println!("Current balance: {}, Transaction amount: {}", current_balance, amount);
		if current_balance + amount.clone() < BigDecimal::from(0) {
			log::error!("Insufficient balance for user: {}", req.user_id);
			return HttpResponse::BadRequest().json(ApiResponse::<NewTransaction> {
				status: "error".to_string(),
				data: None,
				error: Some("Insufficient balance".to_string()),
			});
		}
		let balance_update = diesel::insert_into(balances::table)
			.values(&models::Balance { user_id: req.user_id, balance: amount.clone() })
			.on_conflict(balances::user_id)
			.do_update()
			.set(balances::balance.eq(balances::balance + amount))
			.execute(&mut conn)
			.map_err(|e| {
				log::error!("Balance update error: {:?}", e);
				HttpResponse::InternalServerError().json(ApiResponse::<NewTransaction> {
					status: "error".to_string(),
					data: None,
					error: Some("Failed to update balance".to_string()),
				})
			});

		if let Err(e) = balance_update {
			return e;
		}

		log::info!("Transaction created successfully for user: {}", req.user_id);
		HttpResponse::Created().json(ApiResponse::<NewTransaction> {
			status: "success".to_string(),
			data: Some(new_transactions),
			error: None,
		})
	}

	pub async fn get_transaction_handler(
		pool: web::Data<DbPool>,
		path: web::Path<String>,
		http_req: HttpRequest,
	) -> impl Responder {
		log::info!("Starting transaction retrieval process");

		let user_id = match authenticate(&http_req).await {
			Ok(id) => {
				log::info!("User authenticated successfully: {}", id);
				id.parse::<i32>().unwrap_or(0)
			},
			Err(resp) => return resp,
		};

		let transaction_id = match uuid::Uuid::parse_str(&path.into_inner()) {
			Ok(id) => {
				log::info!("Transaction ID parsed successfully: {}", id);
				id
			},
			Err(e) => {
				log::error!("Invalid transaction ID format: {:?}", e);
				return HttpResponse::BadRequest().json(ApiResponse::<NewTransaction> {
					status: "error".to_string(),
					data: None,
					error: Some("Invalid transaction ID format".to_string()),
				});
			},
		};

		let mut conn = match pool.get() {
			Ok(conn) => conn,
			Err(e) => {
				log::error!("Database connection error: {:?}", e);
				return HttpResponse::InternalServerError().json(ApiResponse::<NewTransaction> {
					status: "error".to_string(),
					data: None,
					error: Some("Database error".to_string()),
				});
			},
		};

		let transaction = match transactions::dsl::transactions
			.filter(transactions::dsl::id.eq(transaction_id))
			.filter(transactions::dsl::user_id.eq(user_id))
			.first::<NewTransaction>(&mut conn)
			.map_err(|e| {
				log::error!(
					"Failed to fetch transaction {} for user {}: {:?}",
					transaction_id,
					user_id,
					e
				);
				HttpResponse::NotFound().json(ApiResponse::<NewTransaction> {
					status: "error".to_string(),
					data: None,
					error: Some("Transaction not found".to_string()),
				})
			}) {
			Ok(transaction) => {
				log::info!(
					"Transaction {} retrieved successfully for user: {}",
					transaction_id,
					user_id
				);
				transaction
			},
			Err(e) => {
				log::error!("Error retrieving transaction {}: {:?}", transaction_id, e);
				return e;
			},
		};

		log::info!("Successfully retrieved transaction {} for user: {}", transaction_id, user_id);
		HttpResponse::Ok().json(ApiResponse::<NewTransaction> {
			status: "success".to_string(),
			data: Some(transaction),
			error: None,
		})
	}

	pub async fn list_transactions_handler(
		pool: web::Data<DbPool>,
		http_req: HttpRequest,
	) -> impl Responder {
		log::info!("Starting transaction list retrieval");

		let user_id = match authenticate(&http_req).await {
			Ok(id) => {
				log::info!("User authenticated successfully: {}", id);
				id.parse::<i32>().unwrap_or(0)
			},
			Err(resp) => return resp,
		};

		let mut conn = match pool.get() {
			Ok(conn) => conn,
			Err(e) => {
				log::error!("Database connection error: {:?}", e);
				return HttpResponse::InternalServerError().json(
					ApiResponse::<Vec<NewTransaction>> {
						status: "error".to_string(),
						data: None,
						error: Some("Database error".to_string()),
					},
				);
			},
		};

		let transactions = transactions::dsl::transactions
			.filter(transactions::dsl::user_id.eq(user_id))
			.order(transactions::dsl::created_at.desc())
			.load::<NewTransaction>(&mut conn)
			.map_err(|e| {
				log::error!("Failed to list transactions for user {}: {:?}", user_id, e);
				HttpResponse::InternalServerError().json(ApiResponse::<Vec<NewTransaction>> {
					status: "error".to_string(),
					data: None,
					error: Some("Failed to list transactions".to_string()),
				})
			})
			.expect("Failed to list transactions");

		log::info!(
			"Successfully retrieved {} transactions for user: {}",
			transactions.len(),
			user_id
		);
		HttpResponse::Ok().json(ApiResponse::<Vec<NewTransaction>> {
			status: "success".to_string(),
			data: Some(transactions),
			error: None,
		})
	}

	pub async fn get_balance_handler(
		pool: web::Data<DbPool>,
		http_req: HttpRequest,
	) -> impl Responder {
		log::info!("Starting balance retrieval");

		let user_id = match authenticate(&http_req).await {
			Ok(id) => {
				log::info!("User authenticated successfully: {}", id);
				id.parse::<i32>().unwrap_or(0)
			},
			Err(resp) => return resp,
		};

		let mut conn = match pool.get() {
			Ok(conn) => conn,
			Err(e) => {
				log::error!("Database connection error: {:?}", e);
				return HttpResponse::InternalServerError().json(ApiResponse::<Balance> {
					status: "error".to_string(),
					data: None,
					error: Some("Database error".to_string()),
				});
			},
		};

		let balance = balances::dsl::balances
			.filter(balances::dsl::user_id.eq(user_id))
			.first::<Balance>(&mut conn)
			.map_err(|e| {
				log::error!("Failed to fetch balance for user {}: {:?}", user_id, e);
				HttpResponse::NotFound().json(ApiResponse::<Balance> {
					status: "error".to_string(),
					data: None,
					error: Some("Balance not found".to_string()),
				})
			})
			.expect("Failed to fetch balance");

		log::info!("Successfully retrieved balance for user {}: {}", user_id, balance.balance);
		HttpResponse::Ok().json(ApiResponse::<Balance> {
			status: "success".to_string(),
			data: Some(balance),
			error: None,
		})
	}
}
