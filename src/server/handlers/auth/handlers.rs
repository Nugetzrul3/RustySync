use std::sync::Mutex;
use actix_web::{web, Responder};
use argon2::PasswordHash;
use rusqlite::Connection;
use crate::shared::models::{
    AuthRequest,
    RefreshRequest,
    UserAccessToken,
    UserRefreshToken
};
use crate::server::db;
use crate::shared::utils;
use jsonwebtoken::{self, Header, EncodingKey, DecodingKey, Validation};
use serde_json::json;

const ACCESS_TOKEN_EXPIRY_SECONDS: usize = 60 * 15; // 15 minutes
const REFRESH_TOKEN_EXPIRY_SECONDS: usize = 60 * 60 * 24 * 7; // 7 days

pub async fn register(payload: web::Json<AuthRequest>, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let (username, password) = match utils::extract_user_info(&payload.0) {
        Ok((password, username)) => (password, username),
        Err(e) => {
            eprintln!("Error with authentication");
            return utils::bad_request_error(e.to_string());
        }

    };

    let users = match db::find_user(&conn, &username) {
        Ok(u) => u,
        Err(e) => return utils::bad_request_error(e.to_string()),
    };

    if users.len() > 0 {
        return utils::conflict_error(String::from("User already exists"));
    }

    match db::register_user(&conn, &username, &password) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            return utils::internal_server_error(e.to_string());
        }
    }

    utils::okay_response(None)

}

pub async fn login(payload: web::Json<AuthRequest>, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let (username, password) = match utils::extract_user_info(&payload.0) {
        Ok((password, username)) => (password, username),
        Err(e) => {
            eprintln!("Error with authentication");
            return utils::bad_request_error(e.to_string());
        }

    };

    // user lookup

    let users = match db::find_user(&conn, &username) {
        Ok(u) => u,
        Err(e) => return utils::bad_request_error(e.to_string()),
    };

    if users.len() == 0 {
        return utils::not_found_error(String::from("User not found"));
    }

    // Extract hashed password

    let user = users.get(0).unwrap();
    let user_password_hash = match PasswordHash::new(user.password()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error extracting hashed password");
            return utils::internal_server_error(e.to_string());
        },
    };

    if !utils::check_password(&password, &user_password_hash) {
        return utils::bad_request_error(String::from("Invalid password"));
    }

    let now = jsonwebtoken::get_current_timestamp() as usize;

    let access_token_claim = UserAccessToken::new(
        username.clone(),
        now + ACCESS_TOKEN_EXPIRY_SECONDS,
    );

    let refresh_token_claim = UserRefreshToken::new(
        username.clone(),
        now + REFRESH_TOKEN_EXPIRY_SECONDS,
    );

    let jwt_secret = std::env::var("JWT_SECRET").unwrap().into_bytes();

    let access_token = match jsonwebtoken::encode(
        &Header::default(),
        &access_token_claim,
        &EncodingKey::from_secret(&jwt_secret)
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error with access token generation {}", e);
            return utils::internal_server_error(e.to_string());
        }
    };

    let refresh_token = match jsonwebtoken::encode(
        &Header::default(),
        &refresh_token_claim,
        &EncodingKey::from_secret(&jwt_secret)
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error with refresh token generation {}", e);
            return utils::internal_server_error(e.to_string());
        }
    };

    utils::okay_response(Some(
        json!(
            {
                "access_token": access_token,
                "refresh_token": refresh_token,
                "token_type": "bearer"
            }
        )
    ))

}

pub async fn refresh(payload: web::Json<RefreshRequest>, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let refresh_req = payload.0;
    let refresh_user = match jsonwebtoken::decode::<UserRefreshToken>(
        refresh_req.refresh_token(),
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap_or("JWT_SECRET".to_string()).as_ref()),
        &Validation::default(),
    ) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("JSON Web token authentication failed, {}", e);
            return utils::authorization_error(e.to_string());
        }
    };

    let username = refresh_user.claims.sub;

    // check user existence
    let users = match db::find_user(&conn, &username) {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Database Error, {}", e);
            return utils::internal_server_error(e.to_string());
        }
    };

    if users.len() == 0 {
        return utils::not_found_error(String::from("User not found"));
    }

    // Generate new access token
    let now = jsonwebtoken::get_current_timestamp() as usize;

    let access_token_claim = UserAccessToken::new(
        username.clone(),
        now + ACCESS_TOKEN_EXPIRY_SECONDS,
    );

    let access_token = match jsonwebtoken::encode(
        &Header::default(),
        &access_token_claim,
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap_or("JWT_SECRET".to_string()).as_ref())
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error generating access token");
            return utils::internal_server_error(e.to_string());
        }
    };

    utils::okay_response(Some(
        json!(
            {
                "username": username,
                "access_token": access_token,
                "token_type": "bearer"
            }
        )
    ))
}
