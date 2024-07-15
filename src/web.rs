/*
 * web.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use actix_http::{HttpService, Request, Response, StatusCode, Error};
use actix_server::Server;
use std::time::Duration;
use bytes::BytesMut;
use color_eyre::Result;
use futures_util::StreamExt as _;

use crate::util::println;

pub async fn start_server(bind: String) -> Result<()> {
    Server::build()
        .bind("magnnetite-cms", bind, || {
            HttpService::build()
                .client_request_timeout(Duration::from_secs(1))
                .client_disconnect_timeout(Duration::from_secs(1))
                // handles HTTP/1.1 and HTTP/2
                .finish(|mut req: Request| async move {
                    let mut body = BytesMut::new();
                    while let Some(item) = req.payload().next().await {
                        body.extend_from_slice(&item?);
                    }

                    println::info(format!("request body: {:?}", body));

                    let res = Response::build(StatusCode::OK)
                        .body(body);

                    Ok::<_, Error>(res)
                })
                .tcp()
        })?.run().await?;
    Ok(())
}
