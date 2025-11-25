# Prediction Market Backend

A high-performance, production-grade prediction market trading system built with Rust. The backend provides a complete trading engine with order matching, real-time orderbook management, and event-driven architecture.

## Overview

This backend system powers a prediction market platform where users can trade on event outcomes. It consists of multiple microservices that work together to provide low-latency order matching, real-time market data, and reliable persistence.

## Architecture

The backend is built as a modular monorepo using Rust's Cargo workspace, organized into three main services and a shared package:

- **Server**: HTTP API service handling client requests
- **Engine**: Core trading engine with orderbook and matching logic
- **DB Worker**: Database persistence and read operations handler
- **Redis Client**: Shared Redis client package for inter-service communication

### Communication Flow

```
Client → Server → Redis Streams → Engine → Redis Streams → DB Worker → PostgreSQL
                ↓                                    ↓
            Response Consumer                    Event Publisher
```

All services communicate asynchronously via Redis Streams, enabling horizontal scalability and fault tolerance.

## Services

### Server (`services/server`)

The HTTP API server built with Actix-web that handles all client-facing operations.

**Key Features:**
- RESTful API endpoints for trading operations
- JWT-based authentication for users and admins
- Middleware for authentication and authorization
- Real-time response handling via Redis Streams
- Health check endpoint

**Main Responsibilities:**
- User registration and authentication
- Order placement, cancellation, and modification
- Orderbook queries
- Position and portfolio management
- Trade history retrieval
- Event browsing and search
- Admin event management

### Engine (`services/engine`)

The core trading engine responsible for order matching and orderbook management.

**Key Features:**
- Actor-based orderbook system for concurrent order processing
- Price-time priority matching algorithm
- Support for limit and market orders
- Real-time orderbook snapshots
- Balance and position management
- Market normalization and canonical market handling
- Order splitting and merging capabilities

**Main Responsibilities:**
- Order matching and trade execution
- Orderbook state management
- Balance reservation and settlement
- Position tracking
- Market status validation
- Event publishing for database synchronization

**Performance:**
- Benchmarked for high-throughput scenarios
- Optimized for low-latency order processing
- Supports concurrent user operations

**Benchmark Results** (orderbook/engine component only):
- **Order Processing**: 57,000 - 77,000 orders/second (sustained throughput)
- **Peak Concurrent Throughput**: ~200,000 operations/second (with 100+ concurrent users)
- **Single Order Latency**: 200-217 microseconds per order
- **Order Cancellation**: ~212 microseconds per cancellation
- **Orderbook Queries**: 7-15 microseconds (best bid/ask, full orderbook, user orders)

*Note: These are orderbook-level benchmarks. End-to-end system performance includes additional latency from network, database, and Redis communication.*

### DB Worker (`services/db_worker`)

Database worker service that handles all persistence and read operations.

**Key Features:**
- Event-driven database writes
- Read request handling
- Dead letter queue for failed operations
- Database migrations support
- Transaction management

**Main Responsibilities:**
- Persisting orders, trades, and positions
- User balance updates
- Event and market data storage
- Historical data queries
- Admin and user data management

### Redis Client (`packages/redis-client`)

Shared Redis client package providing unified Redis Streams communication.

**Key Features:**
- Connection pooling
- Stream-based messaging
- Request-response pattern support
- Error handling and reconnection logic

## Features

### Trading Operations

- **Order Management**
  - Place limit and market orders
  - Cancel open orders
  - Modify existing orders
  - Split orders across multiple markets
  - Merge orders from different markets

- **Order Matching**
  - Price-time priority matching
  - Partial order fills
  - Real-time trade execution
  - Automatic balance settlement

- **Orderbook**
  - Real-time orderbook snapshots
  - Best bid/ask queries
  - Market-level orderbook views
  - Event-level aggregated orderbooks
  - Outcome-level orderbook views

### User Management

- **Authentication**
  - User registration and sign-in
  - Admin authentication
  - JWT token generation and validation
  - Password hashing with bcrypt

- **Account Management**
  - Balance tracking and updates
  - Deposit (onramp) functionality
  - Position tracking across markets
  - Portfolio management
  - Trade history

### Event Management

- **Event Operations**
  - Create prediction events
  - Update event details
  - Resolve events with winning outcomes
  - Delete events
  - Event search and filtering

- **Market Management**
  - Automatic market creation for outcomes
  - Market status tracking (Active, Closed, Resolved)
  - Last price tracking
  - Market normalization

### System Features

- **Real-time Processing**
  - Asynchronous event-driven architecture
  - Redis Streams for inter-service communication
  - Non-blocking I/O operations

- **Reliability**
  - Dead letter queue for failed operations
  - Transaction support for critical operations
  - Error handling and recovery mechanisms

- **Performance**
  - Benchmark suite for performance testing
  - Optimized data structures (BTreeMap for orderbooks)
  - Concurrent request handling

- **Observability**
  - Structured logging
  - Health check endpoints
  - Request tracing capabilities

## Technology Stack

- **Language**: Rust (Edition 2021/2024)
- **Web Framework**: Actix-web 4.11
- **Async Runtime**: Tokio
- **Database**: PostgreSQL (via SQLx)
- **Message Broker**: Redis (via Fred)
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: bcrypt
- **Serialization**: Serde
- **Benchmarking**: Criterion

## Project Structure

```
backend/
├── Cargo.toml              # Workspace configuration
├── packages/
│   └── redis-client/       # Shared Redis client package
└── services/
    ├── server/             # HTTP API service
    ├── engine/             # Trading engine
    └── db_worker/          # Database worker
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- Redis server

### Environment Variables

Create a `.env` file in each service directory with:

```env
DATABASE_URL=postgresql://user:password@localhost:5432/prediction_market
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=your-secret-key
```

### Running the Services

**Start DB Worker:**
```bash
cd services/db_worker
cargo run
```

**Start Engine:**
```bash
cd services/engine
cargo run
```

**Start Server:**
```bash
cd services/server
cargo run
```

The server will be available at `http://127.0.0.1:8000`.

### Running Benchmarks

The engine includes a comprehensive benchmark suite:

```bash
cd services/engine
./run_benchmarks.sh [options]
```

Options:
- `all` - Run all benchmarks (default)
- `matching` - Matching-heavy workload
- `building` - Orderbook-building workload
- `concurrent` - Concurrent users benchmark
- `latency` - Single order latency benchmark
- `quick` - Quick benchmarks with reduced sample size

## Performance Benchmarks

**Note:** These benchmarks measure the **orderbook/engine component** performance in isolation, not the complete end-to-end system (which includes network latency, database I/O, and Redis communication overhead).

The trading engine has been extensively benchmarked to ensure high performance under various workloads. All benchmarks were run on optimized release builds.

### Latency 

Single-order latency defines how quickly fully sequential workloads (typical for deterministically processed orderbooks) can turn around orders and trades:

- **Matching orders:** **200 microseconds** → **~5,000 trades/sec**
- **Non-matching limit orders:** **201 microseconds** → **~5,000 inserts/sec**
- **Order cancellations:** **200 microseconds** → **~5,000 cancels/sec**
- **Orderbook queries:** **7-15 microseconds** for best bid/ask, full snapshot, and user-order lookups

### Throughput Performance

**Order Processing Throughput:**
- **Matching-Heavy Workload** (orders that create trades):
  - 1,000 orders: **58,240 orders/second**
  - 10,000 orders: **62,241 orders/second**
  - 50,000 orders: **57,331 orders/second**
  - Average: **~59,000 orders/second**

- **Orderbook-Building Workload** (non-matching limit orders):
  - 1,000 orders: **66,804 orders/second**
  - 10,000 orders: **76,172 orders/second**
  - 50,000 orders: **77,178 orders/second**
  - Average: **~73,000 orders/second**

**Concurrent User Capacity:**
- 100 concurrent users: **~200,000 operations/second**
- 500 concurrent users: **~198,000 operations/second**
- 1,000 concurrent users: **~193,000 operations/second**

### Latency Performance

While sequential latency is the headline metric, the engine also maintains extremely low microsecond-level access times for read-heavy workloads:

- Best bid/ask queries: **~7.4 microseconds**
- Full orderbook snapshot: **~8.6 microseconds**
- User open orders: **~14.4 microseconds**

### Performance Summary

| Metric | Value |
|--------|-------|
| **Single Order Latency** | 200-217 microseconds (~4.6-5.0k seq orders/sec) |
| **Sustained Order Throughput** | 57,000 - 77,000 orders/sec |
| **Peak Concurrent Throughput** | ~200,000 ops/sec |
| **Order Cancellation Latency** | ~212 microseconds |
| **Orderbook Query Latency** | 7-15 microseconds |

**Important:** These benchmarks measure the **orderbook/engine component** performance in isolation. Real-world end-to-end performance will include additional overhead from:
- Network latency (HTTP requests/responses)
- Redis Streams message passing
- Database I/O operations
- Serialization/deserialization

These benchmarks demonstrate the engine's core ability to handle high-frequency trading scenarios while maintaining low latency for real-time order processing and orderbook queries at the orderbook level.

## Design Principles

- **Separation of Concerns**: Each service has a single, well-defined responsibility
- **Event-Driven**: Services communicate via events through Redis Streams
- **Actor Model**: Orderbook uses actor pattern for safe concurrent access
- **Idempotency**: Operations are designed to be safely retried
- **Performance First**: Optimized for low latency and high throughput
- **Type Safety**: Leverages Rust's type system for compile-time guarantees


