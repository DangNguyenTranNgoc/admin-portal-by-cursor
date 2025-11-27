use std::sync::Arc;

use futures::StreamExt;
use rdkafka::{
    ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
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

pub async fn spawn_consumer(cfg: Arc<KafkaConfig>, _state: SharedState) -> anyhow::Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &cfg.group_id)
        .set("bootstrap.servers", &cfg.brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .create()?;

    consumer.subscribe(&[&cfg.topic])?;

    tokio::spawn(async move {
        let mut stream = consumer.stream();
        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        match serde_json::from_slice::<UserEvent>(payload) {
                            Ok(event) => {
                                info!("Received Kafka event: {:?}", event);
                                // TODO: feed into application services using `state`.
                            }
                            Err(err) => error!("Failed to parse event: {err}"),
                        }
                    }
                }
                Err(err) => error!("Kafka error: {err}"),
            }
        }
    });

    Ok(())
}
