//! # Relayer Order API
//!
//! A JSON-RPC server implementation for handling trading and lending orders in a decentralized exchange.
//!
//! This crate provides functionality for:
//! - Processing trader orders (create, execute, cancel)
//! - Managing lending orders
//! - Message verification using the Twilight Relayer SDK
//! - Kafka integration for request handling
//!
//! ## Features
//!
//! - JSON-RPC server with configurable thread count and socket address
//! - Support for trader and lender order operations
//! - Message verification using twilight-relayer-sdk
//! - Kafka integration for request queuing and failure logging
//! - Environment-based configuration
//!
//! ## RPC Methods
//!
//! ### `CreateTraderOrder`
//! Creates a new trader order with margin and leverage validation.
//!
//! Parameters:
//! - Hex-encoded bytes containing serialized order data (`ByteRec`)
//! - Includes: account ID, initial margin, leverage
//!
//! Validation:
//! - Initial margin must be > 0.0
//! - Leverage must be <= 50.0
//! - Message signature verification via `verify_client_create_trader_order`
//!
//! ### `CreateLendOrder`
//! Creates a new lending order.
//!
//! Parameters:
//! - Hex-encoded bytes containing serialized order data (`ByteRec`)
//! - Includes: account ID, deposit amount, balance
//!
//! Validation:
//! - Deposit must be > 0.0
//! - Message verification via `verify_trade_lend_order`
//!
//! ### `ExecuteTraderOrder`
//! Executes a previously created trader order.
//!
//! Parameters:
//! - Hex-encoded bytes containing serialized execution data (`ByteRec`)
//!
//! Validation:
//! - Message verification via `verify_settle_requests`
//!
//! ### `ExecuteLendOrder`
//! Executes a lending order.
//!
//! Parameters:
//! - Hex-encoded bytes containing serialized execution data (`ByteRec`)
//!
//! Validation:
//! - Message verification via `verify_settle_requests`
//!
//! ### `CancelTraderOrder`
//! Cancels an existing trader order.
//!
//! Parameters:
//! - Hex-encoded bytes containing serialized cancellation data (`ByteRec`)
//!
//! Validation:
//! - Message verification via `verify_query_order`
//!
//! ## Error Handling
//!
//! All methods return `Result<Value, JsonRpcError>` where errors include:
//! - Invalid parameter format
//! - Failed message verification
//! - Invalid business rules (e.g. margin/leverage limits)
//!
//! Failed requests are logged to the "CLIENT-FAILED-REQUEST" Kafka topic.
//!
//! ## Example
//!
//! ```no_run
//! use relayer_order_api::relayer::rpc_server;
//!
//! fn main() {
//!     // Start the RPC server
//!     rpc_server();
//! }
//! ```
//!
//! ## Configuration
//!
//! The server can be configured via environment variables:
//!
//! - `RPC_SERVER_SOCKETADDR`: Socket address (default: "0.0.0.0:3032")
//! - `RPC_SERVER_THREAD`: Thread count (default: 5)
//! - `RPC_QUEUE_MODE`: Queue mode (default: "DIRECT")
pub mod config;
pub mod kafkalib;
pub mod relayer;

#[macro_use]
extern crate lazy_static;
