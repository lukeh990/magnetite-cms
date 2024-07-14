/*
 * database/schema.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
pub struct Page {
    path: String,
    created_at: NaiveDateTime,
    created_by: Uuid,
    modified_at: NaiveDateTime,
    modified_by: Uuid,
    published: bool,
    // permissions: Unkown
    // page_metadata: Unknown
    body: String,
    // styles: Unkown
    // scripts: Unkown
}

#[derive(sqlx::FromRow, Debug)]
pub struct AdminUser {
    uuid: Uuid,
    // permissions: Unkown
    username: String,
    enabled: bool,
    email: String,
    // authentication: Unkown
}
