use std::{net::SocketAddr, os::unix::thread, sync::Arc};

use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::RwLock, time::sleep};
use regex::Regex;

use crate::config::{ServerConfiguration, TestConfiguration};

/**
 * The ServerSetup struct is used to start and stop servers.
 * 
 * This implementation might contain a memory leak as I need to identify what happens when the server handles are stopped.
 */
pub struct ServerSetup {
    servers: Arc<RwLock<Vec<Server>>>,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl ServerSetup {
    fn new() -> Self {
        ServerSetup {
            servers: Arc::new(RwLock::new(vec![])),
            handles: vec![],
        }
    }

    async fn setup_test(&mut self, test_configuration: TestConfiguration) {
        self.stop_servers().await;
        let servers: Vec<Server> = test_configuration
            .servers
            .iter()
            .map(|server_configuration| Server::new(server_configuration.clone()))
            .collect();
        self.servers.write().await.extend(servers);
    }

    async fn start_servers(&mut self) {
        let mut handles = vec![];
        for server in self.servers.write().await.iter_mut() {
            handles.push(server.get_handle().await);
        }
        self.handles = handles;
    }

    async fn stop_servers(&mut self) {
        for handle in self.handles.iter_mut() {
            handle.abort();
        }
        self.servers.write().await.clear();
    }
}

struct Server {
    server_configuration: ServerConfiguration,
}

impl Server {
    fn new(server_configuration: ServerConfiguration) -> Self {
        Server {
            server_configuration,
        }
    }

    async fn get_handle(&mut self) -> tokio::task::JoinHandle<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.server_configuration.port));
        let server_configuration = Arc::new(self.server_configuration.clone());
        let srv = async move {
            let listener = TcpListener::bind(addr).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                let server_configuration = server_configuration.clone();
                tokio::task::spawn(async move {
                    let _ = http1::Builder::new()
                        .serve_connection(io, service_fn(|req| Self::index(req, server_configuration.clone())))
                        .with_upgrades()
                        .await;
                });
            }
        };
        tokio::task::spawn(srv)
    }

    async fn index(request: Request<hyper::body::Incoming>, server_configuration: Arc<ServerConfiguration>) -> Result<Response<Full<Bytes>>, hyper::Error> {
        for endpoint in server_configuration.endpoints.iter() {
            if is_valid_endpoint(&request, endpoint) {
                if let Some(mock_response) = &endpoint.mock_response {
                    return create_mock_response(mock_response).await.unwrap();
                }
            }
        }
        Ok(Response::builder().status(501).body(Full::from(Bytes::from("No endpoint found"))).unwrap())
    }
}

async fn create_mock_response(mock_response: &crate::config::MockResponseConfiguration) -> Option<Result<Response<Full<Bytes>>, hyper::Error>> {
    sleep(std::time::Duration::from_millis(mock_response.delay)).await;
    let mut response = Response::builder().status(mock_response.status);
    for (key, value) in mock_response.headers.iter() {
        response = response.header(key, value);
    }
    return Some(Ok(response.body(Full::from(Bytes::from(mock_response.response.clone()))).unwrap()));
}

fn is_valid_endpoint(request: &Request<hyper::body::Incoming>, endpoint: &crate::config::EndpointConfiguration) -> bool {
    let regexp = Regex::new(&endpoint.endpoint).unwrap();
    regexp.is_match(request.uri().path()) && request.method().as_str() == endpoint.method.as_str()
}

mod test {
    use std::{collections::HashMap, thread, time::Duration};

    use super::*;
    use crate::config::{EndpointConfiguration, MockResponseConfiguration, TestConfiguration};

    /**
     * Verifying that the server can be started and stopped.
     * TODO:Move this to integration tests
     */
    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_server_start_stop() {
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
        server_setup.stop_servers().await;
        thread::sleep(Duration::from_secs(1));
        let res = reqwest::get("http://localhost:8080").await;
        assert!(res.is_err());
        let res = reqwest::get("http://localhost:8081").await;
        assert!(res.is_err());        
    }

    /**
     * Verifying that the endpoints are found.
     */
    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_endpoint_ok() {
        let test_configuration = TestConfiguration::new("test".to_string(), "test".to_string(),
        vec![
            ServerConfiguration::new("test".to_string(), 8082, vec![
                EndpointConfiguration::new("/test2".to_string(), "GET".to_string(), None, Some(MockResponseConfiguration::new("{}".to_string(), 400, HashMap::new(), 1000)), None),
                EndpointConfiguration::new("/test".to_string(), "GET".to_string(), None, Some(MockResponseConfiguration::new("{}".to_string(), 200, HashMap::new(), 1000)), None),    
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
