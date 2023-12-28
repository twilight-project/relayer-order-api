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
        dotenv::dotenv().expect("Failed loading dotenv");
        let broker = std::env::var("BROKER").expect("missing environment variable BROKER");
        let producer = Producer::from_hosts(vec![broker.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();
        Mutex::new(producer)
    };
    pub static ref KAFKA_CLIENT: Mutex<KafkaClient> = {
        dotenv::dotenv().expect("Failed loading dotenv");
        let broker = std::env::var("BROKER").expect("missing environment variable BROKER");
        Mutex::new(KafkaClient::new(vec![broker.to_owned()]))
    };
}

pub fn check_kafka_topics() -> Vec<String> {
    dotenv::dotenv().expect("Failed loading dotenv");
    // kafkalib::kafka_topic::kafka_new_topic("CLIENT-REQUEST");
    let mut kafka_client = KAFKA_CLIENT.lock().unwrap();
    kafka_client.load_metadata_all().unwrap();
    let mut result: Vec<String> = Vec::new();
    for topic in kafka_client.topics() {
        for _spartition in topic.partitions() {
            // println!("{} #{} => {}",topic.name(),partition.id(),partition.leader().map(Broker::host).unwrap_or("no-leader!"));
            result.push(topic.name().to_string());
        }
    }
    result
}

pub fn send_to_kafka_queue(cmd: RpcCommand, topic: String, key: &str) {
    let mut kafka_producer = KAFKA_PRODUCER.lock().unwrap();
    let data = serde_json::to_vec(&cmd).unwrap();
    // println!("my command:{:#?}", cmd);
    // kafka_producer
    //     .send(&Record::from_value(&topic, data))
    //     .unwrap();
    // let broker = std::env::var("BROKER").expect("missing environment variable BROKER");
    kafka_producer
        .send(&Record::from_key_value(&topic, key, data))
        .unwrap();
}

pub fn receive_from_kafka_queue(
    topic: String,
    group: String,
) -> Result<Arc<Mutex<mpsc::Receiver<Message>>>, KafkaError> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let broker = vec![std::env::var("BROKER")
            .expect("missing environment variable BROKER")
            .to_owned()];
        let mut con = Consumer::from_hosts(broker)
            // .with_topic(topic)
            .with_group(group)
            .with_topic_partitions(topic, &[0])
            .with_fallback_offset(FetchOffset::Earliest)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
            .unwrap();
        loop {
            let sender_clone = sender.clone();
            let mss = con.poll().unwrap();
            if mss.is_empty() {
                // println!("No messages available right now.");
                // return Ok(());
            } else {
                for ms in mss.iter() {
                    for m in ms.messages() {
                        sender_clone
                            .send(Message {
                                offset: m.offset,
                                key: String::from_utf8_lossy(&m.key).to_string(),
                                value: serde_json::from_str(&String::from_utf8_lossy(&m.value))
                                    .unwrap(),
                            })
                            .unwrap();
                    }
                    let _ = con.consume_messageset(ms);
                }
                con.commit_consumed().unwrap();
            }
        }
    });

    Ok(Arc::new(Mutex::new(receiver)))
}

#[derive(Debug)]
pub struct Message {
    /// The offset at which this message resides in the remote kafka
    /// broker topic partition.
    pub offset: i64,
    /// The "key" data of this message.  Empty if there is no such
    /// data for this message.
    pub key: String,
    /// The value data of this message.  Empty if there is no such
    /// data for this message.
    pub value: RpcCommand,
}
