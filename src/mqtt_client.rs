use rumqttc::{self, AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::error::Error;
use std::time::Duration;

pub struct MqttClient {
    client: AsyncClient,
    e_loop: EventLoop,
}

impl MqttClient {
    pub fn new(broker_url: &str, port: u16, client_id: &str) -> Result<Self, Box<dyn Error>> {
        let mut opts = MqttOptions::new(client_id, broker_url, port);
        opts.set_keep_alive(Duration::from_secs(5));
        opts.set_clean_session(true);
        let (client, e_loop) = AsyncClient::new(opts, 10);
        Ok(MqttClient { client, e_loop })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.subscribe("#", QoS::AtLeastOnce).await?;

        self.publish("/introduce", "oh yeah").await?;

        loop {
            let event = self.e_loop.poll().await;
            tokio::spawn(async move {
                let event = event.unwrap();
                match event {
                    Event::Incoming(Packet::Publish(x)) => {
                        println!("{:?}", x.payload);
                    }
                    _ => {}
                }
            });
        }

        Ok(())
    }

    async fn publish(&mut self, topic: &str, payload: &str) -> Result<(), Box<dyn Error>> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await?;
        Ok(())
    }
}
