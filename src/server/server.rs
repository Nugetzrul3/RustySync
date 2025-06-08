// Main logic for hosting Actix-Web HTTP server
use actix_web::{ web, App, HttpServer, HttpResponse, Responder };
use serde_json::json;

// basic server health check route
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(
        json!({
            "status": "ok",
        })
    )
}
// main server startup
pub async fn start(port: u16) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
    })
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
