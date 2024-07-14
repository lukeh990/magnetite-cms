/*
 * database/cache.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

use std::collections::HashMap;
use tokio::{sync::Mutex, task};
use std::sync::Arc;
use std::time::Duration;
use chrono::{NaiveDateTime, TimeDelta, Utc};
use tokio::time::sleep;
use uuid::Uuid;
use super::schema;


#[derive(Eq, PartialEq, Hash, Clone)]
enum CacheKey {
    Page(String),
    User(Uuid)
}

#[derive(Clone)]
enum CacheValue {
    Page(schema::Page, NaiveDateTime),
    User(schema::AdminUser, NaiveDateTime)
}

pub struct Cache {
    storage: Arc<Mutex<HashMap<CacheKey, CacheValue>>>,
    loop_handle: task::JoinHandle<()>
}

impl Cache {
    pub async fn new() -> Cache {
        let storage = Arc::new(Mutex::new(HashMap::new()));

        let thread_storage = storage.clone();
        let loop_handle = tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                let mut storage = thread_storage.lock().await;
                
                for (key, value) in storage.clone().into_iter() {
                    let valid_until = match value {
                        CacheValue::Page(_, valid_until) => valid_until,
                        CacheValue::User(_, valid_until) => valid_until
                    };

                    let now = Utc::now().naive_utc();
                    
                    if valid_until < now {
                        storage.remove(&key);
                    }
                }
            }
        });

        Cache {
            storage,
            loop_handle
        }
    }

    pub async fn close(&self) {
        self.loop_handle.abort();
    }

    pub async fn get_page<S>(&self, path: S) -> Option<schema::Page> 
    where S: Into<String>
    {
        let storage = self.storage.lock().await;
        let result = storage.get(&CacheKey::Page(path.into()));

        if let Some(result) = result {
            match result {
                CacheValue::Page(page, _) => Some(page.clone()),
                _ => None
            }
        } else {
            None
        }
    }

    pub async fn get_user<U>(&self, id: U) -> Option<schema::AdminUser> 
    where U: Into<Uuid>
    { 
        let storage = self.storage.lock().await;
        let result = storage.get(&CacheKey::User(id.into())); 
        if let Some(result) = result {
            match result {
                CacheValue::User(user, _) => Some(user.clone()),
                _ => None
            }
        } else {
            None
        }
    }

    pub async fn set_page(&mut self, page: &schema::Page) {
        let naive_time = Utc::now().naive_utc();
        let valid_until = match naive_time.checked_add_signed(TimeDelta::minutes(1)) {
            Some(result) => result, 
            None => {return}
        };

        let mut storage = self.storage.lock().await;
        storage.insert(CacheKey::Page(page.path.clone()), CacheValue::Page(page.clone(), valid_until));
    }
    
    pub async fn set_user(&mut self, user: &schema::AdminUser) {
        let naive_time = Utc::now().naive_utc();
        let valid_until = match naive_time.checked_add_signed(TimeDelta::minutes(1)) {
            Some(result) => result, 
            None => {return}
        };

        let mut storage = self.storage.lock().await;
        storage.insert(CacheKey::User(user.id), CacheValue::User(user.clone(), valid_until));
    }
}

