/*
 * web/html.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use crate::database::schema;
use actix_web::{http::header::ContentType, HttpResponse};

pub async fn page_to_response(page: schema::Page) -> HttpResponse {
    let metadata = page.metadata.join("\n");
    let html_string = format!(
        "
    <!DOCTYPE html>
    <html>
        <head>
            {}
        </head>
        <body>
            {}
        </body>
    </html>
    ",
        metadata, page.body
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_string)
}
