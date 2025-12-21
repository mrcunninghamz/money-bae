use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ConfigurationManager {
    database_connection_string: Option<String>
}

impl ConfigurationManager {
    pub fn new() -> Self {
        let config_name = if cfg!(debug_assertions) {
            "money-bae-dev"
        } else {
            "money-bae"
        };
        confy::load(config_name, None).unwrap_or_default()
    }

    pub fn get_database_connection_string(&self) -> Option<&str> {
        self.database_connection_string.as_ref().map(|s| s.as_str())
    }
}

