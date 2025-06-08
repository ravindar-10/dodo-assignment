use crate::constants::{AUTHORIZATION, EMPTY, IGNORE_ROUTES, MESSAGE_INVALID_TOKEN, ONE_WEEK};
use actix_service::forward_ready;
use actix_web::{
	body::EitherBody,
	dev::{Service, ServiceRequest, ServiceResponse, Transform},
	http::{
		header::{HeaderName, HeaderValue},
		Method,
	},
	Error as AxError, HttpMessage, HttpResponse,
};
use actix_web_httpauth::extractors::{
	bearer::{BearerAuth, Config},
	AuthenticationError,
};
use chrono::Utc;
use futures::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub iat: usize,
	pub exp: usize,
	pub sub: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseBody {
	message: String,
	data: String,
}

impl ResponseBody {
	fn new(m: &str, d: &str) -> Self {
		Self { message: String::from(m), data: String::from(d) }
	}
}

pub struct JWT {
	secret: String,
}

impl JWT {
	pub fn new(s: &str) -> Self {
		Self { secret: s.to_string() }
	}

	pub fn create_jwt(&self, user_id: String) -> Result<String, Error> {
		let now = (Utc::now().timestamp_nanos_opt().unwrap() / 1_000_000_000) as usize; // nano to sec
		let expiry = now + ONE_WEEK;
		let claims = Claims { iat: now, exp: expiry, sub: user_id };
		info!("Creating with: claims: {:?}, secret: {}", claims, self.secret);
		encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_ref()))
	}

	pub fn verify_jwt(&self, token: &str) -> Result<Claims, Error> {
		info!("Verifying with: token: {}, secret: {}", token, self.secret);
		decode::<Claims>(
			token,
			&DecodingKey::from_secret(self.secret.as_ref()),
			&Validation::default(),
		)
		.map(|data| {
			info!("## claims: {:?}", data.claims);
			data.claims
		})
	}

	pub fn _jwt_validator(
		&self,
		req: ServiceRequest,
		auth: BearerAuth,
	) -> Result<ServiceRequest, (AxError, ServiceRequest)> {
		if req.path() == "/register" {
			log::debug!("register req");
			return Ok(req);
		}

		log::debug!("req: {}", req.path());

		match self.verify_jwt(auth.token()) {
			Ok(claims) => {
				// Add claims to request extensions
				req.extensions_mut().insert(claims);
				Ok(req)
			},
			Err(_) => {
				let config = req
					.app_data::<Config>()
					.cloned()
					.unwrap_or_default()
					.scope("urn:example:channel=HBO&urn:example:rating=G,PG-13");

				Err((AuthenticationError::from(config).into(), req))
			},
		}
	}
}


pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = AxError>,
	S::Future: 'static,
	B: 'static,
{
	type Response = ServiceResponse<EitherBody<B>>;
	type Error = AxError;
	type InitError = ();
	type Transform = AuthenticationMiddleware<S>;
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		let jwt_secret =
			std::env::var("JWT_SECRET").expect("JWT_SECRET can be found in the .env file");
		let jwt = JWT::new(&jwt_secret);
		ok(AuthenticationMiddleware { jwt, service })
	}
}

pub struct AuthenticationMiddleware<S> {
	jwt: JWT,
	service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = AxError>,
	S::Future: 'static,
	B: 'static,
{
	type Response = ServiceResponse<EitherBody<B>>;
	type Error = AxError;
	type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	forward_ready!(service);

	fn call(&self, req: ServiceRequest) -> Self::Future {
		info!("## Req: {}", req.path());
		let mut authenticate_pass: bool = false;

		// Bypass some account routes
		let mut headers = req.headers().clone();
		headers.append(HeaderName::from_static("content-length"), HeaderValue::from_static("true"));
		if Method::OPTIONS == *req.method() {
			authenticate_pass = true;
		} else {
			for ignore_route in IGNORE_ROUTES.iter() {
				if req.path().starts_with(ignore_route) {
					authenticate_pass = true;
					break;
				}
			}
		}

		if !authenticate_pass {
			if let Some(authen_header) = req.headers().get(AUTHORIZATION) {
				if let Ok(authen_str) = authen_header.to_str() {
					if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer") {
						let token = authen_str[6..authen_str.len()].trim();
						let vres = self.jwt.verify_jwt(token);
						if vres.is_ok() {
							authenticate_pass = true;
						} else {
							error!("Invalid token: {:?}", vres.unwrap_err());
						}
					}
				}
			}
		}

		if !authenticate_pass {
			let (request, _pl) = req.into_parts();
			let response = HttpResponse::Unauthorized()
				.json(ResponseBody::new(MESSAGE_INVALID_TOKEN, EMPTY))
				.map_into_right_body();

			return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
		}

		let res = self.service.call(req);

		Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
	}
}
