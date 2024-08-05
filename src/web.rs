/*
 * web.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use crate::database::Database;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use color_eyre::Result;

struct AppState {
    db: Database,
}

#[get("/admin")]
async fn admin() -> impl Responder {
    HttpResponse::Ok().body("Admin Page")
}

#[get("/{tail:.*}")]
async fn managed_pages(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let tail = match req.match_info().get("tail") {
        Some(value) => format!("/{}", value),
        None => return HttpResponse::BadRequest().body("No Tailing String"),
    };

    let page = match data.db.get_page(tail, false).await {
        Ok(page) => page,
        Err(err) => match err.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::RowNotFound) => {
                return HttpResponse::NotFound().body("404 Not Found")
            }
            _ => return HttpResponse::InternalServerError().body("500 Internal Server Error"),
        },
    };

    HttpResponse::Ok().body(format!("{:?}", page))
}

pub async fn start_server(bind: String, db: Database) -> Result<()> {
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db.clone() }))
            .service(admin)
            .service(managed_pages)
    })
    .bind(bind)?
    .run()
    .await?)
}
