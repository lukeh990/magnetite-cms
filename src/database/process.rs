/*
 * database/process.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use super::{DatabaseMpscCommand, schema, cache};
use sqlx::PgPool;

pub async fn cmd(cmd: DatabaseMpscCommand, pool: &PgPool, cache: &mut cache::Cache) {
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
