use std::sync::Arc;

use actix_web::{http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer};
use testit_lib::{config::{EndpointConfiguration, HttpsConfiguration, MockResponseConfiguration, ServerConfiguration, TestConfiguration}, error::ApplicationError};
use tokio::sync::RwLock;
use regex::Regex;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

/**
 * The ServerSetup struct is used to start and stop servers.
 */
pub struct ServerSetup {
    servers: Arc<RwLock<Vec<AppServer>>>,
}

impl ServerSetup {
    pub fn new() -> Self {
        ServerSetup {
            servers: Arc::new(RwLock::new(vec![]))
        }
    }

    pub async fn setup_test(&mut self, test_configuration: &TestConfiguration) {
        let servers: Vec<AppServer> = test_configuration
            .servers
            .iter()
            .map(|server_configuration| AppServer::new(server_configuration.clone()))
            .collect();
        self.servers.write().await.extend(servers);
    }

    pub async fn start_servers(&mut self) -> Result<(), ApplicationError> {
        let mut handles = vec![];
        for server in self.servers.write().await.iter_mut() {            
            handles.push(server.start_server_http().await?);
            handles.push(server.start_server_https().await?);
        }
        Ok(())
    }

}

struct AppServer {
    server_configuration: ServerConfiguration,
}

impl AppServer {
    fn new(server_configuration: ServerConfiguration) -> Self {
        AppServer {
            server_configuration,
        }
    }

    async fn start_server_http(&mut self) -> Result<(), ApplicationError> {
        if let Some(http_port) = self.server_configuration.http_port {
            let appstate = web::Data::new(self.server_configuration.clone());
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(appstate.clone())
                    .default_service(web::to(request_handler))
            }).bind(("127.0.0.1", http_port)).map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
            let server = server.workers(2).run();
            tokio::spawn(async move {
                match server.await {
                    Ok(_) => {},
                    Err(err) => eprintln!("{}", err),
                }
            });                                   
        }
        Ok(())            
    }  

    /**
     * Start the server with HTTPS.
     * 
     * # Returns
     * @return Ok if the server was started.
     * 
     * # Errors
     * @return An error if the server could not be started.
     */
    async fn start_server_https(&self) -> Result<(), ApplicationError> {
        let config = self.server_configuration.clone();
        if let Some(https_config) = config.https_config {                        
            let ssl_builder = ssl_builder(&https_config)?;
            let appstate = web::Data::new(self.server_configuration.clone());
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(appstate.clone())
                    .default_service(web::to(request_handler))
            }).bind_openssl("127.0.0.1:".to_owned() + https_config.https_port.to_string().as_str(), ssl_builder).map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
            let server = server.workers(2).run();
            tokio::spawn(async move {
                match server.await {
                    Ok(_) => {},
                    Err(err) => eprintln!("{}", err),
                }
            });                                   
        }
        Ok(())            
        
    }      
}

/**
 * Handle the request.
 * 
 * # Arguments
 * @param server_configuration: The server configuration.
 * @param req: The request.
 * 
 * # Returns
 * @return The response.
 */
async fn request_handler(server_configuration: web::Data<ServerConfiguration>, req: HttpRequest) -> HttpResponse {
    for endpoint in server_configuration.endpoints.iter() {
        match is_valid_endpoint(&req, endpoint) {
            Ok(true) => { 
                match handle_endpoint(endpoint) {
                    Ok(response) => return response,
                    Err(err) => {   
                        eprintln!("{}", err);    
                        return HttpResponse::NotImplemented().body("Not implemented"); 
                    }
                }                
            },
            Ok(false) => continue,
            Err(err) => return HttpResponse::ServiceUnavailable().body(err.to_string())
        }    
    }
    HttpResponse::NotImplemented().body("Not implemented")
}



/**
 * Check if the request is a valid endpoint.
 * 
 * # Arguments
 * @param request: The request.
 * @param endpoint: The endpoint configuration.
 * 
 * # Returns
 * @return True if the request is a valid endpoint.
 * 
 * # Errors
 * @return An error if the endpoint is invalid.
 */
fn is_valid_endpoint(request: &HttpRequest, endpoint: &EndpointConfiguration) -> Result<bool, ApplicationError> {
    let regexp = Regex::new(&endpoint.endpoint).map_err(|err| ApplicationError::ConfigurationError(err.to_string()))?;
    Ok(regexp.is_match(request.uri().path()) && request.method().as_str() == endpoint.method.as_str())
}

/**
 * Handle the endpoint.
 * 
 * # Arguments
 * @param endpoint: The endpoint configuration.
 * 
 * # Returns
 * @return The response.
 * 
 * # Errors
 * @return An error if the status code is invalid.
 */
fn handle_endpoint(endpoint: &EndpointConfiguration) -> Result<HttpResponse, ApplicationError> {
    if let Some(mock_response) = &endpoint.mock_response {
        std::thread::sleep(std::time::Duration::from_millis(mock_response.delay));
        return generate_mock_response(mock_response);
    } 
    Ok(HttpResponse::NotImplemented().body("Not implemented"))
}

/**
 * Generate a mock response.
 * 
 * # Arguments
 * @param mock_response: The mock response configuration.
 * 
 * # Returns
 * @return The generated response.
 * 
 * # Errors
 * @return An error if the status code is invalid.
 */
fn generate_mock_response(mock_response: &MockResponseConfiguration) -> Result<HttpResponse, ApplicationError> {
    let mut response_builder: actix_web::HttpResponseBuilder = HttpResponse::build(StatusCode::from_u16(mock_response.status).map_err(|err| ApplicationError::ConfigurationError(err.to_string()))?);
    for (key, value) in mock_response.headers.iter() {
        response_builder.append_header((key.as_str(), value.as_str()));
    }
    if let Some(response) = &mock_response.response {
        return Ok(response_builder.body(response.clone()));
    }
    Ok(response_builder.finish())
}

/**
 * Create a new SSL builder.
 * 
 * # Arguments
 * @param https_config: The HTTPS configuration.
 * 
 * # Returns
 * @return The SSL builder.
 * 
 * # Errors
 * @return An error if the acceptor could not be created.
 * @return An error if the private key file could not be set.
 * @return An error if the certificate chain file could not be set.
 *
 */
fn ssl_builder(https_config: &HttpsConfiguration) -> Result<SslAcceptorBuilder, ApplicationError> {
    let mut builder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    builder.set_private_key_file(&https_config.private_key, SslFiletype::PEM).map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    builder.set_certificate_chain_file(&https_config.server_certificate).map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    Ok(builder)
} 

#[cfg(test)]
mod test {
    use std::{collections::HashMap, fs::File, io::Read, thread, time::Duration};

    use super::*;

    /**
     * Verifying that the server can be started.
     * TODO:Move this to integration tests
     */
    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_server_start() {
        let test_configuration = TestConfiguration {
            servers: vec![
                ServerConfiguration {
                    name: "test".to_string(),
                    http_port: Some(8080),
                    id: "test".to_string(),
                    endpoints: vec![],
                    https_config: None,
                    
                },
                ServerConfiguration {
                    name: "test".to_string(),
                    http_port: Some(8081),
                    id: "test".to_string(),
                    endpoints: vec![],
                    https_config: None,
                },
            ],
            name: "test".to_string(),
            description: "test".to_string(),
            id: "test".to_string(),
        };
        let mut server_setup = ServerSetup::new();
        server_setup.setup_test(&test_configuration).await;
        let result = server_setup.start_servers().await;
        assert!(result.is_ok());
        thread::sleep(Duration::from_secs(1));
        let res = reqwest::get("http://localhost:8080").await.unwrap();
        assert_eq!(res.status(), 501);
        let res = reqwest::get("http://localhost:8081").await.unwrap();
        assert_eq!(res.status(), 501);          
    }

    /**
     * Verifying that the endpoints are found.
     */
    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_endpoint_ok() {
        let test_configuration = TestConfiguration::new("test".to_string(), "test".to_string(),
        vec![
            ServerConfiguration::new("test".to_string(), Some(8082), vec![
                EndpointConfiguration::new("/test2".to_string(), "GET".to_string(), None, Some(MockResponseConfiguration::new(Some("{}".to_string()), 400, HashMap::new(), 1000)), None),
                EndpointConfiguration::new("/test".to_string(), "GET".to_string(), None, Some(MockResponseConfiguration::new(Some("{}".to_string()), 200, HashMap::new(), 1000)), None),    
            ],
            None),
        ]);
        let mut server_setup = ServerSetup::new();
        server_setup.setup_test(&test_configuration).await;
        let result = server_setup.start_servers().await;
        assert!(result.is_ok());        
        thread::sleep(Duration::from_secs(1));
        let res = reqwest::get("http://localhost:8082/test").await.unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.text().await.unwrap(), "{}".to_string());
        let res = reqwest::get("http://localhost:8082").await.unwrap();
        assert_eq!(res.status(), 501);              
    }   

    /**
     * Verifying https server.
     */
    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_https() {
        let server_cert_path = concat!(env!("CARGO_MANIFEST_DIR"), "/..", "/testit-daemon/test/resources/https_test/server_cert.pem").to_owned();
        let server_key_path = concat!(env!("CARGO_MANIFEST_DIR"), "/..", "/testit-daemon/test/resources/https_test/server_key.pem").to_owned();
        let https_config = HttpsConfiguration::new(server_cert_path.clone(), server_key_path, 8084);
        let test_configuration = TestConfiguration::new("test".to_string(), "test".to_string(),
        vec![
            ServerConfiguration::new("test".to_string(), None, vec![
                EndpointConfiguration::new("/".to_string(), "GET".to_string(), None, Some(MockResponseConfiguration::new(Some("{}".to_string()), 200, HashMap::new(), 1000)), None),    
            ],
            Some(https_config)),
        ]);
        let mut server_setup = ServerSetup::new();
        server_setup.setup_test(&test_configuration).await;
        let result = server_setup.start_servers().await;
        thread::sleep(Duration::from_secs(1));
        println!("{:?}", result);
        assert!(result.is_ok());  
        let mut buf = Vec::new();
        File::open(server_cert_path).unwrap().read_to_end(&mut buf).unwrap();        
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();        
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_hostnames(true)
            .build().unwrap();        
        let res = client.get("https://localhost:8084").send().await.unwrap();
        assert_eq!(res.status(), 200);                  
    }

}
