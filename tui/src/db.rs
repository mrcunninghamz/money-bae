use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::cell::RefCell;

pub struct PgConnector {
    connection: RefCell<PgConnection>,
}

impl PgConnector {
    pub fn new(connection_string: String) -> Self {
        Self {
            connection: RefCell::new(
                PgConnection::establish(connection_string.as_str())
                    .unwrap_or_else(|_| panic!("Error connecting to {}", connection_string))
            ),
        }
    }

    pub fn get_connection(&self) -> std::cell::RefMut<'_, PgConnection> {
        self.connection.borrow_mut()
    }
}


