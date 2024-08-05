/*
 * database/cache.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use super::println;
use super::schema;
use chrono::{NaiveDateTime, TimeDelta, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use uuid::Uuid;

#[derive(Eq, PartialEq, Hash, Clone)]
enum CacheKey {
    Page(String),
    User(Uuid),
}

#[derive(Clone)]
enum CacheValue {
    Page(schema::Page, NaiveDateTime),
    User(schema::AdminUser, NaiveDateTime),
}

pub struct Cache {
    storage: Arc<Mutex<HashMap<CacheKey, CacheValue>>>,
}

impl Cache {
    pub async fn new(tracker: TaskTracker, cancel_token: CancellationToken) -> Cache {
        let storage = Arc::new(Mutex::new(HashMap::new()));

        let thread_storage = storage.clone();
        tracker.spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                let mut storage = thread_storage.lock().await;

                for (key, value) in storage.clone().into_iter() {
                    let valid_until = match value {
                        CacheValue::Page(_, valid_until) => valid_until,
                        CacheValue::User(_, valid_until) => valid_until,
                    };

                    let now = Utc::now().naive_utc();

                    if valid_until < now {
                        storage.remove(&key);
                    }
                }

                if cancel_token.is_cancelled() {
                    println::error("Cache Cancellation Token Received...");
                    break;
                }
            }
        });

        Cache { storage }
    }

    pub async fn get_page<S>(&self, path: S) -> Option<schema::Page>
    where
        S: Into<String>,
    {
        let storage = self.storage.lock().await;
        let result = storage.get(&CacheKey::Page(path.into()));

        if let Some(result) = result {
            match result {
                CacheValue::Page(page, _) => Some(page.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub async fn get_user<U>(&self, id: U) -> Option<schema::AdminUser>
    where
        U: Into<Uuid>,
    {
        let storage = self.storage.lock().await;
        let result = storage.get(&CacheKey::User(id.into()));
        if let Some(result) = result {
            match result {
                CacheValue::User(user, _) => Some(user.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub async fn set_page(&mut self, page: &schema::Page) {
        let naive_time = Utc::now().naive_utc();
        let valid_until = match naive_time.checked_add_signed(TimeDelta::seconds(30)) {
            Some(result) => result,
            None => return,
        };

        let mut storage = self.storage.lock().await;
        storage.insert(
            CacheKey::Page(page.path.clone()),
            CacheValue::Page(page.clone(), valid_until),
        );
    }

    pub async fn set_user(&mut self, user: &schema::AdminUser) {
        let naive_time = Utc::now().naive_utc();
        let valid_until = match naive_time.checked_add_signed(TimeDelta::seconds(30)) {
            Some(result) => result,
            None => return,
        };

        let mut storage = self.storage.lock().await;
        storage.insert(
            CacheKey::User(user.id),
            CacheValue::User(user.clone(), valid_until),
        );
    }
}
