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

use color_eyre::eyre::Result;

mod database;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()?;

    println!("Magnetite CMS Server Starting...");

    // Pull environment variables
    
    // Load Plugins
    println!("Plugin Functionality is still being implemented");

    // Setup database thread
    database::init_db();

    // Setup poem thread

    println!("Magnetite CMS Server has stopped.");
    Ok(())
}
