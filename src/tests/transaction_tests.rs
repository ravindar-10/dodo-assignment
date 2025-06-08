use crate::{
	db::get_db_pool,
	models::{ApiResponse, Balance},
	schema::balances::balance,
	tests::test_utils::generate_test_token,
};
use actix_web::{test, web, App};
use bigdecimal::BigDecimal;
use serde_json::json;
use uuid::Uuid;
static TOKEN:&str="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3NDkzNzg1NDAsImV4cCI6MTc0OTk4MzM0MCwic3ViIjoiMTIifQ.1WDaQj8F83W5XIB_PRntl_bd6rWVm8e5t7_FOAjrUak";

use crate::{
	handler::TransactionHandler,
	models::{NewTransaction, Transaction, TransactionType},
};

#[actix_web::test]
async fn test_create_transaction() {
	let pool = get_db_pool();
	let user_id = 12;
	// Setup test app
	let app = test::init_service(
		App::new().app_data(web::Data::new(pool.clone())).service(
			web::resource("/transactions")
				.route(web::post().to(TransactionHandler::create_transaction_handler)),
		),
	)
	.await;

	// Generate token
	// let token = generate_test_token(user_id);
	let token = TOKEN.to_string();
	// let token="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.
	// eyJpYXQiOjE3NDkzNzg1NDAsImV4cCI6MTc0OTk4MzM0MCwic3ViIjoiMTIifQ.
	// 1WDaQj8F83W5XIB_PRntl_bd6rWVm8e5t7_FOAjrUak".to_string();

	// Create test data
	let request_body = Transaction {
		user_id,
		amount: BigDecimal::from(100),
		description: "Test transaction".to_string(),
		transaction_type: "credit".to_string(),
		created_at: Some(chrono::Utc::now()),
	};

	// Make test request with proper JWT token
	let resp = test::TestRequest::post()
		.uri("/transactions")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.set_json(&request_body)
		.send_request(&app)
		.await;

	println!("response = {:?}", resp);
	assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_transactions() {
	let pool = get_db_pool();

	// Setup test app with database pool
	let app = test::init_service(
		App::new().app_data(web::Data::new(pool.clone())).service(
			web::resource("/transactions")
				.route(web::get().to(TransactionHandler::list_transactions_handler)),
		),
	)
	.await;

	// Generate token
	// let token = generate_test_token(user_id);
	let token = TOKEN.to_string();

	// Make test request with proper JWT token
	let resp = test::TestRequest::get()
		.uri("/transactions")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.send_request(&app)
		.await;
	assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_invalid_transaction_amount() {
	let pool = get_db_pool();
	let user_id = 12;

	// Setup test app with database pool
	let app = test::init_service(
		App::new().app_data(web::Data::new(pool.clone())).service(
			web::resource("/transactions")
				.route(web::post().to(TransactionHandler::create_transaction_handler)),
		),
	)
	.await;

	// Generate token
	// let token = generate_test_token(user_id);
	let token = TOKEN.to_string();

	// Create test data with invalid amount
	let request_body = Transaction {
		user_id,
		amount: BigDecimal::from(-100),
		transaction_type: "credit".to_string(),
		description: "Invalid transaction".to_string(),
		created_at: Some(chrono::Utc::now()),
	};

	let resp = test::TestRequest::post()
		.uri("/transactions")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.set_json(&request_body)
		.send_request(&app)
		.await;

	assert!(resp.status().is_client_error());
}

// Note: test_unauthorized_transaction stays the same since it tests the case without a token

#[actix_web::test]
async fn test_insufficient_balance() {
	let pool = get_db_pool();
	let user_id = 12;

	// Setup test app with database pool
	let app = test::init_service(
		App::new().app_data(web::Data::new(pool.clone())).service(
			web::resource("/transactions")
				.route(web::post().to(TransactionHandler::create_transaction_handler)),
		),
	)
	.await;

	// let token = generate_test_token(user_id);
	let token = TOKEN.to_string();

	// Create test data with amount larger than available balance
	let request_body = Transaction {
		user_id,
		amount: BigDecimal::from(100000),
		transaction_type: "debit".to_string(),
		description: "Large withdrawal".to_string(),
		created_at: Some(chrono::Utc::now()),
	};

	let resp = test::TestRequest::post()
		.uri("/transactions")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.set_json(&request_body)
		.send_request(&app)
		.await;

	assert_eq!(resp.status().as_u16(), 400);
}

#[actix_web::test]
async fn test_balance_update_after_transaction() {
	let pool = get_db_pool();
	let user_id = 12;

	// Setup test app with database pool
	let app = test::init_service(
		App::new()
			.app_data(web::Data::new(pool.clone()))
			.service(
				web::resource("/transactions")
					.route(web::post().to(TransactionHandler::create_transaction_handler)),
			)
			.service(
				web::resource("/balance")
					.route(web::get().to(TransactionHandler::get_balance_handler)),
			),
	)
	.await;

	// let token = generate_test_token(user_id);
	let token = TOKEN.to_string();
	let balance_resp = test::TestRequest::get()
		.uri("/balance")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.send_request(&app)
		.await;
	let balance_body: ApiResponse<Balance> = test::read_body_json(balance_resp).await;
	let balance_before = balance_body.data.unwrap().balance;

	// First, credit some amount
	let credit_request = Transaction {
		user_id,
		amount: BigDecimal::from(100),
		transaction_type: "credit".to_string(),
		description: "Add fund".to_string(),
		created_at: Some(chrono::Utc::now()),
	};

	let _ = test::TestRequest::post()
		.uri("/transactions")
		.insert_header(("Authorization", format!("Bearer {}", token.clone())))
		.set_json(&credit_request)
		.send_request(&app)
		.await;

	// Then check balance
	let balance_resp = test::TestRequest::get()
		.uri("/balance")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.send_request(&app)
		.await;
	let balance_body: ApiResponse<Balance> = test::read_body_json(balance_resp).await;
	let balance_after = balance_body.data.unwrap().balance;
	assert_eq!(balance_after, balance_before + BigDecimal::from(100));
}
