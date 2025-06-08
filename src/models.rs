use crate::schema::*;
use bigdecimal::BigDecimal;
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Queryable, Serialize, Deserialize, Default, Debug, Selectable)]
#[diesel(table_name = otps)]
pub struct Otp {
	pub id: i32,
	pub user_id: i32,
	pub otp: String,
	pub is_valid: Option<bool>,
}
#[derive(Queryable, Serialize, Deserialize, Default, Debug, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Pg))]
pub struct User {
	pub id: i32,
	pub email: String,
	#[diesel(sql_type = Nullable<diesel::sql_types::Text>)]
	pub username: Option<String>,
	pub password: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = otps)]
pub struct NewOtp<'a> {
	pub user_id: i32,
	pub otp: &'a str,
	pub is_valid: bool,
}
#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
	pub email: &'a str,
	pub password: &'a str,
}
#[derive(Deserialize, Serialize)]
pub struct EmailRequest {
	pub user_email: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
	pub email: String,
	pub otp: i64,
	pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResp {
	pub uid: String,
	pub token: String,
}

#[derive(Debug, Deserialize, Serialize, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransaction {
	pub id: uuid::Uuid,
	pub user_id: i32,
	#[diesel(sql_type = Numeric)]
	pub amount: BigDecimal,
	pub description: String,
	pub created_at: Option<chrono::DateTime<chrono::Utc>>,
	pub transaction_type: String,
}

#[derive(Debug, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::balances)]
pub struct Balance {
	pub user_id: i32,
	#[diesel(sql_type = Numeric)]
	pub balance: BigDecimal,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
	#[validate(email)]
	pub email: String,
	#[validate(length(min = 8))]
	pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct ProfileUpdate {
	#[validate(length(min = 3, max = 50))]
	pub username: Option<String>,
	#[validate(email)]
	pub email: Option<String>,
}
#[derive(Serialize)]
pub struct ErrorResponse {
	pub error: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
	pub status: String,
	pub data: Option<T>,
	pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
	pub user_id: i32,
	pub amount: BigDecimal,
	pub description: String,
	pub transaction_type: String,
	pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum TransactionType {
	Credit,
	Debit,
}

impl TransactionType {
	pub fn as_str(&self) -> &str {
		match self {
			TransactionType::Credit => "credit",
			TransactionType::Debit => "debit",
		}
	}
}
