use actix_web::{test, web, App};
use lettre::transport::smtp::response;
use serde_json::json;

use crate::{
	db::get_db_pool,
	handler::UserHandler,
	models::{RegisterRequest, VerifyOtpRequest},
};

#[actix_web::test]
async fn test_send_otp_handler() {
	let pool = get_db_pool();
	// let pool = setup_test_db();
	// Setup test app
	let app =
		test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(
			web::resource("/register").route(web::post().to(UserHandler::send_otp_handler)),
		))
		.await;

	// Create test data
	let request_body = RegisterRequest {
		email: "work.ravindar@gmail.com".to_string(),
		password: Some("testpassword123".to_string()),
	};

	// Make test request
	let resp = test::TestRequest::post()
		.uri("/register")
		.set_json(&request_body)
		.send_request(&app)
		.await;
	println!("response = {:?}", resp);
	// Assert response
	assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_verify_otp_handler() {
	let pool = get_db_pool();
	// Setup test app
	let app =
		test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(
			web::resource("/login").route(web::post().to(UserHandler::verify_otp_handler)),
		))
		.await;

	// Create test data
	let request_body = VerifyOtpRequest {
		email: "work.ravindar@gmail.com".to_string(),
		otp: 920548,
		password: None,
	};

	// Make test request
	let resp = test::TestRequest::post()
		.uri("/login")
		.set_json(&request_body)
		.send_request(&app)
		.await;

	// Assert response
	assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_invalid_otp() {
	let pool = get_db_pool();
	// Setup test app
	let app =
		test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(
			web::resource("/login").route(web::post().to(UserHandler::verify_otp_handler)),
		))
		.await;

	// Create test data with invalid OTP .app_data(web::Data::new(pool.clone()))
	let request_body = VerifyOtpRequest {
		email: "work.ravindar@gmail.com".to_string(),
		otp: 000000,
		password: None,
	};

	// Make test request
	let resp = test::TestRequest::post()
		.uri("/login")
		.set_json(&request_body)
		.send_request(&app)
		.await;

	// Assert response
	assert!(resp.status().is_client_error());
}

// #[actix_web::test]
// async fn test_invalid_email_format() {
// 	let app =
// 		test::init_service(App::new().service(
// 			web::resource("/register").route(web::post().to(UserHandler::send_otp_handler)),
// 		))
// 		.await;

// 	let request_body = RegisterRequest {
// 		email: "invalid_email".to_string(),
// 		password: Some("testpassword123".to_string()),
// 	};

// 	let resp = test::TestRequest::post()
// 		.uri("/register")
// 		.set_json(&request_body)
// 		.send_request(&app)
// 		.await;

// 	assert!(resp.status().is_client_error());
// }

// #[actix_web::test]
// async fn test_weak_password() {
// 	let app =
// 		test::init_service(App::new().service(
// 			web::resource("/register").route(web::post().to(UserHandler::send_otp_handler)),
// 		))
// 		.await;

// 	let request_body = RegisterRequest {
// 		email: "test@example.com".to_string(),
// 		password: Some("weak".to_string()), // Too short password
// 	};

// 	let resp = test::TestRequest::post()
// 		.uri("/register")
// 		.set_json(&request_body)
// 		.send_request(&app)
// 		.await;

// 	assert!(resp.status().is_client_error());
// }

// #[actix_web::test]
// async fn test_duplicate_email() {
// 	let app =
// 		test::init_service(App::new().service(
// 			web::resource("/register").route(web::post().to(UserHandler::send_otp_handler)),
// 		))
// 		.await;

// 	// First registration
// 	let request_body = RegisterRequest {
// 		email: "duplicate@example.com".to_string(),
// 		password: Some("testpassword123".to_string()),
// 	};

// 	let _ = test::TestRequest::post()
// 		.uri("/register")
// 		.set_json(&request_body)
// 		.send_request(&app)
// 		.await;

// 	// Second registration with same email
// 	let resp = test::TestRequest::post()
// 		.uri("/register")
// 		.set_json(&request_body)
// 		.send_request(&app)
// 		.await;

// 	assert!(resp.status().is_client_error());
// }
