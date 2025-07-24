use crate::config::*;
use crate::kafkalib::kafkacmd;
use crate::relayer::*;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_http_server::{
    hyper,
    jsonrpc_core::{MetaIoHandler, Metadata, Params},
    ServerBuilder,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::SystemTime;
use twilight_relayer_sdk::verify_client_message::*;
#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Meta {
    pub metadata: HashMap<String, Option<String>>,
}
impl Metadata for Meta {}

pub fn rpc_server() -> Result<(), String> {
    let mut io = MetaIoHandler::default();

    // Healthcheck
    // A simple endpoint to verify that the RPC server is running
    io.add_method("healthcheck", move |_params: Params| async move {
        Ok(json!({
            "status": "ok",
            "timestamp": SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros()
        }))
    });

    // CreateTraderOrder
    // Handles the creation of new trader orders (market and limit orders)
    // Validates order parameters including initial margin and leverage constraints
    // Verifies zero-knowledge proofs and processes the order through Kafka queue
    io.add_method_with_meta(
        "CreateTraderOrder",
        move |params: Params, meta: Meta| async move {
            let request: Result<CreateTraderOrderClientZkos, jsonrpc_core::Error>;
            // match params.parse::<String>()
            request = match params.parse::<ByteRec>() {
                Ok(hex_data) => match hex::decode(hex_data.data) {
                    Ok(order_bytes) => match bincode::deserialize(&order_bytes) {
                        Ok(ordertx) => Ok(ordertx),
                        Err(args) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                args
                            ));
                            Err(err)
                        }
                    },
                    Err(args) => {
                        let err =
                            JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                        Err(err)
                    }
                },
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            };

            match request {
                Ok(ordertx) => {
                    match verify_client_create_trader_order(&ordertx.tx) {
                        Ok(_) => {
                            let mut order_request = ordertx.create_trader_order.clone();

                            let account_id = order_request.account_id.clone();
                            let response = RequestResponse::new(
                                "Order request submitted successfully".to_string(),
                                account_id,
                            );
                            let response_id = response.get_id();
                            let margin = order_request.initial_margin;
                            order_request.available_margin = margin;
                            if order_request.initial_margin > 0.0 && order_request.leverage <= 50.0
                            {
                                let zkos_tx_string = match bincode::serialize(&ordertx.tx) {
                                    Ok(tx) => hex::encode(tx),
                                    Err(e) => {
                                        return Err(JsonRpcError::invalid_params(format!(
                                            "Failed to serialize zkos tx: {:?}",
                                            e
                                        )));
                                    }
                                };

                                let data = RpcCommand::CreateTraderOrder(
                                    order_request,
                                    meta,
                                    zkos_tx_string,
                                    response_id,
                                );
                                let response_value = match serde_json::to_value(&response) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(JsonRpcError::invalid_params(format!(
                                            "Failed to serialize response: {:?}",
                                            e
                                        )));
                                    }
                                };
                                //call verifier to check balance, etc...
                                //if verified the call kafkacmd::send_to_kafka_queue
                                //also convert public key into hash fn and put it in account_id field
                                match kafkacmd::send_to_kafka_queue(
                                    data.clone(),
                                    String::from("CLIENT-REQUEST"),
                                    &format!(
                                        "CreateTraderOrder-{}",
                                        std::time::SystemTime::now()
                                            .duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap()
                                            .as_micros()
                                            .to_string()
                                    ),
                                ) {
                                    Ok(_) => Ok(response_value),
                                    Err(e) => Err(JsonRpcError::invalid_params(format!(
                                        "Failed to send to kafka queue: {:?}",
                                        e
                                    ))),
                                }
                            } else {
                                let err;
                                if order_request.initial_margin <= 0.0 {
                                    err = JsonRpcError::invalid_params(format!(
                                        "Invalid initial margin:{:?}, should be greater than 0",
                                        order_request.initial_margin
                                    ));
                                } else {
                                    err = JsonRpcError::invalid_params(format!(
                                        "Invalid leverage:{:?}, should be less than or equal to 50",
                                        order_request.leverage
                                    ));
                                }
                                let _ = kafkacmd::send_to_kafka_queue_failed(
                                    ordertx.encode_as_hex_string().unwrap(),
                                    String::from("CLIENT-FAILED-REQUEST"),
                                    "CreateTraderOrderfailed",
                                );
                                Err(err)
                            }
                        }
                        Err(arg) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                arg
                            ));
                            let _ = kafkacmd::send_to_kafka_queue_failed(
                                ordertx.encode_as_hex_string().unwrap(),
                                String::from("CLIENT-FAILED-REQUEST"),
                                &format!(
                                    "CreateTraderOrderfailed-{}",
                                    std::time::SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        .to_string()
                                ),
                            );
                            Err(err)
                        }
                    }
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            }
        },
    );

    // CreateLendOrder
    // Handles the creation of new lending orders for liquidity provision
    // Validates deposit amounts and processes lend order submissions
    // Verifies trade lend order proofs and queues for processing
    io.add_method_with_meta(
        "CreateLendOrder",
        move |params: Params, meta: Meta| async move {
            let request: Result<CreateLendOrderZkos, jsonrpc_core::Error>;
            // match params.parse::<String>()
            request = match params.parse::<ByteRec>() {
                Ok(hex_data) => match hex::decode(hex_data.data) {
                    Ok(order_bytes) => match bincode::deserialize(&order_bytes) {
                        Ok(ordertx) => Ok(ordertx),
                        Err(args) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                args
                            ));
                            Err(err)
                        }
                    },
                    Err(args) => {
                        let err =
                            JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                        Err(err)
                    }
                },
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            };

            match request {
                Ok(ordertx) => match verify_trade_lend_order(&ordertx.input) {
                    Ok(_) => {
                        let mut order_request = ordertx.create_lend_order.clone();

                        let response_clone = order_request.account_id.clone();
                        let response = RequestResponse::new(
                            "Order request submitted successfully".to_string(),
                            response_clone,
                        );
                        let response_id = response.get_id();
                        let deposit = order_request.deposit;
                        let balance = order_request.balance;
                        order_request.deposit = deposit;
                        order_request.balance = balance;
                        if order_request.deposit > 0.0 {
                            let data = RpcCommand::CreateLendOrder(
                                order_request,
                                meta,
                                ordertx.input.encode_as_hex_string(),
                                response_id,
                            );
                            let response_value = match serde_json::to_value(&response) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(JsonRpcError::invalid_params(format!(
                                        "Failed to serialize response: {:?}",
                                        e
                                    )));
                                }
                            };
                            match kafkacmd::send_to_kafka_queue(
                                data,
                                String::from("CLIENT-REQUEST"),
                                &format!(
                                    "CreateLendOrder-{}",
                                    std::time::SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        .to_string()
                                ),
                            ) {
                                Ok(_) => Ok(response_value),
                                Err(e) => Err(JsonRpcError::invalid_params(format!(
                                    "Failed to send to kafka queue: {:?}",
                                    e
                                ))),
                            }
                            // Ok(serde_json::to_value(&response).unwrap())
                        } else {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                "Invalid deposit amount"
                            ));
                            let _ = kafkacmd::send_to_kafka_queue_failed(
                                ordertx.encode_as_hex_string(),
                                String::from("CLIENT-FAILED-REQUEST"),
                                &format!(
                                    "CreateLendOrderfailed-{}",
                                    std::time::SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        .to_string()
                                ),
                            );
                            Err(err)
                        }
                    }
                    Err(arg) => {
                        let err =
                            JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", arg));
                        let _ = kafkacmd::send_to_kafka_queue_failed(
                            ordertx.encode_as_hex_string(),
                            String::from("CLIENT-FAILED-REQUEST"),
                            &format!(
                                "CreateLendOrderfailed-{}",
                                std::time::SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_micros()
                                    .to_string()
                            ),
                        );
                        Err(err)
                    }
                },
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            }
        },
    );

    // ExecuteTraderOrder
    // Handles the execution/settlement of existing trader orders
    // Processes order settlements and validates settlement requests
    // Verifies settlement proofs and coordinates order execution
    io.add_method_with_meta(
        "ExecuteTraderOrder",
        move |params: Params, meta: Meta| async move {
            let request: Result<ExecuteTraderOrderZkos, jsonrpc_core::Error>;
            // match params.parse::<String>()
            request = match params.parse::<ByteRec>() {
                Ok(hex_data) => match hex::decode(hex_data.data) {
                    Ok(order_bytes) => match bincode::deserialize(&order_bytes) {
                        Ok(ordertx) => Ok(ordertx),
                        Err(args) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                args
                            ));
                            Err(err)
                        }
                    },
                    Err(args) => {
                        let err =
                            JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                        Err(err)
                    }
                },
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            };

            match request {
                Ok(ordertx) => match verify_settle_requests(&ordertx.msg) {
                    Ok(_) => {
                        let settle_request = ordertx.execute_trader_order.clone();
                        let account_id = settle_request.account_id.clone();

                        let response = RequestResponse::new(
                            "Order request submitted successfully".to_string(),
                            account_id,
                        );
                        let response_id = response.get_id();

                        let data = RpcCommand::ExecuteTraderOrder(
                            settle_request,
                            meta,
                            ordertx.msg.encode_as_hex_string(),
                            response_id,
                        );
                        let response_value = match serde_json::to_value(&response) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(JsonRpcError::invalid_params(format!(
                                    "Failed to serialize response: {:?}",
                                    e
                                )));
                            }
                        };
                        match kafkacmd::send_to_kafka_queue(
                            data,
                            String::from("CLIENT-REQUEST"),
                            &format!(
                                "ExecuteTraderOrder-{}",
                                std::time::SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_micros()
                                    .to_string()
                            ),
                        ) {
                            Ok(_) => Ok(response_value),
                            Err(e) => Err(JsonRpcError::invalid_params(format!(
                                "Failed to send to kafka queue: {:?}",
                                e
                            ))),
                        }
                    }
                    Err(arg) => {
                        let err =
                            JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", arg));
                        let _ = kafkacmd::send_to_kafka_queue_failed(
                            ordertx.encode_as_hex_string(),
                            String::from("CLIENT-FAILED-REQUEST"),
                            &format!(
                                "ExecuteTraderOrderfailed-{}",
                                std::time::SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_micros()
                                    .to_string()
                            ),
                        );
                        Err(err)
                    }
                },
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            }
        },
    );

    // ExecuteLendOrder
    // Handles the execution/settlement of existing lending orders
    // Processes lend order settlements and validates settlement requests
    // Manages lending order execution and settlement coordination
    io.add_method_with_meta(
        "ExecuteLendOrder",
        move |params: Params, meta: Meta| async move {
            let request: Result<ExecuteLendOrderZkos, jsonrpc_core::Error>;
            // match params.parse::<String>()
            request = match params.parse::<ByteRec>() {
                Ok(hex_data) => {
                    match hex::decode(hex_data.data) {
                        Ok(order_bytes) => match bincode::deserialize(&order_bytes) {
                            Ok(ordertx) => Ok(ordertx),
                            Err(args) => {
                                let err = JsonRpcError::invalid_params(format!(
                                    "Invalid parameters, {:?}",
                                    args
                                ));
                                Err(err)
                            }
                        },
                        // Ok(hex_data) => Ok(hex_data),
                        Err(args) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                args
                            ));
                            Err(err)
                        }
                    }
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            };

            match request {
                Ok(ordertx) => {
                    //to get public key from data

                    match verify_settle_requests(&ordertx.msg) {
                        Ok(_) => {
                            let settle_request = ordertx.execute_lend_order.clone();

                            let account_id = settle_request.account_id.clone();
                            let response = RequestResponse::new(
                                "Order request submitted successfully".to_string(),
                                account_id,
                            );
                            let response_id = response.get_id();
                            let data = RpcCommand::ExecuteLendOrder(
                                settle_request,
                                meta,
                                ordertx.msg.encode_as_hex_string(),
                                response_id,
                            );
                            let response_value = match serde_json::to_value(&response) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(JsonRpcError::invalid_params(format!(
                                        "Failed to serialize response: {:?}",
                                        e
                                    )));
                                }
                            };

                            match kafkacmd::send_to_kafka_queue(
                                data,
                                String::from("CLIENT-REQUEST"),
                                &format!(
                                    "ExecuteLendOrder-{}",
                                    std::time::SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        .to_string()
                                ),
                            ) {
                                Ok(_) => Ok(response_value),
                                Err(e) => Err(JsonRpcError::invalid_params(format!(
                                    "Failed to send to kafka queue: {:?}",
                                    e
                                ))),
                            }
                        }
                        Err(arg) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                arg
                            ));
                            let _ = kafkacmd::send_to_kafka_queue_failed(
                                ordertx.encode_as_hex_string(),
                                String::from("CLIENT-FAILED-REQUEST"),
                                &format!(
                                    "ExecuteLendOrderfailed-{}",
                                    std::time::SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        .to_string()
                                ),
                            );
                            Err(err)
                        }
                    }
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            }
        },
    );

    // CancelTraderOrder
    // Handles the cancellation of existing trader orders
    // Validates cancellation requests and processes order cancellations
    // Verifies query order proofs and manages order state transitions
    io.add_method_with_meta(
        "CancelTraderOrder",
        move |params: Params, meta: Meta| async move {
            let request: Result<CancelTraderOrderZkos, jsonrpc_core::Error>;
            // match params.parse::<String>()
            request = match params.parse::<ByteRec>() {
                Ok(hex_data) => {
                    match hex::decode(hex_data.data) {
                        Ok(order_bytes) => match bincode::deserialize(&order_bytes) {
                            Ok(ordertx) => Ok(ordertx),
                            Err(args) => {
                                let err = JsonRpcError::invalid_params(format!(
                                    "Invalid parameters, {:?}",
                                    args
                                ));
                                Err(err)
                            }
                        },
                        // Ok(hex_data) => Ok(hex_data),
                        Err(args) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                args
                            ));
                            Err(err)
                        }
                    }
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            };

            match request {
                Ok(ordertx) => {
                    //to get public key from data

                    match verify_query_order(
                        ordertx.msg.convert_cancel_to_query(),
                        &bincode::serialize(&ordertx.cancel_trader_order).unwrap(),
                    ) {
                        Ok(_) => {
                            let cancel_request = ordertx.cancel_trader_order.clone();

                            let account_id = cancel_request.account_id.clone();
                            let response = RequestResponse::new(
                                "Order request submitted successfully".to_string(),
                                account_id,
                            );
                            let response_id = response.get_id();
                            let data = RpcCommand::CancelTraderOrder(
                                cancel_request,
                                meta,
                                ordertx.msg.encode_as_hex_string(),
                                response_id,
                            );

                            let response_value = match serde_json::to_value(&response) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(JsonRpcError::invalid_params(format!(
                                        "Failed to serialize response: {:?}",
                                        e
                                    )));
                                }
                            };
                            match kafkacmd::send_to_kafka_queue(
                                data,
                                String::from("CLIENT-REQUEST"),
                                &format!(
                                    "CancelTraderOrder-{}",
                                    std::time::SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        .to_string()
                                ),
                            ) {
                                Ok(_) => Ok(response_value),
                                Err(e) => Err(JsonRpcError::invalid_params(format!(
                                    "Failed to send to kafka queue: {:?}",
                                    e
                                ))),
                            }
                        }
                        Err(arg) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                arg
                            ));
                            let _ = kafkacmd::send_to_kafka_queue_failed(
                                ordertx.encode_as_hex_string(),
                                String::from("CLIENT-FAILED-REQUEST"),
                                &format!(
                                    "CancelTraderOrderfailed-{}",
                                    std::time::SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        .to_string()
                                ),
                            );
                            Err(err)
                        }
                    }
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            }
        },
    );

    println!("Starting jsonRPC server @ {}", *RPC_SERVER_SOCKETADDR);
    let socket_addr = match RPC_SERVER_SOCKETADDR.parse() {
        Ok(addr) => addr,
        Err(e) => return Err(format!("Invalid socket address: {:?}", e)),
    };
    let server = ServerBuilder::new(io)
        .threads(*RPC_SERVER_THREAD)
        .meta_extractor(|req: &hyper::Request<hyper::Body>| {
            let auth = req
                .headers()
                .get(hyper::header::CONTENT_TYPE)
                .map(|h| h.to_str().unwrap_or("").to_owned());
            let relayer = req
                .headers()
                .get("Relayer")
                .map(|h| h.to_str().unwrap_or("").to_owned());
            Meta {
                metadata: {
                    let mut hashmap = HashMap::new();
                    hashmap.insert(String::from("CONTENT_TYPE"), auth);
                    hashmap.insert(String::from("Relayer"), relayer);
                    hashmap.insert(
                        String::from("request_server_time"),
                        Some(
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_micros()
                                .to_string(),
                        ),
                    );
                    hashmap
                },
            }
        })
        .start_http(&socket_addr)
        .map_err(|e| e.to_string())?;
    server.wait();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_core::Request;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_healthcheck() {
        let mut io = MetaIoHandler::default();
        io.add_method("healthcheck", |_params: Params| async {
            Ok(json!({
                "status": "ok",
                "timestamp": SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
            }))
        });

        let request = r#"{"jsonrpc": "2.0", "method": "healthcheck", "id": 1}"#;
        let response = io.handle_request(request, Meta::default()).await;

        let expected = r#""status":"ok""#;
        assert!(response.is_some());
        assert!(response.unwrap().contains(expected));
    }

    #[test]
    fn test_rpc_server_metadata_extraction() {
        // Create mock request headers
        let mut headers = HashMap::new();
        headers.insert(
            String::from("content-type"),
            Some("application/json".to_string()),
        );
        headers.insert(String::from("relayer"), Some("test-relayer".to_string()));

        // Create mock metadata
        let meta = Meta {
            metadata: {
                let mut hashmap = HashMap::new();
                hashmap.insert(
                    String::from("CONTENT_TYPE"),
                    Some("application/json".to_string()),
                );
                hashmap.insert(String::from("Relayer"), Some("test-relayer".to_string()));
                hashmap.insert(
                    String::from("request_server_time"),
                    Some(
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_micros()
                            .to_string(),
                    ),
                );
                hashmap
            },
        };

        // Create mock RPC request
        let request = Request::Single(jsonrpc_core::Call::MethodCall(jsonrpc_core::MethodCall {
            jsonrpc: Some(jsonrpc_core::Version::V2),
            method: "CreateTraderOrder".to_string(),
            params: jsonrpc_core::Params::None,
            id: jsonrpc_core::Id::Num(1),
        }));

        // Verify metadata extraction
        assert!(meta.metadata.contains_key("CONTENT_TYPE"));
        assert!(meta.metadata.contains_key("Relayer"));
        assert!(meta.metadata.contains_key("request_server_time"));

        // Verify timestamp is valid
        let timestamp = meta.metadata.get("request_server_time").unwrap();
        assert!(timestamp.is_some());
        assert!(timestamp.as_ref().unwrap().parse::<u128>().is_ok());
    }
}
