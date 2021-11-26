//use bytes::Bytes;
use crate::ServiceManager;
use lazy_static::lazy_static;
use meta_service::ServiceMeta;
use rumqttc::{self, AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS, Transport};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

lazy_static! {
    static ref ACTIVE_REQUESTS: Arc<Mutex<HashMap<Uuid, Option<serde_json::Value>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Clone)]
pub struct MqttClient {
    client: AsyncClient,
}

async fn handle_incoming(payload: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let uuid = payload["uuid"].as_str().ok_or("No uuid in payload")?;
    let uuid = Uuid::parse_str(uuid)?;
    let mut active_requests = ACTIVE_REQUESTS.lock().await;
    let request = active_requests
        .get_mut(&uuid)
        .ok_or("No active request with this uuid")?;
    *request = Some(payload);

    Ok(())
}

async fn mqtt_polling_function(e_loop: &mut EventLoop, _client: AsyncClient) {
    loop {
        let event = e_loop.poll().await;
        if event.is_err() {
            println!("Poll error: {:?}", event.err());
            continue;
        }

        tokio::spawn(async move {
            match event.unwrap() {
                Event::Incoming(Packet::Publish(x)) => {
                    //ignore my own messages
                    if x.topic.contains("/proxy/") {
                        return;
                    }

                    if x.topic.contains("whoami") {
                        let service = serde_json::from_slice::<ServiceMeta>(&x.payload);
                        if let Ok(service) = service {
                            println!("inserting new service! {}", service.service_name);
                            ServiceManager::register_service(service).await;
                        } else {
                            println!("service not inserted! invalid structure");
                        }

                        return;
                    }

                    let payload = serde_json::from_slice::<serde_json::Value>(&x.payload);
                    if let Ok(payload) = payload {
                        if let Err(e) = handle_incoming(payload).await {
                            println!("Error handling incoming: {}", e);
                        }
                    } else {
                        println!("payload not handled! invalid structure");
                    }
                }

                _ => {}
            }
        });
    }
}

impl MqttClient {
    pub async fn new(
        broker_url: &str,
        port: u16,
        client_id: &str,
    ) -> Result<MqttClient, Box<dyn Error>> {
        let mut opts = MqttOptions::new(client_id, broker_url, port);
        opts.set_transport(Transport::Tcp);
        opts.set_keep_alive(Duration::from_secs(5));
        opts.set_clean_session(true);
        let (async_client, mut e_loop) = AsyncClient::new(opts, 10);
        async_client.subscribe("#", QoS::AtLeastOnce).await.unwrap();

        let e_client = async_client.clone();
        tokio::spawn(async move {
            println!("Starting MQTT client");
            mqtt_polling_function(&mut e_loop, e_client).await;
        });

        Ok(MqttClient {
            client: async_client,
        })
    }

    pub async fn send_and_wait(
        &self,
        service_name: &str,
        action_name: &str,
        mut payload: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        let topic = format!("/proxy/{}/{}", service_name, action_name);

        //add uuid to request
        let uuid = Uuid::new_v4();
        payload["uuid"] = serde_json::Value::String(uuid.to_string());
        {
            let mut active_requests = ACTIVE_REQUESTS.lock().await;
            active_requests.insert(uuid, None);
        }

        //send request
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.to_string())
            .await?;

        let mut val = timeout(Duration::from_secs(60), self.check_response(&uuid)).await??;
        val["uuid"] = serde_json::Value::Null;
        Ok(val)
    }

    async fn check_response(&self, uuid: &Uuid) -> Result<serde_json::Value, Box<dyn Error>> {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;
            {
                //try to get the response
                let mut active_requests = ACTIVE_REQUESTS.lock().await;
                if !active_requests.contains_key(&uuid) {
                    return Err(format!("error! {} does not exist", uuid).into());
                }

                if let Some(Some(_response)) = active_requests.get(&uuid) {
                    let val = active_requests.remove(&uuid).unwrap();
                    return Ok(val.unwrap());
                }
            }
        }
    }
}
