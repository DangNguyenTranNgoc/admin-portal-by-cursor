use futures::StreamExt;
use rdkafka::{
    ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::{config::KafkaConfig, domain::events::UserEvent, state::SharedState};

#[derive(Clone)]
pub struct UserEventProducer {
    inner: FutureProducer,
    topic: String,
}

impl UserEventProducer {
    pub fn new(cfg: &KafkaConfig) -> anyhow::Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &cfg.brokers)
            .create()?;

        Ok(Self {
            inner: producer,
            topic: cfg.topic.clone(),
        })
    }

    pub async fn send(&self, key: &str, event: &UserEvent) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(event)?;
        self.inner
            .send(
                FutureRecord::to(&self.topic).payload(&payload).key(key),
                Timeout::Never,
            )
            .await
            .map(|_| ())
            .map_err(|(err, _)| err.into())
    }
}
pub async fn spawn_consumer(cfg: Arc<KafkaConfig>, state: SharedState) -> anyhow::Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &cfg.group_id)
        .set("bootstrap.servers", &cfg.brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .create()?;

    consumer.subscribe(&[&cfg.topic])?;

    let state_clone = state.clone();

    tokio::spawn(async move {
        consumer
            .stream()
            .for_each_concurrent(cfg.max_concurrency_consumer, move |message| {
                let _state = state_clone.clone();

                async move {
                    match message {
                        Ok(msg) => {
                            if let Some(payload) = msg.payload() {
                                match serde_json::from_slice::<UserEvent>(payload) {
                                    Ok(event) => {
                                        info!("Kafka UserEvent: {:?}", event);

                                        // Process using application services
                                        // TODO: handle events
                                    }
                                    Err(err) => error!("Failed to parse event: {:?}", err),
                                }
                            }
                        }
                        Err(err) => {
                            error!("Kafka stream error: {:?}", err);
                        }
                    }
                }
            })
            .await;
    });

    Ok(())
}
