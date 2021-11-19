use std::sync::Arc;

mod request_interface;
use request_interface::RequestInterface;
mod service_manager;
use service_manager::ServiceManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let service_manager = Arc::new(ServiceManager::new().await);
    let server = RequestInterface::new(service_manager.clone());
    server.run().await;

    Ok(())
}
