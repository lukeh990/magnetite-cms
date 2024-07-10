/*
 * database.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use tokio::sync::{mpsc, oneshot};
use color_eyre::eyre::Result;

mod schema;

/// DatabaseMpscCommand
/// This enum contains all possible commands that can be issued to the database
///
/// List of commands:
/// GetPage(path, skip_cache)
pub enum DatabaseMpscCommand {
    GetPage(String, bool, DatabaseOneshotReply<schema::Page>),
}

pub type DatabaseOneshotReply<T> = oneshot::Sender<Result<T>>;

pub struct Database {
    tx: mpsc::Sender<DatabaseMpscCommand>,
}

impl Database {
    pub async fn get_page(&self, path: String, skip_cache: bool) -> Result<schema::Page> {
        let (tx, rx) = oneshot::channel::<Result<schema::Page>>();
        
        self.tx.send(DatabaseMpscCommand::GetPage(path, skip_cache, tx)).await?;

        rx.await?
    }
}

/// init_db() -> Database
/// Initilize DB connection and return struct.
pub fn init_db() -> Database {
    // No idea if 10 is a big enough channel. Remember to change.
    let (tx, rx) = mpsc::channel::<DatabaseMpscCommand>(10);

    Database {
        tx
    }
}
