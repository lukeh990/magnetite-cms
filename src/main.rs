/*
 * main.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 *
 * This is the main entrypoint of the magnetite server
 *
 * Responsibility:
 * - Set up logging
 * - Configure env variables
 * - Load Plugins
 * - Set up cache
 * - Set up HTTP Server
 * - Inactive until error then attempt restart
 */

use color_eyre::Result;
use std::env;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use util::println;

mod database;
mod util;
mod web;

async fn shutdown(cancel_token: CancellationToken, tracker: TaskTracker) {
    println::error("Shutdown Signal Received.");

    cancel_token.cancel();
    tracker.wait().await;
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()?;

    println::important("Magnetite CMS Server Starting...");

    // Pull environment variables
    let database_url = match env::var("DATABASE_URL") {
        Ok(var) => var,
        Err(_) => {
            println::error("Failed to get database_url. Cannot start.");
            return Ok(());
        }
    };

    let server_bind = match env::var("SERVER_BIND") {
        Ok(var) => var,
        Err(_) => {
            println::warn("Failed to get server bind. Using default 127.0.0.1:3000.");
            String::from("127.0.0.1:3000")
        }
    };

    // Cancellation Tokens
    let cancel_token = CancellationToken::new();

    let db_cancel_token = cancel_token.clone();
    let web_cancel_token = cancel_token.clone();

    let tracker = TaskTracker::new();

    // Load Plugins
    println::warn("Plugin Functionality is still being implemented");

    // Setup database thread
    println::info("Initializing DB");
    let db = database::Database::new(database_url, &tracker, db_cancel_token).await?;
    println::info("Sucessfully connected to DB");

    // Setup actix thread
    println::info(format!("Starting HTTP Server on {}", server_bind));
    web::start_server(server_bind, db, &tracker, web_cancel_token).await?;

    tracker.close();

    // Ctrl + C Handler
    match signal::ctrl_c().await {
        Ok(()) => {
            shutdown(cancel_token, tracker).await;
            Ok(())
        }
        Err(err) => {
            shutdown(cancel_token, tracker).await;
            Err(err.into())
        }
    }
}
