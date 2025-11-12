use once_cell::sync::OnceCell;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    config::ClientConfig,
    consumer::{CommitMode, Consumer, DefaultConsumerContext, StreamConsumer},
    message::OwnedMessage,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
    error::KafkaError,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{error, info};

#[derive(Clone)]
pub struct KafkaManager {
    brokers: String,
    producer: Arc<FutureProducer>,
    admin: Arc<AdminClient<DefaultClientContext>>,
    consumers: Arc<Mutex<HashMap<String, Arc<StreamConsumer<DefaultConsumerContext>>>>>,
}

static INSTANCE: OnceCell<KafkaManager> = OnceCell::new();

impl KafkaManager {
    pub fn get_instance(brokers: &str)-> &'static KafkaManager{
        INSTANCE.get_or_init(|| KafkaManager::new(brokers))
    }

    fn new(brokers: &str) -> Self {
        let producer = Arc::new(
            ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create::<FutureProducer>()
            .expect("Failed to create kafka producer"),
        );

        let admin: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .expect("Failed to create kafka admin client");

        KafkaManager {
            brokers: brokers.to_string(),
            producer,
            admin: Arc::new(admin),
            consumers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_topics(&self, topics: Vec<(&str, i32, i32)>){
        let new_topics: Vec<NewTopic> = topics
        .into_iter()
        .map(|(name, partitions, replication)|{
            NewTopic::new(name, partitions, TopicReplication::Fixed(replication))
        })
        .collect();

        let res = self
        .admin
        .create_topics(&new_topics, &AdminOptions::new())
        .await;

        match res {
            Ok(result) => info!("Topic creation result: {:?}", result),
            Err(error) => error!("Failed to create topics: {:?}", error),
        }
    }

    pub async fn send_message(&self, topic: &str, key: &str, data: &str) -> Result<(), KafkaError> {
        let record = FutureRecord::to(topic)
            .key(key)
            .payload(data);

        match self.producer.send(record, Timeout::After(Duration::from_secs(5))).await {
            Ok(delivery) => {
                info!("Message sent successfully to topic '{}': {:?}", topic, delivery);
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to send message to topic '{}': {:?}", topic, e);
                Err(e)
            }
        }
    }

    pub async fn start_consumer<F>(&self, topic: &str, group_id: &str, handler: F)
    where  
        F: Fn(OwnedMessage) + Send + 'static + Clone,
    {
    let consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", &self.brokers)
    .set("group_id", group_id)
    .set("auto.offset.reset", "earliest")
    .create()
    .expect("Failed to create consumer");

    consumer
    .subscribe(&[topic])
    .expect("Failed to subscribe to topic");

    let consumer_arc = Arc::new(consumer);

    tokio::spawn(async move {
        loop{
            match consumer_arc.recv().await {
                Ok(msg) => {
                    handler(msg.detach());
                    consumer_arc
                    .commit_message(&msg, CommitMode::Async)
                    .unwrap_or(());
                }
                Err(e) => error!("Kafka consumer error: {:?}", e),
            }
        }
    });
    }


    pub async fn delete_topics(&self, topics: &[&str]) {
        match self 
        .admin
        .delete_topics (
            topics,
            &AdminOptions::new().operation_timeout(Some(Duration::from_secs(30))),
        )
        .await
        {
            Ok(_) => info!("Topics deleted succesfully: {:?}", topics),
            Err(e) => error!("Failed to delete topics: {:?}", e),
        }
    }

}

