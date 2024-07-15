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

use std::env;
use color_eyre::Result;
use util::println;

mod database;
mod web;
mod util;

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
    
    // Load Plugins
    println::warn("Plugin Functionality is still being implemented");

    // Setup database thread
    println::info("Initializing DB");
    let db = database::Database::new(database_url).await?;
    println::info("Sucessfully connected to DB");

    // Setup poem thread
    println::info("Starting HTTP Server");
    web::start_server(server_bind, db).await?;

    // Shutdown
    println::error("Magnetite CMS Server has stopped.");
    Ok(())
}
