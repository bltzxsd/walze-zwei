pub mod database;
pub mod error;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

pub use error::{Error, Result};

pub use crate::db::database::User;

/// A container for storing users, keyed by a hashable and equality-comparable type.
///
/// This struct provides a simple interface for adding users and retrieving them by their
/// associated key. It also supports serialization and deserialization using serde.
///
/// # Examples
///
/// ```
/// use walzecore::db::Users;
/// use walzecore::db::User;
///
/// let mut users = Users::<u64>::new("{}").unwrap();
/// let user = User::new();
/// users.add_user(1, user);
/// ```
#[derive(Debug, Default, Serialize)]
pub struct Users<T>
where
    T: Hash + Eq + Serialize + DeserializeOwned,
{
    users: HashMap<T, User>,
}

impl<T: Hash + Eq + Serialize + DeserializeOwned> Users<T> {
    /// Creates a new `Users` instance from a JSON string.
    ///
    /// If the provided JSON string is invalid or cannot be deserialized, an empty `Users`
    /// instance is created.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::Users;
    ///
    /// let users = Users::<u64>::new("{}").unwrap();
    /// ```
    pub fn new(json: &str) -> Result<Users<T>> {
        let users = serde_json::from_str(json).unwrap_or_default();
        Ok(Users { users })
    }

    /// Adds a new user to the container.
    ///
    /// If a user with the same key already exists, it will be overwritten.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::Users;
    /// use walzecore::db::User;
    ///
    /// let mut users = Users::<u64>::new("{}").unwrap();
    /// let user = User::new();
    /// users.add_user(1, user);
    /// ```
    pub fn add_user(&mut self, id: T, user: User) {
        self.insert(id, user);
    }

    /// Converts the users container to a JSON string.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::Users;
    ///
    /// let users = Users::<u64>::new("{}").unwrap();
    /// let json = users.to_json();
    /// ```
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.users).unwrap_or_else(|_| "{}".to_string())
    }
}

impl<T: Hash + Eq + Serialize + DeserializeOwned> Deref for Users<T> {
    type Target = HashMap<T, User>;

    fn deref(&self) -> &Self::Target {
        &self.users
    }
}

impl<T: Hash + Eq + Serialize + DeserializeOwned> DerefMut for Users<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.users
    }
}

impl<'de, T> Deserialize<'de> for Users<T>
where
    T: Hash + Eq + Serialize + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let users = HashMap::deserialize(deserializer)?;
        Ok(Users { users })
    }
}
