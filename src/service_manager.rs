use crate::mqtt_client::MqttClient;
use lazy_static::lazy_static;
use meta_service::ServiceMeta;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    static ref SERVICE_DB: Arc<RwLock<HashMap<String, ServiceMeta>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

// ------------------- Service -------------------

#[derive(Clone)]
pub struct ServiceManager {
    mqtt_client: Arc<MqttClient>,
}

impl ServiceManager {
    pub async fn new() -> ServiceManager {
        {
            let mut db = SERVICE_DB.write().await;
            db.insert("service_1".to_string(), ServiceMeta::mock());
        }

        ServiceManager {
            mqtt_client: Arc::new(MqttClient::new("127.0.0.1", 1883, "proxy").await.unwrap()),
        }
    }

    pub async fn register_service(service_meta: ServiceMeta) {
        let mut db = SERVICE_DB.write().await;
        db.insert(service_meta.service_name.clone(), service_meta);
    }

    pub async fn get_services_meta() -> Vec<ServiceMeta> {
        let db = SERVICE_DB.read().await;
        db.iter().map(|(_, v)| v.clone()).collect()
    }

    pub async fn make_service(&self, service_name: &str) -> Option<Service> {
        let db = SERVICE_DB.read().await;
        db.get(service_name).map(|s| Service {
            meta: s.clone(),
            mqtt_client: Arc::clone(&self.mqtt_client),
        })
    }
}

pub struct Service {
    pub meta: ServiceMeta,
    mqtt_client: Arc<MqttClient>,
}

impl Service {
    pub async fn perform(
        &mut self,
        action: &str,
        request: serde_json::Value,
    ) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        let _ = self.meta.caters(action, &request)?;
        let v = self
            .mqtt_client
            .send_and_wait(&self.meta.service_name, action, request)
            .await?;
        Ok(Some(v))
    }
}
