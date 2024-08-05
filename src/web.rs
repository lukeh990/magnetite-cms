/*
 * web.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use crate::{database::Database, util::println};
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use color_eyre::Result;
use html::page_to_response;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod html;

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

    page_to_response(page).await
}

pub async fn start_server(
    bind: String,
    db: Database,
    tracker: &TaskTracker,
    cancel_token: CancellationToken,
) -> Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db.clone() }))
            .service(admin)
            .service(managed_pages)
    })
    .bind(bind)?
    .run();

    let clone_tracker = tracker.clone();

    tracker.spawn(async move {
        let server_handle = clone_tracker.spawn(server);

        cancel_token.cancelled().await;
        println::error("Webserver Cancellation Token Received...");
        server_handle.abort();
    });

    Ok(())
}
