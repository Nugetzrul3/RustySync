use std::sync::Mutex;
use actix_web::{web, Responder};
use rusqlite::Connection;
use crate::shared::models::RegisterRequest;
use crate::server::db;
use crate::shared::utils;

pub async fn register(payload: web::Json<RegisterRequest>, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let username = match payload.username.clone() {
        Some(u) => u,
        None => {
            return utils::bad_request_error(String::from(String::from("username is required")))
        }
    };

    // check if user exists

    let users = match db::find_user(&conn, &username) {
        Ok(u) => u,
        Err(e) => return utils::bad_request_error(e.to_string()),
    };

    if users.len() > 0 {
        return utils::bad_request_error(String::from("User already exists"));
    }

    let password = match payload.password.clone() {
        Some(p) => p,
        None => {
            return utils::bad_request_error(String::from(String::from("password is required")))
        }
    };

    match db::register_user(&conn, &username, &password) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            return utils::internal_server_error(e.to_string());
        }
    }

    utils::okay_response(None)

}