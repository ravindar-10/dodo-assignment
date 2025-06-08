use actix_web::{test, web, App};
use bigdecimal::BigDecimal;
use serde_json::json;

use crate::{
	db::get_db_pool,
	handler::{TransactionHandler, UserHandler},
	models::{ApiResponse, Balance, LoginResp},
	tests::fixtures::TestFixtures,
};

#[actix_web::test]
async fn test_complete_user_flow() {
	// Setup test app with mock database
	let db_pool = get_db_pool();

	let app = test::init_service(
		App::new()
			.app_data(web::Data::new(db_pool))
			.app_data(web::Data::new("your-secret-key".to_string()))
			.service(
				web::resource("/register").route(web::post().to(UserHandler::send_otp_handler)),
			)
			.service(web::resource("/login").route(web::post().to(UserHandler::verify_otp_handler)))
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

	// Step 1: Register user
	let register_resp = test::TestRequest::post()
		.uri("/register")
		.set_json(&TestFixtures::valid_user_registration())
		.send_request(&app)
		.await;

	// assert!(register_resp.status().is_success(), "Registration failed");

	// Step 2: Verify OTP
	let verify_resp = test::TestRequest::post()
		.uri("/login")
		.set_json(&json!({
			"email": TestFixtures::valid_user_registration().email,
			"otp": 761746 // Using test OTP
		}))
		.send_request(&app)
		.await;

	assert!(verify_resp.status().is_success(), "OTP verification failed");

	// Extract token from verification response
	let verify_body: ApiResponse<LoginResp> = test::read_body_json(verify_resp).await;

	let token = verify_body.data.expect("Token not found in response").token;

	// Step 3: Create a credit transaction
	let credit_resp = test::TestRequest::post()
		.uri("/transactions")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.set_json(&TestFixtures::credit_transaction(100.0))
		.send_request(&app)
		.await;

	assert!(credit_resp.status().is_success(), "Credit transaction failed");

	// Step 4: Check balance
	let balance_resp = test::TestRequest::get()
		.uri("/balance")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.send_request(&app)
		.await;

	assert!(balance_resp.status().is_success(), "Balance check failed");

	let balance_body: ApiResponse<Balance> = test::read_body_json(balance_resp).await;
	assert_eq!(balance_body.data.unwrap().balance, BigDecimal::from(100), "Balance is incorrect");

	// Step 5: Create a debit transaction
	let debit_resp = test::TestRequest::post()
		.uri("/transactions")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.set_json(&TestFixtures::debit_transaction(50.0))
		.send_request(&app)
		.await;

	assert!(debit_resp.status().is_success(), "Debit transaction failed");

	// Step 6: Verify final balance
	let final_balance_resp = test::TestRequest::get()
		.uri("/balance")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.send_request(&app)
		.await;

	assert!(final_balance_resp.status().is_success(), "Final balance check failed");

	let final_balance_body: ApiResponse<Balance> = test::read_body_json(final_balance_resp).await;
	assert_eq!(
		final_balance_body.data.unwrap().balance,
		BigDecimal::from(50),
		"Final balance is incorrect"
	);
}
