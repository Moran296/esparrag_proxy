use meta_service::ServiceMeta;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type ServiceDb = Arc<Mutex<HashMap<String, ServiceMeta>>>; //git copilot offered to Box the service

pub struct ServiceAction {
    service: &'static ServiceMeta,
    //mqttClient: &'static MqttClient,
}

#[derive(Debug, Clone)]
pub struct ServiceManager {
    db: ServiceDb,
    //sender MqttClient
}

impl ServiceManager {
    pub async fn new() -> ServiceManager {
        let manager = ServiceManager {
            db: Arc::new(Mutex::new(HashMap::new())),
        };

        manager
            .db
            .lock()
            .unwrap()
            .insert("mock".to_string(), ServiceMeta::mock());

        manager
    }

    // pub fn get_service(&self, name: &str) -> Option<Service> {
    //     let services = self.db.lock().unwrap();
    //     services.get(name).map(|service| service.clone())
    // }

    ///get all services in a vector
    pub fn get_services(&self) -> Vec<ServiceMeta> {
        let services = self.db.lock().unwrap();
        services.values().map(|service| service.clone()).collect()
    }
}
