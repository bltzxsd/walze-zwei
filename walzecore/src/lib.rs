pub mod db;
pub mod tz;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {

    use crate::db::{database::User, Users};

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn write_to_json() {
        let mut db = Users::new("{}").unwrap();
        let mut user = User::new();
        user.add_namespace("dnd");
        user.namespace_mut("dnd");
        user.alias_mut("$adv", "2d10").unwrap();
        user.add_namespace("w&g");
        user.namespace_mut("w&g");
        user.alias_mut("$ballistics", "7d6, 1d6").unwrap();
        db.insert(1, user);
        let mut user2 = User::new();
        user2.add_namespace("dnd");
        user2.namespace_mut("dnd");
        user2.alias_mut("$adv", "2d10").unwrap();
        user2.add_namespace("w&g");
        user2.namespace_mut("w&g");
        user2.alias_mut("$ballistics", "7d6, 1d6").unwrap();
        db.add_user(2, user2); // unwrap is NONE
        let json = serde_json::to_string_pretty(&db).unwrap();
        std::fs::write("users.json", json).unwrap();
    }
}
