use meta_service::{ServiceMeta, ServiceRequest, ServiceResponse};
use rumqttc::{self, AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// ------------------ MQTT ------------------
async fn mqtt_polling_function(e_loop: &mut EventLoop) {
    loop {
        let event = e_loop.poll().await;
        if event.is_err() {
            panic!("Poll error: {:?}", event.err());
        }

        tokio::spawn(async move {
            match event.unwrap() {
                Event::Incoming(Packet::Publish(x)) => {
                    //ignore my own messages
                    if x.topic.contains("/proxy/") {
                        return;
                    }

                    println!("{:?}", x.payload);
                }
                _ => {}
            }
        });
    }
}

async fn mqtt_init(
    broker_url: &str,
    port: u16,
    client_id: &str,
) -> Result<AsyncClient, Box<dyn Error>> {
    let mut opts = MqttOptions::new(client_id, broker_url, port);
    opts.set_keep_alive(Duration::from_secs(5));
    opts.set_clean_session(true);
    let (client, mut e_loop) = AsyncClient::new(opts, 10);

    client.subscribe("#", QoS::AtLeastOnce).await.unwrap();

    tokio::spawn(async move {
        println!("Starting MQTT client");
        mqtt_polling_function(&mut e_loop).await;
    });

    client
        .publish("/proxy/whoareyou", QoS::AtLeastOnce, false, "Hello")
        .await
        .unwrap();

    Ok(client)
}

// ------------------- ServiceManager -------------------
#[derive(Clone)]
pub struct ServiceManager {
    db: Arc<RwLock<HashMap<String, Box<ServiceMeta>>>>,
    client: AsyncClient,
}

impl ServiceManager {
    pub async fn new() -> ServiceManager {
        let mut manager = ServiceManager {
            db: Arc::new(RwLock::new(HashMap::new())),
            client: mqtt_init("127.0.0.1", 1883, "service_manager")
                .await
                .unwrap(),
        };

        manager.mock().await;
        println!("service manager initialized");
        manager
    }

    async fn mock(&mut self) {
        let mock = Box::new(ServiceMeta::mock());

        self.db
            .write()
            .await
            .insert(mock.service_name.clone(), mock);
    }

    ///get a service to perform actions on
    pub async fn get_service(&self, name: &str) -> Option<Service> {
        let services = self.db.read().await;
        if services.contains_key(name) {
            return Some(Service {
                meta: *services[name].clone(),
            });
        }

        None
    }

    ///get all services metadata in a vector
    pub async fn get_services_meta(&self) -> Vec<ServiceMeta> {
        let services = self.db.read().await;
        services
            .iter()
            .map(|(_, service)| *service.clone())
            .collect()
    }
}

// ------------------- Service -------------------
pub struct Service {
    pub meta: ServiceMeta,
    //client: AsyncClient,
}

impl Service {
    pub async fn perform(
        &mut self,
        req: ServiceRequest,
    ) -> Result<Option<ServiceResponse>, Box<dyn Error>> {
        if !self.meta.caters(&req) {
            return Err("this service does not cater this request")?;
        }

        // let topic = format!("{}/{}", self.service.service, req.action);
        // let payload = serde_json::to_string(&req.payload)?;
        // let packet = Packet::Publish(QoS::AtLeastOnce, false, topic, payload);
        // let _ = self.client.publish(packet).await?;
        Ok(None)
    }
}
