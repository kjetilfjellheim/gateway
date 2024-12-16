use std::{cell::RefCell, net::SocketAddr, sync::Arc};

use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::RwLock};

use crate::config::{ServerConfiguration, TestConfiguration};

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
        let srv = async move {
            let listener = TcpListener::bind(addr).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
    
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(Self::index))
                        .await
                    {
                        println!("Error serving connection: {:?}", err);
                    }
                });
            }
        };
        tokio::task::spawn(srv)
    }

    async fn stop(&mut self) {

    }
    async fn index(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
        Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
    }
}

mod test {
    use std::{thread, time::Duration};

    use super::*;
    use crate::config::TestConfiguration;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_server() {
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
        assert_eq!(res.status(), 200);
        let res = reqwest::get("http://localhost:8081").await.unwrap();
        assert_eq!(res.status(), 200);        
        server_setup.stop_servers().await;
        thread::sleep(Duration::from_secs(1));
        let res = reqwest::get("http://localhost:8080").await;
        assert!(res.is_err());
        let res = reqwest::get("http://localhost:8081").await;
        assert!(res.is_err());        
    }
}
