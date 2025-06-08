use crate::handler::TransactionHandler;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
	cfg
		// user mgmt routes
		.route("/transactions", web::post().to(TransactionHandler::create_transaction_handler))
		.route("/transactions/{id}", web::get().to(TransactionHandler::get_transaction_handler))
		.route("/transactions", web::get().to(TransactionHandler::list_transactions_handler))
		// Account Balances
		.route("/balance", web::get().to(TransactionHandler::get_balance_handler));
}
