mod kafka_manager;

use crate::kafka_manager::KafkaManager;
use log::{error, info};
use rdkafka::message::Message;


pub async fn connect_producer(brokers: &str){
    info!("Connecting to kafka producer..");

    let _kafka = KafkaManager::get_instance(brokers);

    info!("kafka producer is ready at '{}'", brokers);
}

pub async fn message_producer(brokers: &str, topic: &str, key: &str, data: &str){
    let kafka = KafkaManager::get_instance(brokers);
    
    kafka.create_topics(vec![(topic, 1, 1)]).await;

    if let Err(err) = kafka.send_message(topic, key, data).await {
        error!("Failed to send message : {:?}", err);
    }
}

pub async fn consume_message<F>(brokers: &str, topic: &str, group_id: &str, handler: F)
where 
    F: Fn(String) + Send + 'static + Clone,
{
    let kafka = KafkaManager::get_instance(brokers);
    
    kafka
        .start_consumer(topic, group_id, move |msg| {
            if let Some(payload_bytes) = msg.payload_view::<str>() {
                match payload_bytes {
                    Ok(value) => handler(value.to_string()),
                    Err(_) => error!("Failed to parse kafka message payload as UTF-8"),
                }
            }
        })
        .await;
}

pub async fn disconnect_kafka(brokers: &str, topic: &str) {
    let kafka = KafkaManager::get_instance(brokers);
    kafka.delete_topics(&[topic]).await;
    info!("Kafka cleanup complete for topic: '{}'", topic);
}

fn main() {

}