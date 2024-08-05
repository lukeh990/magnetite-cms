/*
 * database/schema.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Page {
    pub path: String,
    pub created_at: NaiveDateTime,
    pub created_by: Uuid,
    pub modified_at: NaiveDateTime,
    pub modified_by: Uuid,
    pub published: bool,
    // permissions: Unkown
    pub metadata: Vec<String>,
    pub body: String,
    // styles: Unkown
    // scripts: Unkown
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AdminUser {
    pub id: Uuid,
    // permissions: Unkown
    pub username: String,
    pub enabled: bool,
    pub email: String,
    // authentication: Unkown
}
