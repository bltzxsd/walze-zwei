use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;
use walzecore::db::Users;

use crate::error::Error;

/// `Data` struct holds the users's dice rolls, which is an `Arc<Mutex<Users<serenity::UserId>>>`.
#[derive(Debug)]
pub struct Data {
    data: Arc<Mutex<Inner>>,
}

impl Data {
    pub fn new(data: Arc<Mutex<Inner>>) -> Self {
        Self { data }
    }
}

impl Deref for Data {
    type Target = Arc<Mutex<Inner>>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Data {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Debug)]
pub struct Inner(Users<serenity::UserId>);

impl Inner {
    pub fn new(usr: Users<serenity::UserId>) -> Self {
        Self(usr)
    }
}

impl Deref for Inner {
    type Target = Users<serenity::UserId>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Type alias for `poise::Context` with the `Data` struct as the data type and `Error` as the error type.
pub type Context<'a> = poise::Context<'a, Data, Error>;

impl Drop for Inner {
    /// When the `Data` instance is dropped, we want to write whatever is written into the `users.json` file. 
    fn drop(&mut self) {
        // Write the updated users data to the JSON file before dropping
        let string = self.0.to_json();
        let _ = std::fs::write("users.json", string);
    }
}
