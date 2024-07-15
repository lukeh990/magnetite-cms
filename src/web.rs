/*
 * web.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use poem::{get, handler, http::StatusCode, listener::TcpListener, middleware::AddData, web::{Data, Html, Path}, EndpointExt, Response, Route, Server, Result};
use crate::util::println;
use super::database::Database;

#[handler]
async fn index(Path(name): Path<String>, db: Data<&Database>) -> Result<Response> {
    let name = format!("/{}", name);
    let page = match db.get_page(name, false).await {
        Ok(page) => page,
        Err(err) => {
            return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(err.to_string()));
        }
    };
    dbg!(&page);
    Ok(Response::builder().body(format!("
        <!DOCTYPE html>
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                {}
            </body>
        </html>", page.body)))
}

pub async fn start_server(bind: String, db: Database) -> std::result::Result<(), std::io::Error> {
    let app = Route::new().at("/*path", get(index)).with(AddData::new(db.clone()));

    println::info(format!("Staring server on: {}", &bind));

    Server::new(TcpListener::bind(bind))
        .run(app)
        .await
}
