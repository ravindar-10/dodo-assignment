use crate::handler::UserHandler;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
	cfg
		// user mgmt routes
		.route("/profile", web::put().to(UserHandler::update_profile_handler))
		.route("/register", web::post().to(UserHandler::send_otp_handler))
		.route("/login", web::post().to(UserHandler::verify_otp_handler));
}
