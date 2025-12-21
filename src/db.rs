use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenvy::dotenv;
use std::env;

pub struct PgConnector {
    connection: PgConnection,
}

impl PgConnector {
    pub fn new(connection_string: String) -> Self {
        Self {
            connection: PgConnection::establish(connection_string.as_str())
                .unwrap_or_else(|_| panic!("Error connecting to {}", connection_string)),
        }
    }

    pub fn get_connection(&mut self) -> &mut PgConnection {
        &mut self.connection
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("MONEYBAE_DATABASE_URL")
        .expect("MONEYBAE_DATABASE_URL must be set in environment or .env file");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
