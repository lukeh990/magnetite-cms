/*
 * database/schema.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use chrono::NaiveDateTime;

pub struct Page {
    path: String,
    created_at: NaiveDateTime,
    // created_by: Unkown
    modified_at: NaiveDateTime,
    // modified_by: Unkown
    published: bool,
    // permissions: Unkown
    // page_metadata: Unknown
    body: String,
    // styles: Unkown
    // scripts: Unkown
}
