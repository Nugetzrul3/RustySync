use actix_web::{Error, FromRequest, HttpRequest};
use actix_web::dev::Payload;
use actix_web::error::InternalError;
use futures_util::future::{ready, Ready};
use crate::shared::models::UserAccessToken;
use crate::shared::utils;

// Actix extractor for Auth
pub struct AuthUser(pub UserAccessToken);

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match utils::validate_token(req) {
            Ok(claims) => ready(Ok(AuthUser(claims))),
            Err(_) => {
                ready(Err(InternalError::from_response(
                    "Unauthorized",
                    utils::authorization_error(String::from("Invalid token")),
                ).into()))
            }
        }
    }
}