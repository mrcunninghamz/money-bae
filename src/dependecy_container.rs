use std::cell::OnceCell;
use crate::configuration_manager::ConfigurationManager;

pub struct DependencyContainer{
    configuration_manager: OnceCell<ConfigurationManager>,
}

impl DependencyContainer {
    pub fn new() -> Self {
        DependencyContainer {
            configuration_manager: OnceCell::new(),
        }
    }

    pub fn configuration_manager(&self) -> &ConfigurationManager {
        self.configuration_manager.get_or_init(|| ConfigurationManager::new())
    }
}
