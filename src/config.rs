#![allow(dead_code)]
#![allow(unused_imports)]

lazy_static! {
    pub static ref RELAYER_VERSION: String =
        std::env::var("RelayerVersion").unwrap_or("0.1.0".to_string());
    pub static ref SNAPSHOT_VERSION: String = {
        match std::env::var("SnapshotVersion") {
            Ok(version) => version,
            Err(_) => "0.1.0".to_string(),
        }
    };
    pub static ref RPC_QUEUE_MODE: String = match std::env::var("RPC_QUEUE_MODE") {
        Ok(mode) => mode,
        Err(_) => "DIRECT".to_string(),
    };
    pub static ref RPC_SERVER_SOCKETADDR: String = match std::env::var("RPC_SERVER_SOCKETADDR") {
        Ok(addr) => addr,
        Err(_) => "0.0.0.0:3032".to_string(),
    };
    pub static ref RPC_SERVER_THREAD: usize = match std::env::var("RPC_SERVER_THREAD") {
        Ok(thread_count) => match thread_count.parse::<usize>() {
            Ok(thread_count_value) => thread_count_value,
            Err(_) => 2,
        },
        Err(_) => 2,
    };
}
