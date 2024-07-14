/*
 * database.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::error::TryRecvError;
use color_eyre::Result;

mod schema;

/// DatabaseMpscCommand
/// This enum contains all possible commands that can be issued to the database
///
/// List of commands:
/// GetPage(path, skip_cache, reply)
pub enum DatabaseMpscCommand {
    GetPage(String, bool, DatabaseOneshotReply<schema::Page>),
}

pub type DatabaseOneshotReply<T> = oneshot::Sender<Result<T>>;

pub struct Database {
    tx: mpsc::Sender<DatabaseMpscCommand>,
}

impl Database {
    pub async fn get_page<S>(&self, path: S, skip_cache: bool) -> Result<schema::Page> 
    where S: Into<String> {
        let (tx, rx) = oneshot::channel::<Result<schema::Page>>();

        self.tx.send(DatabaseMpscCommand::GetPage(path.into(), skip_cache, tx)).await?;

        rx.await?
    }
}

pub async fn process_cmd(cmd: DatabaseMpscCommand, pool: &PgPool) -> bool {
    match cmd {
        DatabaseMpscCommand::GetPage(path, skip_cache, reply) => {
            if skip_cache {
                let page: schema::Page = match sqlx::query_as("SELECT * FROM pages WHERE path = $1")
                    .bind(path)
                    .fetch_one(pool).await {
                        Ok(page) => page,
                        Err(err) => {
                            let _ = reply.send(Err(err.into()));
                            return true;
                        }
                    };
                let _ = reply.send(Ok(page));
            }
            true
        }
    }
}

/// init_db()
/// Initilize DB connection and return struct.
pub async fn init_db(database_url: String) -> Result<Database> {
    // No idea if 10 is a big enough channel. Remember to change.
    let (tx, mut rx) = mpsc::channel::<DatabaseMpscCommand>(10);

    let pool = PgPoolOptions::new().connect(&database_url).await?;

    tokio::spawn(async move {
        loop {
           match rx.try_recv() {
               Ok(cmd) => {
                   process_cmd(cmd, &pool).await;
               },
               Err(err) => {
                   match err {
                       TryRecvError::Empty => {
                           // Do cache cleanup and stuff
                       },
                       TryRecvError::Disconnected => {
                           // Throw error message and stop loop
                           eprintln!("All transmitters have been disconnected. Exiting...");
                           break;
                       }
                   }
               }
            }
        }
    });

    Ok(Database {
        tx
    })
}
