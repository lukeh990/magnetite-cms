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
use uuid::Uuid;

mod schema;
mod cache;

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
    NewUser(schema::AdminUser, DatabaseOneshotReply<()>)
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
    
    pub async fn set_page(&self, new_page: schema::Page) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx.send(DatabaseMpscCommand::SetPage(new_page, tx)).await?;

        rx.await?
    }

    pub async fn delete_page<S>(&self, path: S) -> Result<()>
    where S: Into<String>
    {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx.send(DatabaseMpscCommand::DeletePage(path.into(), tx)).await?;

        rx.await?
    }

    pub async fn new_page(&self, new_page: schema::Page) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx.send(DatabaseMpscCommand::NewPage(new_page, tx)).await?;

        rx.await?
    }

    pub async fn get_user(&self, id: Uuid, skip_cache: bool) -> Result<schema::AdminUser> {
        let (tx, rx) = oneshot::channel::<Result<schema::AdminUser>>();

        self.tx.send(DatabaseMpscCommand::GetUser(id, skip_cache, tx)).await?;

        rx.await?
    }

    pub async fn set_user(&self, new_user: schema::AdminUser) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx.send(DatabaseMpscCommand::SetUser(new_user, tx)).await?;

        rx.await?
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx.send(DatabaseMpscCommand::DeleteUser(id, tx)).await?;

        rx.await?
    }

    pub async fn new_user(&self, new_user: schema::AdminUser) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();

        self.tx.send(DatabaseMpscCommand::NewUser(new_user, tx)).await?;

        rx.await?
    }
}

pub async fn process_cmd(cmd: DatabaseMpscCommand, pool: &PgPool, cache: &mut cache::Cache) {
    match cmd {
        DatabaseMpscCommand::GetPage(path, skip_cache, reply) => {
            if !skip_cache {
                if let Some(result) = cache.get_page(&path).await {
                    let _ = reply.send(Ok(result));
                    return;
                }
            }

            let page = match sqlx::query_as!(schema::Page, 
                "SELECT * FROM pages WHERE path = $1", path)
                .fetch_one(pool).await {
                    Ok(page) => page,
                    Err(err) => {
                        let _ = reply.send(Err(err.into()));
                        return;
                    }
                };
            cache.set_page(&page).await;
            let _ = reply.send(Ok(page));
        },
        DatabaseMpscCommand::SetPage(new_page, reply) => {
            let result = sqlx::query!("UPDATE pages SET 
                created_at = $1, 
                created_by = $2, 
                modified_at = $3, 
                modified_by = $4, 
                published = $5, 
                body = $6 
                WHERE \"path\" = $7", 
                new_page.created_at, 
                new_page.created_by, 
                new_page.modified_at, 
                new_page.modified_by, 
                new_page.published, 
                new_page.body, 
                new_page.path
                )
                .execute(pool).await;
            
            if let Err(err) = result {
                let _ = reply.send(Err(err.into()));
            } else {
                let _ = reply.send(Ok(()));
                cache.set_page(&new_page).await;
            }
        },
        DatabaseMpscCommand::DeletePage(path, reply) => {
            let result = sqlx::query!("DELETE FROM pages WHERE \"path\" = $1", path)
                .execute(pool).await;

            if let Err(err) = result {
                let _ = reply.send(Err(err.into()));
            } else {
                let _ = reply.send(Ok(()));
            }
        },
        DatabaseMpscCommand::NewPage(new_page, reply) => {
            let result = sqlx::query!("INSERT INTO pages VALUES($1, $2, $3, $4, $5, $6, $7)",
                new_page.path, 
                new_page.created_at, 
                new_page.created_by,
                new_page.modified_at, 
                new_page.modified_by, 
                new_page.published,
                new_page.body)
                .execute(pool).await;

            if let Err(err) = result {
                let _ = reply.send(Err(err.into()));
            } else {
                let _ = reply.send(Ok(()));
            }
        },
        DatabaseMpscCommand::GetUser(id, skip_cache, reply) => {
            if !skip_cache {
                if let Some(user) = cache.get_user(id).await {
                    let _ = reply.send(Ok(user));
                    return;
                }
            }

            let user = match sqlx::query_as!(schema::AdminUser, 
                "SELECT * FROM admins WHERE id = $1", id)
                .fetch_one(pool).await {
                    Ok(user) => user,
                    Err(err) => {
                        let _ = reply.send(Err(err.into()));
                        return;
                    }
                };
            cache.set_user(&user).await;
            let _ = reply.send(Ok(user));
        },
        DatabaseMpscCommand::SetUser(new_user, reply) => {
            let result = sqlx::query!("UPDATE admins SET
                username = $1,
                enabled = $2,
                email = $3
                WHERE id = $4", 
                new_user.username,
                new_user.enabled,
                new_user.email,
                new_user.id)
                .execute(pool).await;

            if let Err(err) = result {
                let _ = reply.send(Err(err.into()));
            } else {
                let _ = reply.send(Ok(()));
            }
        },
        DatabaseMpscCommand::DeleteUser(id, reply) => {
            let result = sqlx::query!("DELETE FROM admins WHERE id = $1", id)
                .execute(pool).await;

            if let Err(err) = result {
                let _ = reply.send(Err(err.into()));
            } else {
                let _ = reply.send(Ok(()));
            }
        },
        DatabaseMpscCommand::NewUser(new_user, reply) => {
            let result = sqlx::query!("INSERT INTO admins VALUES($1, $2, $3, $4)",
                new_user.id,
                new_user.username,
                new_user.enabled,
                new_user.email)
                .execute(pool).await;

            if let Err(err) = result {
                let _ = reply.send(Err(err.into()));
            } else {
                let _ = reply.send(Ok(()));
            }
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
        let mut cache = cache::Cache::new().await; 
        loop {
           match rx.try_recv() {
               Ok(cmd) => {
                   process_cmd(cmd, &pool, &mut cache).await;
               },
               Err(err) => {
                   match err {
                       TryRecvError::Empty => {
                           // Do cache cleanup and stuff
                       },
                       TryRecvError::Disconnected => {
                           // Throw error message and stop loop
                           eprintln!("All transmitters have been disconnected. Exiting...");
                           cache.close().await;
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
