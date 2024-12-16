use crate::{config::{AppConfiguration, TestConfiguration}, server::ServerSetup};

struct Application {
    application_setup: Option<AppConfiguration>,
    server_setup: Option<ServerSetup>,
}

impl Application {
    pub fn new() -> Self {
        Application {
            application_setup: None,
            server_setup: None,
        }
    }

    pub fn load(&mut self, config_file: &str) {
        self.application_setup = Some(AppConfiguration::load(config_file));
    }

    pub fn select_test(&mut self, test_configuration: TestConfiguration) {
        
    }
}
