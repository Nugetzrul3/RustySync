use actix_web::{ web, Responder};
use crate::shared::models::RegisterRequest;

pub async fn register(payload: web::Json<RegisterRequest>) -> impl Responder {

}