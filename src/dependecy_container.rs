use std::cell::OnceCell;
use std::rc::Rc;
use crate::configuration_manager::ConfigurationManager;
use crate::db::PgConnector;

pub struct DependencyContainer{
    configuration_manager: OnceCell<ConfigurationManager>,
    pg_connector: OnceCell<Rc<PgConnector>>,
}

impl DependencyContainer {
    pub fn new() -> Self {
        DependencyContainer {
            configuration_manager: OnceCell::new(),
            pg_connector: OnceCell::new(),
        }
    }

    pub fn configuration_manager(&self) -> &ConfigurationManager {
        self.configuration_manager.get_or_init(|| ConfigurationManager::new())
    }
    
    pub fn pg_connector(&self) -> Rc<PgConnector> {
        Rc::clone(self.pg_connector.get_or_init(|| {
            let config_manager = self.configuration_manager();
            let connection_string = config_manager
                .get_database_connection_string()
                .unwrap_or_else(|| {
                    panic!("database_connection_string not found in configuration file.")
                })
                .to_string();
            Rc::new(PgConnector::new(connection_string))
        }))
    }
}
