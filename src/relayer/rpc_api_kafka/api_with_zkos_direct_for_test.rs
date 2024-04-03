use crate::config::*;
use crate::kafkalib::kafkacmd;
// use crate::relayer::RpcCommand;
use crate::relayer::*;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_http_server::{
    hyper,
    jsonrpc_core::{MetaIoHandler, Params},
    ServerBuilder,
};
use relayerwalletlib::verify_client_message::*;

use std::collections::HashMap;
use std::time::SystemTime;

pub fn kafka_queue_rpc_server_with_zkos_test() {
    let mut io = MetaIoHandler::default();

    // CreateTraderOrder
    io.add_method_with_meta(
        "CreateTraderOrder",
        move |params: Params, meta: Meta| async move {
            let create_trader_order: CreateTraderOrder;

            match params.parse::<CreateTraderOrder>() {
                Ok(request) => create_trader_order = request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    return Err(err);
                }
            };
            let input_hex = "".to_string();
            let input_result = ZkosCreateOrder::decode_from_hex_string(input_hex);

            let zkos_request: Result<CreateTraderOrderZkos, jsonrpc_core::Error> =
                match input_result {
                    Ok(input) => Ok(CreateTraderOrderZkos::new(create_trader_order, input)),
                    Err(args) => {
                        let err = JsonRpcError::invalid_params(format!(
                            "Invalid parameters, {:?}",
                            "wrong zkos msg input".to_string()
                        ));
                        Err(err)
                    }
                };

            match zkos_request {
                Ok(ordertx) => {
                    match verify_trade_lend_order(&ordertx.input) {
                        Ok(_) => {
                            let mut order_request = ordertx.create_trader_order.clone();

                            let account_id = ordertx.input.input.as_owner_address().unwrap();
                            order_request.account_id = account_id.clone();
                            let response_clone = order_request.account_id.clone();

                            let margin = order_request.initial_margin;
                            order_request.initial_margin = margin;
                            order_request.available_margin = margin;
                            let data = RpcCommand::CreateTraderOrder(
                                order_request,
                                meta,
                                ordertx.input.encode_as_hex_string(),
                                "".to_string(),
                            );
                            //call verifier to check balance, etc...
                            //if verified the call kafkacmd::send_to_kafka_queue
                            //also convert public key into hash fn and put it in account_id field
                            kafkacmd::send_to_kafka_queue(
                                data.clone(),
                                String::from("CLIENT-REQUEST"),
                                "CreateTraderOrder",
                            );
                            println!("orderdata : {:?}", data);

                            Ok(serde_json::to_value(&RequestResponse::new(
                                "Order request submitted successfully".to_string(),
                                response_clone,
                            ))
                            .unwrap())
                        }
                        Err(arg) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                arg
                            ));
                            kafkacmd::send_to_kafka_queue_failed(
                                hex::encode(bincode::serialize(&ordertx.clone()).unwrap()),
                                String::from("CLIENT-FAILED-REQUEST"),
                                "CreateTraderOrderfailed",
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
    io.add_method_with_meta(
        "CreateLendOrder",
        move |params: Params, meta: Meta| async move {
            let create_lend_order: CreateLendOrder;

            match params.parse::<CreateLendOrder>() {
                Ok(request) => create_lend_order = request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    return Err(err);
                }
            };
            let input_hex = "".to_string();
            let input_result = ZkosCreateOrder::decode_from_hex_string(input_hex);

            let zkos_request: Result<CreateLendOrderZkos, jsonrpc_core::Error> = match input_result
            {
                Ok(input) => Ok(CreateLendOrderZkos::new(create_lend_order, input)),
                Err(args) => {
                    let err = JsonRpcError::invalid_params(format!(
                        "Invalid parameters, {:?}",
                        "wrong zkos msg input".to_string()
                    ));
                    Err(err)
                }
            };

            match zkos_request {
                Ok(ordertx) => {
                    match verify_trade_lend_order(&ordertx.input) {
                        Ok(_) => {
                            //to get public key from data
                            let mut order_request = ordertx.create_lend_order.clone();
                            let account_id = ordertx.input.input.as_owner_address().unwrap();
                            order_request.account_id = account_id.clone();
                            let response_clone = order_request.account_id.clone();

                            let deposit = order_request.deposit;
                            let balance = order_request.balance;
                            order_request.deposit = deposit;
                            order_request.balance = balance;
                            let data = RpcCommand::CreateLendOrder(
                                order_request,
                                meta,
                                ordertx.input.encode_as_hex_string(),
                                "".to_string(),
                            );
                            kafkacmd::send_to_kafka_queue(
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
                            );
                            Ok(serde_json::to_value(&RequestResponse::new(
                                "Order request submitted successfully".to_string(),
                                response_clone,
                            ))
                            .unwrap())
                        }
                        Err(arg) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                arg
                            ));
                            kafkacmd::send_to_kafka_queue_failed(
                                hex::encode(bincode::serialize(&ordertx.clone()).unwrap()),
                                String::from("CLIENT-FAILED-REQUEST"),
                                "CreateLendOrderfailed",
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

    // ExecuteTraderOrder
    io.add_method_with_meta(
        "ExecuteTraderOrder",
        move |params: Params, meta: Meta| async move {
            let execute_trader_order: ExecuteTraderOrder;

            match params.parse::<ExecuteTraderOrder>() {
                Ok(request) => execute_trader_order = request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    return Err(err);
                }
            };
            let input_hex = "".to_string();
            let input_result = ZkosSettleMsg::decode_from_hex_string(input_hex);

            let zkos_request: Result<ExecuteTraderOrderZkos, jsonrpc_core::Error> =
                match input_result {
                    Ok(input) => Ok(ExecuteTraderOrderZkos::new(execute_trader_order, input)),
                    Err(args) => {
                        let err = JsonRpcError::invalid_params(format!(
                            "Invalid parameters, {:?}",
                            "wrong zkos msg input".to_string()
                        ));
                        Err(err)
                    }
                };

            match zkos_request {
                Ok(ordertx) => match verify_settle_requests(&ordertx.msg) {
                    Ok(_) => {
                        let mut settle_request = ordertx.execute_trader_order.clone();
                        let account_id =
                            match ordertx.msg.output.as_output_data().get_owner_address() {
                                Some(address) => address.clone(),
                                None => "".to_string(),
                            };

                        settle_request.account_id = account_id.clone();

                        let data = RpcCommand::ExecuteTraderOrder(
                            settle_request,
                            meta,
                            ordertx.msg.encode_as_hex_string(),
                            "".to_string(),
                        );
                        kafkacmd::send_to_kafka_queue(
                            data,
                            String::from("CLIENT-REQUEST"),
                            "ExecuteTraderOrder",
                        );

                        Ok(serde_json::to_value(&RequestResponse::new(
                            "Execution request submitted successfully".to_string(),
                            account_id.clone(),
                        ))
                        .unwrap())
                    }
                    Err(arg) => {
                        let err =
                            JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", arg));
                        kafkacmd::send_to_kafka_queue_failed(
                            hex::encode(bincode::serialize(&ordertx.clone()).unwrap()),
                            String::from("CLIENT-FAILED-REQUEST"),
                            "ExecuteTraderOrderfailed",
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
    io.add_method_with_meta(
        "ExecuteLendOrder",
        move |params: Params, meta: Meta| async move {
            let execute_lend_order: ExecuteLendOrder;

            match params.parse::<ExecuteLendOrder>() {
                Ok(request) => execute_lend_order = request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    return Err(err);
                }
            };
            let input_hex = "".to_string();
            let input_result = ZkosSettleMsg::decode_from_hex_string(input_hex);

            let zkos_request: Result<ExecuteLendOrderZkos, jsonrpc_core::Error> = match input_result
            {
                Ok(input) => Ok(ExecuteLendOrderZkos::new(execute_lend_order, input)),
                Err(args) => {
                    let err = JsonRpcError::invalid_params(format!(
                        "Invalid parameters, {:?}",
                        "wrong zkos msg input".to_string()
                    ));
                    Err(err)
                }
            };

            match zkos_request {
                Ok(ordertx) => {
                    //to get public key from data

                    match verify_settle_requests(&ordertx.msg) {
                        Ok(_) => {
                            let mut settle_request = ordertx.execute_lend_order.clone();
                            let account_id =
                                match ordertx.msg.output.as_output_data().get_owner_address() {
                                    Some(address) => address.clone(),
                                    None => "".to_string(),
                                };
                            settle_request.account_id = account_id.clone();

                            let data = RpcCommand::ExecuteLendOrder(
                                settle_request,
                                meta,
                                ordertx.msg.encode_as_hex_string(),
                                "".to_string(),
                            );
                            kafkacmd::send_to_kafka_queue(
                                data,
                                String::from("CLIENT-REQUEST"),
                                "ExecuteLendOrder",
                            );
                            Ok(serde_json::to_value(&RequestResponse::new(
                                "Execution request submitted successfully".to_string(),
                                account_id.clone(),
                            ))
                            .unwrap())
                        }
                        Err(arg) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                arg
                            ));
                            kafkacmd::send_to_kafka_queue_failed(
                                hex::encode(bincode::serialize(&ordertx.clone()).unwrap()),
                                String::from("CLIENT-FAILED-REQUEST"),
                                "ExecuteLendOrderfailed",
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
    io.add_method_with_meta(
        "CancelTraderOrder",
        move |params: Params, meta: Meta| async move {
            let request: Result<CancelTraderOrderZkos, jsonrpc_core::Error>;
            let cancel_trader_order: CancelTraderOrder;

            match params.parse::<CancelTraderOrder>() {
                Ok(request) => cancel_trader_order = request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    return Err(err);
                }
            };
            let input_hex = "".to_string();
            let input_result = ZkosCancelMsg::decode_from_hex_string(input_hex);

            let zkos_request: Result<CancelTraderOrderZkos, jsonrpc_core::Error> =
                match input_result {
                    Ok(input) => Ok(CancelTraderOrderZkos::new(cancel_trader_order, input)),
                    Err(args) => {
                        let err = JsonRpcError::invalid_params(format!(
                            "Invalid parameters, {:?}",
                            "wrong zkos msg input".to_string()
                        ));
                        Err(err)
                    }
                };

            match zkos_request {
                Ok(ordertx) => {
                    //to get public key from data

                    match verify_query_order(
                        ordertx.msg.convert_cancel_to_query(),
                        &bincode::serialize(&ordertx.cancel_trader_order).unwrap(),
                    ) {
                        Ok(_) => {
                            let mut cancel_request = ordertx.cancel_trader_order.clone();
                            let account_id = ordertx.msg.public_key.clone();
                            // cancel_request.account_id = hex::encode(account_id.as_bytes());
                            cancel_request.account_id = account_id.clone();

                            let data = RpcCommand::CancelTraderOrder(
                                cancel_request,
                                meta,
                                ordertx.msg.encode_as_hex_string(),
                                "".to_string(),
                            );
                            kafkacmd::send_to_kafka_queue(
                                data,
                                String::from("CLIENT-REQUEST"),
                                "CancelTraderOrder",
                            );

                            Ok(serde_json::to_value(&RequestResponse::new(
                                "Cancellation request submitted successfully.".to_string(),
                                account_id.clone(),
                            ))
                            .unwrap())
                        }
                        Err(arg) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Invalid parameters, {:?}",
                                arg
                            ));
                            kafkacmd::send_to_kafka_queue_failed(
                                hex::encode(bincode::serialize(&ordertx.clone()).unwrap()),
                                String::from("CLIENT-FAILED-REQUEST"),
                                "CancelTraderOrderfailed",
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

    println!(
        "Starting jsonRPC server @ {}",
        *RPC_SERVER_SOCKETADDR_TEST_DIRECT
    );
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
        .start_http(&RPC_SERVER_SOCKETADDR_TEST_DIRECT.parse().unwrap())
        .unwrap();
    server.wait();
}
