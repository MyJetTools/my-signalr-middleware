use std::collections::HashMap;

use rust_extensions::lazy::LazyVec;

pub struct Tags {
    pub tags_to_connection: HashMap<String, HashMap<String, HashMap<String, ()>>>,
    pub connection_tags: HashMap<String, HashMap<String, String>>,
}

impl Tags {
    pub fn new() -> Self {
        Self {
            tags_to_connection: HashMap::new(),
            connection_tags: HashMap::new(),
        }
    }

    pub fn add_tag(&mut self, connection_id: &str, key: &str, value: &str) {
        if !self.connection_tags.contains_key(connection_id) {
            self.connection_tags
                .insert(connection_id.to_string(), HashMap::new());
        }

        let removed_tag = self
            .connection_tags
            .get_mut(connection_id)
            .unwrap()
            .insert(key.to_string(), value.to_string());

        if let Some(removed_tag) = removed_tag {
            if let Some(keys) = self.tags_to_connection.get_mut(key) {
                if let Some(tags) = keys.get_mut(&removed_tag) {
                    tags.remove(connection_id);
                }
            }
        }

        // Fill Tags to conection

        if !self.tags_to_connection.contains_key(key) {
            self.tags_to_connection
                .insert(key.to_string(), HashMap::new());
        }

        if let Some(key_value_connections) = self.tags_to_connection.get_mut(key) {
            if !key_value_connections.contains_key(value) {
                key_value_connections.insert(value.to_string(), HashMap::new());
            }

            key_value_connections
                .get_mut(value)
                .unwrap()
                .insert(connection_id.to_string(), ());
        }
    }

    pub fn remove_tag(&mut self, connection_id: &str, key: &str, value: &str) {
        // Removing first part

        if let Some(tags_of_connections) = self.connection_tags.get_mut(connection_id) {
            tags_of_connections.remove(key);
        }

        // Removing second part

        if let Some(tags) = self.tags_to_connection.get_mut(key) {
            let connections_remains = if let Some(connections) = tags.get_mut(value) {
                connections.remove(connection_id);
                connections.len()
            } else {
                0
            };

            if connections_remains == 0 {
                tags.remove(value);
            }
        }
    }

    pub fn remove_connection(&mut self, connection_id: &str) {
        if let Some(tags) = self.connection_tags.remove(connection_id) {
            for (key, value) in tags {
                let remove_tag = if let Some(values) = self.tags_to_connection.get_mut(&key) {
                    let remove = if let Some(connections) = values.get_mut(&value) {
                        connections.remove(connection_id);
                        connections.is_empty()
                    } else {
                        false
                    };

                    if remove {
                        values.remove(&value);
                    }

                    values.is_empty()
                } else {
                    false
                };

                if remove_tag {
                    self.tags_to_connection.remove(&key);
                }
            }
        };
    }

    pub fn get_tagged_connections_with_value(&self, key: &str, value: &str) -> Option<Vec<String>> {
        if let Some(tags) = self.tags_to_connection.get(key) {
            if let Some(connections) = tags.get(value) {
                if connections.len() == 0 {
                    return None;
                }

                let mut result = Vec::new();

                for id in connections.keys() {
                    result.push(id.to_string());
                }

                return Some(result);
            }
        }

        None
    }

    pub fn get_tagged_connections(&self, key: &str) -> Option<Vec<String>> {
        if let Some(tags) = self.tags_to_connection.get(key) {
            let mut result = LazyVec::new();
            for tags in tags.values() {
                for connection_id in tags.keys() {
                    result.add(connection_id.to_string());
                }
            }

            return result.get_result();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn general_tests() {
        let mut tags = Tags::new();

        tags.add_tag("c1", "userId", "1");
        tags.add_tag("c2", "userId", "2");

        tags.add_tag("c1", "asset", "BTCUSD");
        tags.add_tag("c2", "asset", "BTCUSD");

        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);

        let connections_by_asset = tags
            .get_tagged_connections_with_value("asset", "BTCUSD")
            .unwrap();

        assert_eq!(connections_by_asset.len(), 2);

        let connections_by_id = tags
            .get_tagged_connections_with_value("userId", "1")
            .unwrap();

        assert_eq!(connections_by_id.len(), 1);
    }

    #[test]
    fn test_remove_tag() {
        let mut tags = Tags::new();

        tags.add_tag("c1", "userId", "1");
        tags.add_tag("c2", "userId", "2");
        tags.add_tag("c3", "userId", "3");

        tags.add_tag("c1", "asset", "BTCUSD");
        tags.add_tag("c2", "asset", "BTCUSD");
        tags.add_tag("c3", "asset", "BTCUSD");

        println!("Initialized connections-------");
        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);

        tags.remove_tag("c2", "asset", "BTCUSD");
        println!("Removed one tag connections-------");
        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);

        let connections_by_asset = tags
            .get_tagged_connections_with_value("asset", "BTCUSD")
            .unwrap();

        assert_eq!(connections_by_asset.len(), 2);

        let connections_by_id = tags
            .get_tagged_connections_with_value("userId", "1")
            .unwrap();

        assert_eq!(connections_by_id.len(), 1);
    }

    #[test]
    fn remove_connection() {
        let mut tags = Tags::new();

        tags.add_tag("c1", "userId", "1");
        tags.add_tag("c2", "userId", "2");
        tags.add_tag("c3", "userId", "3");

        tags.add_tag("c1", "asset", "BTCUSD");
        tags.add_tag("c2", "asset", "BTCUSD");
        tags.add_tag("c3", "asset", "BTCUSD");

        println!("Initialized connections -------");
        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);
        println!("");

        tags.remove_connection("c2");
        println!("Removed one connection -------");
        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);
        println!("");

        let connections_by_asset = tags
            .get_tagged_connections_with_value("asset", "BTCUSD")
            .unwrap();

        assert_eq!(connections_by_asset.len(), 2);

        let connections_by_id = tags
            .get_tagged_connections_with_value("userId", "1")
            .unwrap();
        assert_eq!(connections_by_id.len(), 1);

        let connections_by_id = tags
            .get_tagged_connections_with_value("userId", "3")
            .unwrap();
        assert_eq!(connections_by_id.len(), 1);

        assert!(tags
            .get_tagged_connections_with_value("userId", "2")
            .is_none());
    }

    #[test]
    fn test_update_tag() {
        let mut tags = Tags::new();

        tags.add_tag("c1", "userId", "1");
        tags.add_tag("c2", "userId", "2");
        tags.add_tag("c3", "userId", "3");

        tags.add_tag("c1", "asset", "BTCUSD");
        tags.add_tag("c2", "asset", "BTCUSD");
        tags.add_tag("c3", "asset", "BTCUSD");

        println!("Initialized connections -------");
        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);
        println!("");

        tags.add_tag("c2", "asset", "ETHUSD");
        println!("Updated Asset -------");
        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);
        println!("");

        let connections_by_asset = tags
            .get_tagged_connections_with_value("asset", "BTCUSD")
            .unwrap();

        assert_eq!(connections_by_asset.len(), 2);

        let connections_by_asset = tags
            .get_tagged_connections_with_value("asset", "ETHUSD")
            .unwrap();

        assert_eq!(connections_by_asset.len(), 1);

        let connections_by_id = tags
            .get_tagged_connections_with_value("userId", "1")
            .unwrap();
        assert_eq!(connections_by_id.len(), 1);

        let connections_by_id = tags
            .get_tagged_connections_with_value("userId", "3")
            .unwrap();
        assert_eq!(connections_by_id.len(), 1);

        let connections_by_id = tags
            .get_tagged_connections_with_value("userId", "2")
            .unwrap();
        assert_eq!(connections_by_id.len(), 1);
    }

    #[test]
    fn test_tagged_connections() {
        let mut tags = Tags::new();

        tags.add_tag("c1", "userId", "1");
        tags.add_tag("c2", "userId", "2");
        tags.add_tag("c3", "userId", "3");

        tags.add_tag("c1", "asset", "BTCUSD");
        tags.add_tag("c2", "asset", "EUTHUSD");
        tags.add_tag("c3", "asset", "BTCUSD");

        println!("Initialized connections -------");
        println!("Connection tags: {:?}", tags.connection_tags);
        println!("Tags to connections: {:?}", tags.tags_to_connection);
        println!("");

        let connections_by_asset = tags.get_tagged_connections("asset").unwrap();

        println!("{:?}", connections_by_asset);
        assert_eq!(connections_by_asset.len(), 3);
    }
}
