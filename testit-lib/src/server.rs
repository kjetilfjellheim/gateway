use std::{borrow::BorrowMut, cell::RefCell, future::IntoFuture, net::SocketAddr, os::unix::thread, sync::Arc};

use actix_web::{dev::{Response, Server}, http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer};
use tokio::sync::{Mutex, RwLock};
use regex::Regex;

use crate::config::{ServerConfiguration, TestConfiguration};

/**
 * The ServerSetup struct is used to start and stop servers.
 * 
 * This implementation might contain a memory leak as I need to identify what happens when the server handles are stopped.
 */
pub struct ServerSetup {
    servers: Arc<RwLock<Vec<AppServer>>>,
}

impl ServerSetup {
    fn new() -> Self {
        ServerSetup {
            servers: Arc::new(RwLock::new(vec![]))
        }
    }

    async fn setup_test(&mut self, test_configuration: TestConfiguration) {
        let servers: Vec<AppServer> = test_configuration
            .servers
            .iter()
            .map(|server_configuration| AppServer::new(server_configuration.clone()))
            .collect();
        self.servers.write().await.extend(servers);
    }

    async fn start_servers(&mut self) {
        let mut handles = vec![];
        for server in self.servers.write().await.iter_mut() {
            handles.push(server.start_server().await);
        }
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

    async fn start_server(&mut self) {
        let appstate = web::Data::new(self.server_configuration.clone());
        let server = HttpServer::new(move || {
            App::new()
                .app_data(appstate.clone())
                .default_service(web::to(Self::request_handler))
        })
        .bind(("127.0.0.1", self.server_configuration.port)).unwrap()
        .workers(5)
        .run();
        tokio::spawn(async move {
            server.await;
        });
    }

    async fn request_handler(server_configuration: web::Data<ServerConfiguration>, req: HttpRequest) -> HttpResponse {
        for endpoint in server_configuration.endpoints.iter() {
            if is_valid_endpoint(&req, &endpoint) {
                return handle_endpoint(&endpoint);
            }
        }
        HttpResponse::NotImplemented().body("Not implemented")
    }    

}

fn is_valid_endpoint(request: &HttpRequest, endpoint: &crate::config::EndpointConfiguration) -> bool {
    let regexp = Regex::new(&endpoint.endpoint).unwrap();
    regexp.is_match(request.uri().path()) && request.method().as_str() == endpoint.method.as_str()
}

fn handle_endpoint(endpoint: &crate::config::EndpointConfiguration) -> HttpResponse {
    if let Some(mock_response) = &endpoint.mock_response {
        std::thread::sleep(std::time::Duration::from_millis(mock_response.delay));
        return generate_mock_response(mock_response);
    } 
    HttpResponse::NotImplemented().body("Not implemented")
}

fn generate_mock_response(mock_response: &crate::config::MockResponseConfiguration) -> HttpResponse {
    let mut response_builder: actix_web::HttpResponseBuilder = HttpResponse::build(StatusCode::from_u16(mock_response.status).unwrap());
    for (key, value) in mock_response.headers.iter() {
        response_builder.append_header((key.as_str(), value.as_str()));
    }
    if let Some(response) = &mock_response.response {
        return response_builder.body(response.clone());
    }
    response_builder.finish()
}

mod test {
    use std::{collections::HashMap, thread, time::Duration};

    use super::*;
    use crate::config::{EndpointConfiguration, MockResponseConfiguration, TestConfiguration};

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
                    port: 8080,
                    id: "test".to_string(),
                    endpoints: vec![],
                },
                ServerConfiguration {
                    name: "test".to_string(),
                    port: 8081,
                    id: "test".to_string(),
                    endpoints: vec![],
                },
            ],
            name: "test".to_string(),
            description: "test".to_string(),
            id: "test".to_string(),
        };
        let mut server_setup = ServerSetup::new();
        server_setup.setup_test(test_configuration).await;
        server_setup.start_servers().await;
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
            ServerConfiguration::new("test".to_string(), 8082, vec![
                EndpointConfiguration::new("/test2".to_string(), "GET".to_string(), None, Some(MockResponseConfiguration::new(Some("{}".to_string()), 400, HashMap::new(), 1000)), None),
                EndpointConfiguration::new("/test".to_string(), "GET".to_string(), None, Some(MockResponseConfiguration::new(Some("{}".to_string()), 200, HashMap::new(), 1000)), None),    
            ]),
        ]);
        let mut server_setup = ServerSetup::new();
        server_setup.setup_test(test_configuration).await;
        server_setup.start_servers().await;
        thread::sleep(Duration::from_secs(1));
        let res = reqwest::get("http://localhost:8082/test").await.unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.text().await.unwrap(), "{}".to_string());
        let res = reqwest::get("http://localhost:8082").await.unwrap();
        assert_eq!(res.status(), 501);        
      
    }    

}
