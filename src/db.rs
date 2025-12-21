use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenvy::dotenv;
use std::env;
use std::cell::RefCell;

pub struct PgConnector {
    connection: RefCell<PgConnection>
}

impl PgConnector {
    pub fn new(connection_string: String) -> Self {
        Self {
            connection: RefCell::new(
                PgConnection::establish(connection_string.as_str())
                    .unwrap_or_else(|_| panic!("Error connecting to {}", connection_string))
            )
        }
    }
    
    pub fn get_connection(&self) -> std::cell::RefMut<'_, PgConnection> {
        self.connection.borrow_mut()
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("MONEYBAE_DATABASE_URL")
        .expect("MONEYBAE_DATABASE_URL must be set in environment or .env file");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
