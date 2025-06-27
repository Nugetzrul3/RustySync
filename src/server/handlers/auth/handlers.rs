use std::sync::Mutex;
use actix_web::{web, Responder};
use argon2::PasswordHash;
use rusqlite::Connection;
use crate::shared::models::{
    AuthRequest,
    UserAccessToken,
    UserRefreshToken
};
use crate::server::db;
use crate::shared::utils;
use jsonwebtoken::{
    self,
    Header,
    EncodingKey,
};

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

    jsonwebtoken::encode(&Header::default(), &user, &EncodingKey::from_secret(password.as_ref())).unwrap();


}
