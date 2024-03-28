use std::collections::{HashMap, HashSet};
use std::convert;

use serde::{Deserialize, Serialize};

use crate::db;
use crate::db::Result;

/// A struct representing a user with namespaces and aliases.
///
/// The `User` struct allows managing namespaces, where each namespace can have its own set of
/// key-value aliases. A user has a current namespace and can switch between available namespaces.
///
/// # Examples
///
/// ```
/// use walzecore::db::database::User;
///
/// let mut user = User::new();
/// user.add_namespace("char1"); // create namespace
/// user.namespace_mut("char1"); // switch to the "char1" namespace
/// user.alias_mut("$adv", "2d20"); // set alias for stealth in the current namespace
/// assert_eq!(user.alias("$adv")?, "2d20".to_string());
/// # Ok::<(), self::walzecore::db::Error>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct User {
    namespace: String,
    available_namespaces: HashSet<String>,
    alias: HashMap<String, HashMap<String, String>>,
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

impl User {
    /// Creates a new `User` instance with a default namespace.
    ///
    /// The default namespace is named "default", and it is the initial current namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let user = User::new();
    /// assert_eq!(user.namespace(), "default");
    /// assert!(user.available_namespaces().contains("default"));
    /// ```
    pub fn new() -> Self {
        let namespace = "default".to_string();
        let mut available_namespaces = HashSet::new();
        available_namespaces.insert(namespace.clone());
        let mut alias = HashMap::new();
        alias.insert(namespace.clone(), HashMap::new());
        Self {
            namespace,
            available_namespaces,
            alias,
        }
    }

    /// Adds a new namespace to the user.
    ///
    /// If the namespace already exists, this method has no effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let mut user = User::new();
    /// user.add_namespace("character-sheets");
    /// assert!(user.available_namespaces().contains("character-sheets"));
    /// ```
    pub fn add_namespace<T: Into<String>>(&mut self, name: T) {
        let k = name.into();
        self.available_namespaces.insert(k.clone());
        self.alias.insert(k, HashMap::new());
    }

    /// Returns the current namespace of the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let mut user = User::new();
    /// assert_eq!(user.namespace(), "default");
    /// ```
    pub fn namespace(&self) -> &str {
        self.namespace.as_str()
    }

    /// Changes the current namespace of the user.
    ///
    /// If the provided namespace does not exist, this method has no effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let mut user = User::new();
    /// assert_eq!(user.namespace(), "default");
    /// user.add_namespace("game-rules");
    /// user.namespace_mut("game-rules");
    /// assert_eq!(user.namespace(), "game-rules");
    /// ```
    pub fn namespace_mut<T: Into<String>>(&mut self, namespace: T) {
        let namespace = namespace.into();
        if self.available_namespaces.contains(&namespace) {
            self.namespace = namespace;
        }
    }

    /// Returns a [``HashSet``] containing all declared namespaces.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let user = User::new();
    /// let ns: Vec<_> = user.available_namespaces().into_iter().collect();
    /// assert_eq!(ns, vec!["default".to_string()]);
    /// ```
    pub fn available_namespaces(&self) -> HashSet<String> {
        self.available_namespaces.clone()
    }

    /// Adds or updates an alias in the current namespace.
    ///
    /// # Errors
    ///
    /// If the namespace does not exist, an error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let mut user = User::new();
    /// assert_eq!(user.namespace(), "default");
    /// user.alias_mut("test", "new");
    /// assert_eq!(user.alias("test").unwrap(), "new");
    /// ```
    pub fn alias_mut<'a, T>(&mut self, k: T, v: T) -> Result<()>
    where
        T: Into<String> + convert::From<&'a str>,
    {
        let Some(set) = self.alias.get_mut(&self.namespace) else {
            return Err(db::Error::InvalidNamespace(self.namespace.clone()));
        };
        set.insert(k.into(), v.into());

        Ok(())
    }

    /// Retrieves the value associated with an alias in the current namespace.
    ///
    /// If the alias does not exist in the current namespace, an error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let mut user = User::new();
    /// user.add_namespace("LMoP"); // create namespace: LMoP
    /// user.namespace_mut("LMoP"); // switch to LMoP namespace
    /// user.alias_mut("$stealth", "2d6 t4 tt4, 1d6").unwrap(); // add $stealth to LMoP
    /// let stealth_roll = user.alias("$stealth").unwrap();
    /// assert_eq!(stealth_roll, "2d6 t4 tt4, 1d6".to_string());
    /// ```
    pub fn alias<'a, T: Into<String> + convert::From<&'a str>>(&self, alias: T) -> Result<String> {
        let alias = alias.into();
        match self
            .alias
            .get(&self.namespace)
            .ok_or_else(|| db::Error::InvalidNamespace(self.namespace.clone()))?
            .get(&alias)
        {
            Some(v) => Ok(v.to_owned()),
            None => Err(db::Error::AliasNotFound(alias)),
        }
    }

    /// Returns a list of all aliases in the current namespace
    ///
    /// # Errors
    ///
    /// If the namespace does not exist, an error is returned.
    pub fn aliases(&self) -> Result<Vec<(&str, &str)>> {
        let aliases = self
            .alias
            .get(self.namespace())
            .ok_or_else(|| db::Error::InvalidNamespace(self.namespace.clone()))?;

        Ok(aliases
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect())
    }

    /// Removes an alias from the current namespace.
    ///
    /// # Errors
    ///
    /// If the namespace or alias does not exist, an error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let mut user = User::new();
    /// user.add_namespace("game-rules");
    /// user.namespace_mut("game-rules");
    /// user.alias_mut("$stealth", "2d6 t4 tt4, 1d6").unwrap();
    /// let removed_alias = user.remove_alias("$stealth").unwrap();
    /// assert_eq!(removed_alias, "2d6 t4 tt4, 1d6");
    /// ```
    pub fn remove_alias<T: Into<String>>(&mut self, alias: T) -> Result<String> {
        let alias = alias.into();
        let alias_set = self
            .alias
            .get_mut(&self.namespace)
            .ok_or_else(|| db::Error::NamespaceNotFound(self.namespace.clone()))?;

        alias_set
            .remove(&alias)
            .ok_or_else(|| db::Error::AliasNotFound(alias))
    }

    /// Removes a namespace and returns its associated aliases.
    ///
    /// If the namespace does not exist, an error is returned.
    /// If the removed namespace was the current namespace, the current namespace is set to "default".
    ///
    /// # Examples
    ///
    /// ```
    /// use walzecore::db::database::User;
    ///
    /// let mut user = User::new();
    /// user.add_namespace("game-rules");
    /// user.namespace_mut("game-rules");
    /// user.alias_mut("$stealth", "2d6 t4 tt4, 1d6").unwrap();
    /// let (namespace, aliases) = user.remove_namespace("game-rules").unwrap();
    /// assert_eq!(namespace, "game-rules");
    /// assert_eq!(aliases.get("$stealth"), Some(&"2d6 t4 tt4, 1d6".to_string()));
    /// ```
    pub fn remove_namespace<T: Into<String>>(
        &mut self,
        namespace: T,
    ) -> Result<(String, HashMap<String, String>)> {
        let ns: String = namespace.into();
        if self.available_namespaces.contains(&ns) {
            self.available_namespaces.remove(&ns);
        } else {
            return Err(db::Error::NamespaceNotFound(ns));
        }

        if self.namespace == ns {
            self.namespace = "default".to_string();
        }

        self.alias
            .remove_entry(&ns)
            .ok_or_else(|| db::Error::NamespaceNotFound(ns))
    }
}
