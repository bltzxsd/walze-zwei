use std::{ops::Deref, sync::Arc};

use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;
use walzecore::db::Users;

use crate::error::Error;

/// `Data` struct holds the users's dice rolls, which is an `Arc<Mutex<Users<serenity::UserId>>>`.
#[derive(Debug)]
pub struct Data(Arc<Mutex<Users<serenity::UserId>>>);

impl Data {
    /// Creates a new `Data` instance by wrapping the `Users` data in an `Arc` and `Mutex`.
    pub fn new(users: Users<serenity::UserId>) -> Self {
        Self(Arc::new(Mutex::new(users)))
    }
}

impl Deref for Data {
    type Target = Mutex<Users<serenity::UserId>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Type alias for `poise::Context` with the `Data` struct as the data type and `Error` as the error type.
pub type Context<'a> = poise::Context<'a, Data, Error>;

impl Drop for Data {
    /// When the `Data` instance is dropped, we want to write whatever is written into the `users.json` file. 
    fn drop(&mut self) {
        if let Ok(users) = self.0.try_lock() {
            let string = users.to_json();
            if let Err(e) = std::fs::write("users.json", string) {
                eprintln!("Error writing users.json file: {e}");
            }
        } else {
            eprintln!("Failed to acquire lock for writing users.json file");
        }
    }
}
