use serde::{Deserialize, Serialize};
/**
 * The configuration for the application. It contains all data that needs to be stored for the application.
 */
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::ApplicationError;

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
    ) -> Self {
        AppConfiguration {
            name,
            description,
            tests,
        }
    }

    /**
     * Save the configuration to a file.
     *
     * @param path The path to save the configuration to.
     * 
     * @return Ok if the configuration was saved successfully.
     * 
     * # Errors
     * @return An error if the configuration could not be saved.
     */
    fn save(&self, path: &str) -> Result<(), ApplicationError> {
        let string_data = serde_json::to_string_pretty(&self).map_err(|err| ApplicationError::FileError(err.to_string()))?;
        std::fs::write(path, string_data).map_err(|err| ApplicationError::FileError(err.to_string()))?;
        Ok(())
    }

    /**
     * Load the configuration from a file.
     *
     * @param path The path to load the configuration from.
     *
     * @return The configuration.
     * 
     * # Errors
     * @return An error if the configuration could not be loaded.
     */
    pub fn load(path: &str) -> Result<Self, ApplicationError> {
        let string_data = std::fs::read_to_string(path).map_err(|err| ApplicationError::FileError(err.to_string()))?;
        serde_json::from_str(&string_data).map_err(|err| ApplicationError::FileError(err.to_string()))
    }

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
}

/**
 * Configuration for an https server.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HttpsConfiguration {
    // The path to the certificate.
    pub server_certificate: String,
    // The path to the private key.
    pub private_key: String,
    // The https port
    pub https_port: u16,

}

impl HttpsConfiguration {
    /**
     * Create a new https configuration.
     *
     * @param certificate The path to the certificate.
     * @param private_key The path to the private key.
     * @param https_port The https port.
     *
     * @return The https configuration.
     */
    pub fn new(server_certificate: String, private_key: String, https_port: u16) -> Self {
        HttpsConfiguration {
            server_certificate,
            private_key,
            https_port,
        }
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
    pub http_port: Option<u16>,
    // The endpoints to configure.
    pub endpoints: Vec<EndpointConfiguration>,
    // The https configuration.
    pub https_config: Option<HttpsConfiguration>,
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
    pub fn new(name: String, http_port: Option<u16>, endpoints: Vec<EndpointConfiguration>, https_config: Option<HttpsConfiguration>) -> Self {
        ServerConfiguration {
            id: Uuid::new_v4().to_string(),
            name,
            http_port,
            endpoints,            
            https_config,
        }
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
    pub response: Option<String>,
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
        response: Option<String>,
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
                    Some(8080),
                    vec![EndpointConfiguration::new(
                        "/test".to_string(),
                        "GET".to_string(),
                        None,
                        Some(MockResponseConfiguration::new(
                            Some("Test Response".to_string()),
                            200,
                            HashMap::new(),
                            0,
                        )),
                        Some(RouteConfiguration::new("/test".to_string())),
                    )],
                    None
                )],
            )],
        );

        assert_eq!(configuration.name, "Test Configuration");
        assert_eq!(configuration.description, "Test Configuration Description");
        assert_eq!(configuration.tests.len(), 1);
        assert_eq!(configuration.tests[0].name, "Test");
        assert_eq!(configuration.tests[0].description, "Test Description");
        assert_eq!(configuration.tests[0].servers.len(), 1);
        assert_eq!(configuration.tests[0].servers[0].name, "Server");
        assert_eq!(configuration.tests[0].servers[0].http_port, Some(8080));
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
            Some("Test Response".to_string())
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
                    Some(8080),
                    vec![EndpointConfiguration::new(
                        "/test".to_string(),
                        "GET".to_string(),
                        None,
                        Some(MockResponseConfiguration::new(
                            Some("Test Response".to_string()),
                            200,
                            HashMap::new(),
                            0,
                        )),
                        Some(RouteConfiguration::new("/test".to_string())),
                    )],
                    None
                )],
            )],
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
                    Some(8080),
                    vec![EndpointConfiguration::new(
                        "/test".to_string(),
                        "GET".to_string(),
                        None,
                        Some(MockResponseConfiguration::new(
                            Some("Test Response".to_string()),
                            200,
                            HashMap::new(),
                            0,
                        )),
                        Some(RouteConfiguration::new("/test".to_string())),
                    )],
                    None
                )],
            )],
        );

        let path = "/tmp/test.json";
        let _ = configuration.save(path);
        let loaded = AppConfiguration::load(path).unwrap();

        assert_eq!(configuration, loaded);
    }
}
