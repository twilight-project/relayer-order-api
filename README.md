# Twilight Relayer Order API v0.1.0

A high-throughput JSON-RPC service for submitting, updating, and managing orders on the Twilight Relayer Core.

This API suite handles:

- Market and limit order submissions
- Order cancellation and amendment
- Lend order submission and settlement
- Limit order settlement

Built for scale and operational isolation, this service is optimized to handle large volumes of order activity without impacting the performance or reliability of the query layer. Ideal for trading bots, exchanges, and institutions interacting directly with the relayer engine.

## Features

- **Order Management**: Create, execute, and cancel trader orders
- **Kafka Integration**: Asynchronous message processing with Kafka
- **ZK-SNARK Verification**: Built-in zero-knowledge proof verification
- **Multi-threading**: Configurable RPC server threads for optimal performance
- **Docker Support**: Ready-to-deploy Docker containers

## Prerequisites

- Rust 1.87.0 or later
- Kafka broker (for message queuing)
- Git (for dependency resolution)

## Environment Setup

### Environment Variables

Create a `.env` file in the project root by copying the example:

```bash
cp .env.example .env
```

Configure the following environment variables in your `.env` file:

#### Required Variables

```env
# Relayer version identifier (REQUIRED)
RelayerVersion=1.0.0

# Kafka broker address for message queue operations (REQUIRED)
BROKER=localhost:9092
```

#### Optional Variables (with defaults)

```env
# Snapshot version for the system (default: 1.000)
SnapshotVersion=1.000

# RPC queue mode operation setting (default: DIRECT)
RPC_QUEUE_MODE=DIRECT

# RPC server socket address and port (default: 0.0.0.0:3032)
RPC_SERVER_SOCKETADDR=0.0.0.0:3032

# RPC server socket address for test direct mode (default: 0.0.0.0:3033)
RPC_SERVER_SOCKETADDR_TEST_DIRECT=0.0.0.0:3033

# Number of RPC server threads (default: 2)
RPC_SERVER_THREAD=2
```

## Building and Running

### Development Build

```bash
# Build the project
cargo build

# Run the application
cargo run
```

### Release Build

```bash
# Build optimized release version
cargo build --release

# Run the release binary
./target/release/main
```

### Docker Deployment

#### Development Container

```bash
# Build development image
docker build -f Dockerfile.dev -t relayer-order-api:dev .

# Run development container
docker run -p 3032:3032 --env-file .env relayer-order-api:dev
```

#### Production Container

```bash
# Build production image
docker build -t relayer-order-api:latest .

# Run production container
docker run -p 3032:3032 --env-file .env relayer-order-api:latest
```

## API Endpoints

The server exposes the following JSON-RPC methods:

- `CreateTraderOrder` - Submit a new trader order
- `ExecuteTraderOrder` - Execute an existing trader order
- `CancelTraderOrder` - Cancel an existing trader order
- `CreateLendOrder` - Create a lending order
- `ExecuteLendOrder` - Execute a lending order

### Example Usage

```bash
# Health check - server should respond on configured port
curl -X POST http://localhost:3032 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "CreateTraderOrder", "params": {...}, "id": 1}'
```

## Configuration

### Kafka Topics

The application uses the following Kafka topics:

- `CLIENT-REQUEST` - For incoming client requests
- `CLIENT-FAILED-REQUEST` - For failed request logging

### Server Configuration

- **Default Port**: 3032
- **Default Thread Count**: 2

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Environment configuration
├── lib.rs               # Library exports
├── kafkalib/            # Kafka integration
│   ├── mod.rs
│   └── kafkacmd.rs
└── relayer/             # Core relayer order API functionality
    ├── mod.rs
    ├── commands.rs
    └── rpc_api_kafka/
```

### Dependencies

Key dependencies include:

- `kafka` - Kafka client library
- `jsonrpc-*` - JSON-RPC server components
- `dotenv` - Environment variable management
- `twilight-relayer-sdk` - Twilight protocol SDKs

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Support

For issues and questions, please refer to the project repository or contact the development team.
