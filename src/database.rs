/*
 * database.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use super::util::println;
use color_eyre::Result;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

mod cache;
mod process;
mod schema;

// This enum contains all possible commands that can be issued to the database
pub enum DatabaseMpscCommand {
    // GetPage(path, skip_cache, reply)
    // -> Result<schema::Page>
    GetPage(String, bool, DatabaseOneshotReply<schema::Page>),

    // SetPage(new_page, reply)
    // -> Result<()>
    SetPage(schema::Page, DatabaseOneshotReply<()>),

    // DeletePage(path, reply)
    // -> Result<()>
    DeletePage(String, DatabaseOneshotReply<()>),

    // NewPage(new_page, reply)
    // -> Result<()>
    NewPage(schema::Page, DatabaseOneshotReply<()>),

    // GetUser(id, skip_cache, reply)
    // -> Result<schema::AdminUser>
    GetUser(Uuid, bool, DatabaseOneshotReply<schema::AdminUser>),

    // SetUser(new_user, reply)
    // -> Result<()>
    SetUser(schema::AdminUser, DatabaseOneshotReply<()>),

    // DeleteUser(id, reply)
    // -> Result<()>
    DeleteUser(Uuid, DatabaseOneshotReply<()>),

    // NewUser(new_user, reply)
    // -> Result<()>
    NewUser(schema::AdminUser, DatabaseOneshotReply<()>),
}

pub type DatabaseOneshotReply<T> = oneshot::Sender<Result<T>>;

#[derive(Debug, Clone)]
pub struct Database {
    tx: mpsc::Sender<DatabaseMpscCommand>,
}

impl Database {
    pub async fn get_page<S>(&self, path: S, skip_cache: bool) -> Result<schema::Page>
    where
        S: Into<String>,
    {
        let (tx, rx) = oneshot::channel::<Result<schema::Page>>();

        self.tx
            .send(DatabaseMpscCommand::GetPage(path.into(), skip_cache, tx))
            .await?;

        rx.await?
    }

    pub async fn set_page(&self, new_page: schema::Page) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx
            .send(DatabaseMpscCommand::SetPage(new_page, tx))
            .await?;

        rx.await?
    }

    pub async fn delete_page<S>(&self, path: S) -> Result<()>
    where
        S: Into<String>,
    {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx
            .send(DatabaseMpscCommand::DeletePage(path.into(), tx))
            .await?;

        rx.await?
    }

    pub async fn new_page(&self, new_page: schema::Page) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx
            .send(DatabaseMpscCommand::NewPage(new_page, tx))
            .await?;

        rx.await?
    }

    pub async fn get_user(&self, id: Uuid, skip_cache: bool) -> Result<schema::AdminUser> {
        let (tx, rx) = oneshot::channel::<Result<schema::AdminUser>>();

        self.tx
            .send(DatabaseMpscCommand::GetUser(id, skip_cache, tx))
            .await?;

        rx.await?
    }

    pub async fn set_user(&self, new_user: schema::AdminUser) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx
            .send(DatabaseMpscCommand::SetUser(new_user, tx))
            .await?;

        rx.await?
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx
            .send(DatabaseMpscCommand::DeleteUser(id, tx))
            .await?;

        rx.await?
    }

    pub async fn new_user(&self, new_user: schema::AdminUser) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx
            .send(DatabaseMpscCommand::NewUser(new_user, tx))
            .await?;

        rx.await?
    }

    pub async fn new(database_url: String) -> Result<Database> {
        // No idea if 10 is a big enough channel. Remember to change.
        let (tx, mut rx) = mpsc::channel::<DatabaseMpscCommand>(10);

        let pool = PgPoolOptions::new().connect(&database_url).await?;

        // run migrations
        println::info("Running DB Migrations");
        sqlx::migrate!("./migrations").run(&pool).await?;

        tokio::spawn(async move {
            let mut cache = cache::Cache::new().await;
            loop {
                match rx.try_recv() {
                    Ok(cmd) => {
                        process::cmd(cmd, &pool, &mut cache).await;
                    }
                    Err(err) => {
                        if err == TryRecvError::Disconnected {
                            // Throw error message and stop loop
                            println::error("All transmitters have been disconnected. Exiting...");
                            cache.close().await;
                            break;
                        }
                    }
                }
            }
        });

        Ok(Database { tx })
    }
}
