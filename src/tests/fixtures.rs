use bigdecimal::BigDecimal;

use crate::models::{RegisterRequest, Transaction};

pub struct TestFixtures;

impl TestFixtures {
	pub fn valid_user_registration() -> RegisterRequest {
		RegisterRequest {
			email: "test.user@gmail.com".to_string(),
			password: Some("SecurePassword123!".to_string()),
		}
	}

	pub fn credit_transaction(amount: f64) -> Transaction {
		Transaction {
			user_id: 12,
			amount: BigDecimal::from(amount as i64), // Negative amount should be invalid
			transaction_type: "credit".to_string(),
			description: "Test credit transaction".to_string(),
			created_at: Some(chrono::Utc::now()),
		}
	}

	pub fn debit_transaction(amount: f64) -> Transaction {
		Transaction {
			user_id: 12,
			amount: BigDecimal::from(amount as i64), // Negative amount should be invalid
			transaction_type: "debit".to_string(),
			description: "Test credit transaction".to_string(),
			created_at: Some(chrono::Utc::now()),
		}
	}
}
