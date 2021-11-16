use std::sync::Arc;

mod request_interface;
use request_interface::RequestInterface;
mod service_manager;
use service_manager::ServiceManager;
mod mqtt_client;
use mqtt_client::MqttClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let service_manager = Arc::new(ServiceManager::new().await);
    let mut client = MqttClient::new("127.0.0.1", 1883, "me").unwrap();
    let client = client.run();

    let server = RequestInterface::new(service_manager.clone());
    let server = tokio::spawn(async move {
        server.run().await;
    });

    tokio::join!(server, client);

    Ok(())
}
