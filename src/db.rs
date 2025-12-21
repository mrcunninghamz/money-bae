use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenvy::dotenv;
use std::env;
use crate::configuration_manager::ConfigurationManager;
use crate::dependecy_container::DependencyContainer;

pub struct PgConnector {
    connection_string: String,
}

impl PgConnector {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
    
    pub fn establish_connection(&self) -> PgConnection {
        PgConnection::establish(self.connection_string.as_str())
            .unwrap_or_else(|_| panic!("Error connecting to {}", self.connection_string))
    }

}

impl DependencyContainer {
    fn create_pg_connector(
        &self,
        configuration_manager: &ConfigurationManager,
    ) -> PgConnector {
        let connection_string = configuration_manager
            .get_database_connection_string()
            .unwrap_or_else(|| {
                panic!("Database connection string not found in configuration manager")
            })
            .to_string();
        
        PgConnector::new(connection_string)
    }

    pub fn pg_connector(&self) -> PgConnector {
        let config_manager = self.configuration_manager();
        self.create_pg_connector(config_manager)
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("MONEYBAE_DATABASE_URL")
        .expect("MONEYBAE_DATABASE_URL must be set in environment or .env file");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
