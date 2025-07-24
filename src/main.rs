#![allow(dead_code)]
#![allow(unused_imports)]
mod config;
mod kafkalib;
mod relayer;
use config::*;
use relayer::*;
use std::{thread, time};
#[macro_use]
extern crate lazy_static;
fn main() {
    dotenv::dotenv().ok();
    // Check Kafka connection before starting RPC server
    match kafkalib::kafkacmd::KAFKA_CLIENT.lock() {
        Ok(mut client) => {
            if let Err(e) = client.load_metadata_all() {
                eprintln!("Failed to connect to Kafka: {e}");
                std::process::exit(1);
            }
            println!("Successfully connected to Kafka");
            drop(client);
        }
        Err(e) => {
            eprintln!("Failed to acquire Kafka client lock: {e}");
            std::process::exit(1);
        }
    }
    if let Err(e) = rpc_server() {
        eprintln!("Failed to start rpc server: {e}");
        std::process::exit(1);
    }
}
