use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	pub sub: String,
	pub exp: usize,
}

pub fn generate_test_token(user_id: i32) -> String {
	let claims = Claims {
		sub: user_id.to_string(),
		exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
	};

	encode(&Header::default(), &claims, &EncodingKey::from_secret("test_secret".as_ref())).unwrap()
}
