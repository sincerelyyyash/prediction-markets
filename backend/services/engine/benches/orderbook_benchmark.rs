use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;

// Import engine modules
use engine::store::market::MarketStore;
use engine::store::orderbook::spawn_orderbook_actor;
use engine::types::market_types::MarketMeta;
use engine::types::orderbook_types::{Order, OrderSide, OrderType};
use engine::types::user_types::User;

// ==================== TEST DATA GENERATION UTILITIES ====================

struct BenchmarkContext {
    orderbook: engine::store::orderbook::Orderbook,
    user_count: u64,
    market_id: u64,
}

/// Generate test users with sufficient balances and positions
fn generate_users(count: u64) -> Vec<User> {
    let mut users = Vec::new();
    for i in 1..=count {
        users.push(User {
            id: i,
            name: format!("user_{}", i),
            email: format!("user{}@test.com", i),
            balance: 100_000_000, // 100M units for high-volume testing
            positions: HashMap::new(),
        });
    }
    users
}

/// Setup benchmark context with users and market
async fn setup_benchmark_context(user_count: u64) -> BenchmarkContext {
    let market_store = MarketStore::new();
    let orderbook = spawn_orderbook_actor(market_store.clone());

    // Create and add users
    let users = generate_users(user_count);
    for user in users {
        orderbook.add_user(user).await;
    }

    // Initialize markets using MarketMeta (paired Yes/No markets)
    // For benchmarks, we use market_id 1 for Yes and 2 for No
    let yes_market_id = 1u64;
    let no_market_id = 2u64;
    let event_id = 1u64;
    let outcome_id = 1u64;

    let market_meta = MarketMeta {
        event_id,
        outcome_id,
        yes_market_id,
        no_market_id,
    };

    // Initialize markets in the orderbook (this creates the orderbook data structures)
    orderbook.init_markets(vec![market_meta]).await.ok();

    // Use yes_market_id as the canonical market_id for orders
    let market_id = yes_market_id;

    // Initialize positions for all users (on the yes market)
    for user_id in 1..=user_count {
        orderbook
            .update_position(user_id, market_id, 50_000_000)
            .await
            .ok();
    }

    BenchmarkContext {
        orderbook,
        user_count,
        market_id,
    }
}

/// Generate orders that will match (taker orders)
fn generate_matching_orders(
    user_count: u64,
    market_id: u64,
    count: usize,
    side: OrderSide,
    price: u64,
) -> Vec<Order> {
    let mut orders = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let user_id = ((i as u64) % user_count) + 1;
        orders.push(Order {
            order_id: None,
            market_id,
            user_id,
            price,
            original_qty: rng.gen_range(1..100),
            remaining_qty: rng.gen_range(1..100),
            side: side.clone(),
            order_type: OrderType::Limit,
        });
    }
    orders
}

/// Generate orders with varied prices for orderbook building
fn generate_varied_price_orders(
    user_count: u64,
    market_id: u64,
    count: usize,
    side: OrderSide,
) -> Vec<Order> {
    let mut orders = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let user_id = ((i as u64) % user_count) + 1;
        let price = match side {
            OrderSide::Bid => rng.gen_range(40..50), // Bids below mid-price
            OrderSide::Ask => rng.gen_range(51..61), // Asks above mid-price
        };

        orders.push(Order {
            order_id: None,
            market_id,
            user_id,
            price,
            original_qty: rng.gen_range(10..200),
            remaining_qty: rng.gen_range(10..200),
            side: side.clone(),
            order_type: OrderType::Limit,
        });
    }
    orders
}

// ==================== BENCHMARK SCENARIOS ====================

/// Benchmark Scenario A: Matching-Heavy Workload
/// Pre-populate orderbook with limit orders, then flood with marketable orders
fn bench_matching_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching_heavy");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    for order_count in [1_000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*order_count as u64));

        group.bench_with_input(
            BenchmarkId::new("orders", order_count),
            order_count,
            |b, &count| {
                let runtime = tokio::runtime::Runtime::new().unwrap();

                b.to_async(&runtime).iter(|| async {
                    let ctx = setup_benchmark_context(100).await;

                    // Pre-populate orderbook with asks at price 50
                    let maker_orders = generate_matching_orders(
                        ctx.user_count,
                        ctx.market_id,
                        count / 2,
                        OrderSide::Ask,
                        50,
                    );

                    for order in maker_orders {
                        ctx.orderbook.place_order(order).await.ok();
                    }

                    // Measure: Flood with matching bid orders
                    let taker_orders = generate_matching_orders(
                        ctx.user_count,
                        ctx.market_id,
                        count / 2,
                        OrderSide::Bid,
                        50,
                    );

                    let start = std::time::Instant::now();
                    for order in taker_orders {
                        black_box(ctx.orderbook.place_order(order).await.ok());
                    }
                    let duration = start.elapsed();

                    black_box(duration)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Scenario B: Orderbook-Building Workload
/// Submit non-matching limit orders at various price levels
fn bench_orderbook_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_building");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    for order_count in [1_000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*order_count as u64));

        group.bench_with_input(
            BenchmarkId::new("orders", order_count),
            order_count,
            |b, &count| {
                let runtime = tokio::runtime::Runtime::new().unwrap();

                b.to_async(&runtime).iter(|| async {
                    let ctx = setup_benchmark_context(100).await;

                    // Generate orders that won't match (spread: 40-50 bids, 51-61 asks)
                    let mut orders = Vec::new();
                    orders.extend(generate_varied_price_orders(
                        ctx.user_count,
                        ctx.market_id,
                        count / 2,
                        OrderSide::Bid,
                    ));
                    orders.extend(generate_varied_price_orders(
                        ctx.user_count,
                        ctx.market_id,
                        count / 2,
                        OrderSide::Ask,
                    ));

                    let start = std::time::Instant::now();
                    for order in orders {
                        black_box(ctx.orderbook.place_order(order).await.ok());
                    }
                    let duration = start.elapsed();

                    black_box(duration)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Scenario C: Concurrent Users at Max Capacity
/// Simulate many concurrent users with mixed order types
fn bench_concurrent_users_max_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_max_capacity");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(30);

    for user_count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*user_count as u64 * 100));

        group.bench_with_input(
            BenchmarkId::new("users", user_count),
            user_count,
            |b, &users| {
                let runtime = tokio::runtime::Runtime::new().unwrap();

                b.to_async(&runtime).iter(|| async {
                    let ctx = setup_benchmark_context(users).await;

                    // Each user submits 100 orders (50% matching, 50% non-matching)
                    let orders_per_user = 100;
                    let mut handles = Vec::new();

                    let start = std::time::Instant::now();

                    for user_id in 1..=users {
                        let orderbook = ctx.orderbook.clone();
                        let market_id = ctx.market_id;

                        let handle = tokio::spawn(async move {
                            // Use Rng::from_entropy() which is Send-safe
                            use rand::SeedableRng;
                            let mut rng = rand::rngs::StdRng::from_entropy();

                            for _ in 0..orders_per_user {
                                let should_match = rng.gen_bool(0.5);
                                let side = if rng.gen_bool(0.5) {
                                    OrderSide::Bid
                                } else {
                                    OrderSide::Ask
                                };

                                let price = if should_match {
                                    50 // Mid-price for matching
                                } else {
                                    match side {
                                        OrderSide::Bid => rng.gen_range(40..50),
                                        OrderSide::Ask => rng.gen_range(51..61),
                                    }
                                };

                                let order = Order {
                                    order_id: None,
                                    market_id,
                                    user_id,
                                    price,
                                    original_qty: rng.gen_range(10..100),
                                    remaining_qty: rng.gen_range(10..100),
                                    side,
                                    order_type: OrderType::Limit,
                                };

                                orderbook.place_order(order).await.ok();
                            }
                        });

                        handles.push(handle);
                    }

                    // Wait for all concurrent operations
                    for handle in handles {
                        handle.await.ok();
                    }

                    let duration = start.elapsed();
                    black_box(duration)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Single Order Latency (for p50, p95, p99 measurements)
fn bench_single_order_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_order_latency");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000); // Large sample for percentile accuracy

    group.bench_function("matching_order", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&runtime).iter(|| async {
            let ctx = setup_benchmark_context(10).await;

            // Place a maker order
            let maker = Order {
                order_id: None,
                market_id: ctx.market_id,
                user_id: 1,
                price: 50,
                original_qty: 100,
                remaining_qty: 100,
                side: OrderSide::Ask,
                order_type: OrderType::Limit,
            };
            ctx.orderbook.place_order(maker).await.ok();

            // Measure single taker order
            let taker = Order {
                order_id: None,
                market_id: ctx.market_id,
                user_id: 2,
                price: 50,
                original_qty: 50,
                remaining_qty: 50,
                side: OrderSide::Bid,
                order_type: OrderType::Limit,
            };

            black_box(ctx.orderbook.place_order(taker).await.ok())
        });
    });

    group.bench_function("non_matching_order", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&runtime).iter(|| async {
            let ctx = setup_benchmark_context(10).await;

            let order = Order {
                order_id: None,
                market_id: ctx.market_id,
                user_id: 1,
                price: 40,
                original_qty: 100,
                remaining_qty: 100,
                side: OrderSide::Bid,
                order_type: OrderType::Limit,
            };

            black_box(ctx.orderbook.place_order(order).await.ok())
        });
    });

    group.finish();
}

/// Benchmark: Order cancellation performance
fn bench_order_cancellation(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_cancellation");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("cancel_orders", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&runtime).iter(|| async {
            let ctx = setup_benchmark_context(10).await;

            // Place order
            let order = Order {
                order_id: None,
                market_id: ctx.market_id,
                user_id: 1,
                price: 45,
                original_qty: 100,
                remaining_qty: 100,
                side: OrderSide::Bid,
                order_type: OrderType::Limit,
            };

            let placed = ctx.orderbook.place_order(order).await.unwrap();
            let order_id = placed.order_id.unwrap();

            // Measure cancellation
            black_box(
                ctx.orderbook
                    .cancel_order(ctx.market_id, order_id)
                    .await
                    .ok(),
            )
        });
    });

    group.finish();
}

/// Benchmark: Orderbook query operations
fn bench_orderbook_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_queries");
    group.measurement_time(Duration::from_secs(10));

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once for all query benchmarks
    let ctx = runtime.block_on(async {
        let ctx = setup_benchmark_context(50).await;

        // Populate orderbook
        let orders =
            generate_varied_price_orders(ctx.user_count, ctx.market_id, 1000, OrderSide::Bid);
        for order in orders {
            ctx.orderbook.place_order(order).await.ok();
        }

        let orders =
            generate_varied_price_orders(ctx.user_count, ctx.market_id, 1000, OrderSide::Ask);
        for order in orders {
            ctx.orderbook.place_order(order).await.ok();
        }

        ctx
    });

    group.bench_function("get_best_bid", |b| {
        b.to_async(&runtime)
            .iter(|| async { black_box(ctx.orderbook.best_bid(ctx.market_id).await.ok()) });
    });

    group.bench_function("get_best_ask", |b| {
        b.to_async(&runtime)
            .iter(|| async { black_box(ctx.orderbook.best_ask(ctx.market_id).await.ok()) });
    });

    group.bench_function("get_full_orderbook", |b| {
        b.to_async(&runtime)
            .iter(|| async { black_box(ctx.orderbook.get_orderbook(ctx.market_id).await.ok()) });
    });

    group.bench_function("get_user_orders", |b| {
        b.to_async(&runtime)
            .iter(|| async { black_box(ctx.orderbook.get_user_open_orders(1).await.ok()) });
    });

    group.finish();
}

// ==================== CRITERION CONFIGURATION ====================

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_matching_heavy,
        bench_orderbook_building,
        bench_concurrent_users_max_capacity,
        bench_single_order_latency,
        bench_order_cancellation,
        bench_orderbook_queries
);

criterion_main!(benches);
