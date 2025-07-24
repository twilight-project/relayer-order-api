//https://docs.rs/kafka/0.4.1/kafka/client/struct.KafkaClient.html
#![allow(dead_code)]
#![allow(unused_imports)]
use crate::relayer::RpcCommand;
use kafka::client::{metadata::Broker, FetchPartition, KafkaClient};
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::error::Error as KafkaError;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::{thread, time};

lazy_static! {
    pub static ref KAFKA_PRODUCER: Mutex<Producer> = {
        dotenv::dotenv().ok();
        let broker = match std::env::var("BROKER") {
            Ok(broker_address) => broker_address,
            Err(_) => "localhost:9092".to_string(),
        };
        let producer = Producer::from_hosts(vec![broker.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();
        Mutex::new(producer)
    };
    pub static ref KAFKA_CLIENT: Mutex<KafkaClient> = {
        dotenv::dotenv().ok();
        let broker = match std::env::var("BROKER") {
            Ok(broker_address) => broker_address,
            Err(_) => "localhost:9092".to_string(),
        };
        Mutex::new(KafkaClient::new(vec![broker.to_owned()]))
    };
}

pub fn send_to_kafka_queue(cmd: RpcCommand, topic: String, key: &str) -> Result<(), String> {
    let data = match serde_json::to_vec(&cmd) {
        Ok(data) => data,
        Err(e) => return Err(e.to_string()),
    };
    let mut kafka_producer = match KAFKA_PRODUCER.lock() {
        Ok(producer) => producer,
        Err(e) => return Err(e.to_string()),
    };

    kafka_producer
        .send(&Record::from_key_value(&topic, key, data))
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn send_to_kafka_queue_failed(cmd: String, topic: String, key: &str) -> Result<(), String> {
    let data = match serde_json::to_vec(&cmd) {
        Ok(data) => data,
        Err(e) => return Err(e.to_string()),
    };
    let mut kafka_producer = match KAFKA_PRODUCER.lock() {
        Ok(producer) => producer,
        Err(e) => return Err(e.to_string()),
    };
    kafka_producer
        .send(&Record::from_key_value(&topic, key, data))
        .map_err(|e| e.to_string())?;
    Ok(())
}
