use serde::{Deserialize, Serialize};
/**
 * The configuration for the application. It contains all data that needs to be stored for the application.
 */
use std::collections::HashMap;
use uuid::Uuid;

/**
 * The configuration for the application.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfiguration {
    // The name of the configuration.
    pub name: String,
    // The description of the configuration.
    pub description: String,
    // The test configurations.
    pub tests: Vec<TestConfiguration>,
    // The endpoint for closing the servers.
    #[serde(default = "default_close_endpoint", alias = "closeEndpoint")]
    pub close_endpoint: String,
}

impl AppConfiguration {
    /**
     * Create a new configuration.
     *
     * @param name The name of the configuration.
     * @param description The description of the configuration.
     * @param tests The test configurations.
     *
     * @return The configuration.
     */
    pub fn new(
        name: String,
        description: String,
        tests: Vec<TestConfiguration>,
        close_endpoint: String,
    ) -> Self {
        AppConfiguration {
            name,
            description,
            tests,
            close_endpoint,
        }
    }

    /**
     * Add a test to the configuration.
     *
     * @param test The test configuration to add.
     */
    fn add_test(&mut self, test: TestConfiguration) {
        self.tests.push(test);
    }

    /**
     * Remove a test from the configuration.
     *
     * @param test_id The ID of the test to remove.
     */
    fn remove_test(&mut self, test_id: &str) {
        self.tests.retain(|test| test.id != test_id);
    }

    /**
     * Save the configuration to a file.
     *
     * @param path The path to save the configuration to.
     */
    fn save(&self, path: &str) {
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(path, serialized).unwrap();
    }

    /**
     * Load the configuration from a file.
     *
     * @param path The path to load the configuration from.
     *
     * @return The configuration.
     */
    pub fn load(path: &str) -> Self {
        let serialized = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&serialized).unwrap()
    }
}

fn default_close_endpoint() -> String {
    "/".to_string()
}

/**
 * Configuration for a test.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestConfiguration {
    // The ID of the test. This is a UUID automatically generated.
    pub id: String,
    // The name of the test.
    pub name: String,
    // The description of the test.
    pub description: String,
    // The server configurations.
    pub servers: Vec<ServerConfiguration>,
}

impl TestConfiguration {
    /**
     * Create a new test configuration.
     *
     * @param name The name of the test.
     * @param description The description of the test.
     * @param servers The server configurations.
     *
     * @return The test configuration.
     */
    pub fn new(name: String, description: String, servers: Vec<ServerConfiguration>) -> Self {
        TestConfiguration {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            servers,
        }
    }

    /**
     * Add a server to the test.
     *
     * @param server The server configuration to add.
     */
    fn add_server(&mut self, server: ServerConfiguration) {
        self.servers.push(server);
    }

    /**
     * Remove a server from the test.
     *
     * @param server_id The ID of the server to remove.
     */
    fn remove_server(&mut self, server_id: &str) {
        self.servers.retain(|server| server.id != server_id);
    }
}

/**
 * Configuration for a server.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfiguration {
    // The ID of the test. This is a UUID automatically generated.
    pub id: String,
    // The name of the server.
    pub name: String,
    // The port to run the server on.
    pub port: u16,
    // The endpoints to configure.
    pub endpoints: Vec<EndpointConfiguration>,
}

impl ServerConfiguration {
    /**
     * Create a new server configuration.
     *
     * @param name The name of the server.
     * @param port The port to run the server on.
     * @param endpoints The endpoints to configure.
     *
     * @return The server configuration.
     */
    pub fn new(name: String, port: u16, endpoints: Vec<EndpointConfiguration>) -> Self {
        ServerConfiguration {
            id: Uuid::new_v4().to_string(),
            name,
            port,
            endpoints,
        }
    }

    /**
     * Add an endpoint to the server.
     *
     * @param endpoint The endpoint configuration to add.
     */
    fn add_endpoint(&mut self, endpoint: EndpointConfiguration) {
        self.endpoints.push(endpoint);
    }

    /**
     * Remove an endpoint from the server.
     *
     * @param endpoint_id The ID of the endpoint to remove.
     */
    fn remove_endpoint(&mut self, endpoint_id: &str) {
        self.endpoints.retain(|endpoint| endpoint.id != endpoint_id);
    }
}

/**
 * Configuration for an endpoint.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointConfiguration {
    // The ID of the test. This is a UUID automatically generated.
    pub id: String,
    // Endpoint for the testit API. This is a regular expression.
    pub endpoint: String,
    // The HTTP method.
    pub method: String,
    // The SOAP action. Should only be used for soap requests.
    pub soap_action: Option<String>,
    // The mock response.
    pub mock_response: Option<MockResponseConfiguration>,
    // The route configuration.
    pub route: Option<RouteConfiguration>,
}

impl EndpointConfiguration {
    /**
     * Create a new endpoint configuration.
     *
     * @param endpoint Endpoint for the testit API. This is a regular expression.
     * @param mock_response The mock response.
     * @param route The route configuration.
     *
     * @return The endpoint configuration.
     */
    pub fn new(
        endpoint: String,
        method: String,
        soap_action: Option<String>,
        mock_response: Option<MockResponseConfiguration>,
        route: Option<RouteConfiguration>,
    ) -> Self {
        EndpointConfiguration {
            id: Uuid::new_v4().to_string(),
            endpoint,
            method,
            soap_action,
            mock_response,
            route,
        }
    }
}

/**
 * Configuration for a mock response.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MockResponseConfiguration {
    // The response to return when the mock is called.
    pub response: String,
    // The status code to return when the mock is called.
    pub status: u16,
    // The headers to return when the mock is called.
    pub headers: HashMap<String, String>,
    // Time to wait in milliseconds before returning the response.
    pub delay: u64,
}

impl MockResponseConfiguration {
    /**
     * Create a new mock response configuration.
     *
     * @param response The response to return when the mock is called.
     * @param status The status code to return when the mock is called.
     * @param headers The headers to return when the mock is called.
     * @param delay Time to wait before returning the response.
     *
     * @return The mock response configuration.
     */
    pub fn new(
        response: String,
        status: u16,
        headers: HashMap<String, String>,
        delay: u64,
    ) -> Self {
        MockResponseConfiguration {
            response,
            status,
            headers,
            delay,
        }
    }
}

/**
 * Configuration for a route.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouteConfiguration {
    // The URL of the endpoint.
    pub endpoint: String,
}

impl RouteConfiguration {
    /**
     * Create a new route configuration.
     *
     * @param endpoint The URL of the endpoint.
     *
     * @return The route configuration.
     */
    pub fn new(endpoint: String) -> Self {
        RouteConfiguration { endpoint }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    /**
     * Test creating a configuration.
     */
    #[test]
    fn test_configuration() {
        let configuration = AppConfiguration::new(
            "Test Configuration".to_string(),
            "Test Configuration Description".to_string(),
            vec![TestConfiguration::new(
                "Test".to_string(),
                "Test Description".to_string(),
                vec![ServerConfiguration::new(
                    "Server".to_string(),
                    8080,
                    vec![EndpointConfiguration::new(
                        "/test".to_string(),
                        "GET".to_string(),
                        None,
                        Some(MockResponseConfiguration::new(
                            "Test Response".to_string(),
                            200,
                            HashMap::new(),
                            0,
                        )),
                        Some(RouteConfiguration::new("/test".to_string())),
                    )],
                )],
            )],
            "/close".to_string(),
        );

        assert_eq!(configuration.name, "Test Configuration");
        assert_eq!(configuration.description, "Test Configuration Description");
        assert_eq!(configuration.tests.len(), 1);
        assert_eq!(configuration.tests[0].name, "Test");
        assert_eq!(configuration.tests[0].description, "Test Description");
        assert_eq!(configuration.tests[0].servers.len(), 1);
        assert_eq!(configuration.tests[0].servers[0].name, "Server");
        assert_eq!(configuration.tests[0].servers[0].port, 8080);
        assert_eq!(configuration.tests[0].servers[0].endpoints.len(), 1);
        assert_eq!(
            configuration.tests[0].servers[0].endpoints[0].endpoint,
            "/test"
        );
        assert_eq!(
            configuration.tests[0].servers[0].endpoints[0]
                .mock_response
                .as_ref()
                .unwrap()
                .response,
            "Test Response"
        );
        assert_eq!(
            configuration.tests[0].servers[0].endpoints[0]
                .mock_response
                .as_ref()
                .unwrap()
                .status,
            200
        );
        assert_eq!(
            configuration.tests[0].servers[0].endpoints[0]
                .mock_response
                .as_ref()
                .unwrap()
                .headers
                .len(),
            0
        );
        assert_eq!(
            configuration.tests[0].servers[0].endpoints[0]
                .mock_response
                .as_ref()
                .unwrap()
                .delay,
            0
        );
        assert_eq!(
            configuration.tests[0].servers[0].endpoints[0]
                .route
                .as_ref()
                .unwrap()
                .endpoint,
            "/test"
        );
    }

    /**
     * Test serializing and deserializing the configuration.
     */
    #[test]
    fn test_serialize_deserialize() {
        let configuration = AppConfiguration::new(
            "Test Configuration".to_string(),
            "Test Configuration Description".to_string(),
            vec![TestConfiguration::new(
                "Test".to_string(),
                "Test Description".to_string(),
                vec![ServerConfiguration::new(
                    "Server".to_string(),
                    8080,
                    vec![EndpointConfiguration::new(
                        "/test".to_string(),
                        "GET".to_string(),
                        None,
                        Some(MockResponseConfiguration::new(
                            "Test Response".to_string(),
                            200,
                            HashMap::new(),
                            0,
                        )),
                        Some(RouteConfiguration::new("/test".to_string())),
                    )],
                )],
            )],
            "/close".to_string(),
        );

        let serialized = serde_json::to_string(&configuration).unwrap();
        let deserialized: AppConfiguration = serde_json::from_str(&serialized).unwrap();

        assert_eq!(configuration, deserialized);
    }

    #[test]
    fn test_save_load() {
        let configuration = AppConfiguration::new(
            "Test Configuration".to_string(),
            "Test Configuration Description".to_string(),
            vec![TestConfiguration::new(
                "Test".to_string(),
                "Test Description".to_string(),
                vec![ServerConfiguration::new(
                    "Server".to_string(),
                    8080,
                    vec![EndpointConfiguration::new(
                        "/test".to_string(),
                        "GET".to_string(),
                        None,
                        Some(MockResponseConfiguration::new(
                            "Test Response".to_string(),
                            200,
                            HashMap::new(),
                            0,
                        )),
                        Some(RouteConfiguration::new("/test".to_string())),
                    )],
                )],
            )],
            "/close".to_string(),
        );

        let path = "/tmp/test.json";
        configuration.save(path);
        let loaded = AppConfiguration::load(path);

        assert_eq!(configuration, loaded);
    }
}
