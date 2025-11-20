#!/bin/bash

# Engine Benchmark Runner Script
# Usage: ./run_benchmarks.sh [options]
#
# Options:
#   all              - Run all benchmarks (default)
#   matching         - Run matching-heavy workload only
#   building         - Run orderbook-building workload only
#   concurrent       - Run concurrent users benchmark only
#   latency          - Run single order latency benchmark only
#   quick            - Run quick benchmarks (reduced sample size)
#   baseline <name>  - Save results as baseline
#   compare <name>   - Compare against baseline

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print header
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Prediction Market Engine Benchmarks      ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo ""

# Navigate to engine directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Parse command line arguments
BENCHMARK_TYPE="${1:-all}"
BASELINE_NAME="${2:-}"

# Function to run benchmarks
run_benchmark() {
    local bench_name="$1"
    local bench_filter="$2"
    local extra_args="$3"
    
    echo -e "${GREEN}Running: $bench_name${NC}"
    echo "----------------------------------------"
    
    if [ -n "$extra_args" ]; then
        cargo bench --bench orderbook_benchmark "$bench_filter" -- $extra_args
    else
        cargo bench --bench orderbook_benchmark "$bench_filter"
    fi
    
    echo ""
}

# Handle different benchmark types
case "$BENCHMARK_TYPE" in
    all)
        echo -e "${YELLOW}Running all benchmarks...${NC}"
        echo ""
        cargo bench --bench orderbook_benchmark
        ;;
    
    matching)
        run_benchmark "Matching-Heavy Workload" "matching_heavy"
        ;;
    
    building)
        run_benchmark "Orderbook-Building Workload" "orderbook_building"
        ;;
    
    concurrent)
        run_benchmark "Concurrent Users Max Capacity" "concurrent_max_capacity"
        ;;
    
    latency)
        run_benchmark "Single Order Latency" "single_order_latency"
        ;;
    
    cancel)
        run_benchmark "Order Cancellation" "order_cancellation"
        ;;
    
    queries)
        run_benchmark "Orderbook Queries" "orderbook_queries"
        ;;
    
    quick)
        echo -e "${YELLOW}Running quick benchmarks (reduced sample size)...${NC}"
        echo ""
        cargo bench --bench orderbook_benchmark -- --quick
        ;;
    
    baseline)
        if [ -z "$BASELINE_NAME" ]; then
            echo -e "${RED}Error: Please provide a baseline name${NC}"
            echo "Usage: ./run_benchmarks.sh baseline <name>"
            exit 1
        fi
        echo -e "${YELLOW}Saving baseline as: $BASELINE_NAME${NC}"
        echo ""
        cargo bench --bench orderbook_benchmark -- --save-baseline "$BASELINE_NAME"
        ;;
    
    compare)
        if [ -z "$BASELINE_NAME" ]; then
            echo -e "${RED}Error: Please provide a baseline name to compare against${NC}"
            echo "Usage: ./run_benchmarks.sh compare <name>"
            exit 1
        fi
        echo -e "${YELLOW}Comparing against baseline: $BASELINE_NAME${NC}"
        echo ""
        cargo bench --bench orderbook_benchmark -- --baseline "$BASELINE_NAME"
        ;;
    
    *)
        echo -e "${RED}Unknown benchmark type: $BENCHMARK_TYPE${NC}"
        echo ""
        echo "Available options:"
        echo "  all              - Run all benchmarks (default)"
        echo "  matching         - Run matching-heavy workload only"
        echo "  building         - Run orderbook-building workload only"
        echo "  concurrent       - Run concurrent users benchmark only"
        echo "  latency          - Run single order latency benchmark only"
        echo "  cancel           - Run order cancellation benchmark only"
        echo "  queries          - Run orderbook queries benchmark only"
        echo "  quick            - Run quick benchmarks (reduced sample size)"
        echo "  baseline <name>  - Save results as baseline"
        echo "  compare <name>   - Compare against baseline"
        exit 1
        ;;
esac

# Print completion message
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Benchmarks Complete!                      ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "View detailed results at: ${BLUE}target/criterion/report/index.html${NC}"
echo ""

# Check if we should open the report
if command -v open &> /dev/null; then
    echo -e "${YELLOW}Would you like to open the HTML report? [y/N]${NC}"
    read -t 10 -n 1 -r REPLY || REPLY='n'
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open target/criterion/report/index.html
    fi
elif command -v xdg-open &> /dev/null; then
    echo -e "${YELLOW}Would you like to open the HTML report? [y/N]${NC}"
    read -t 10 -n 1 -r REPLY || REPLY='n'
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        xdg-open target/criterion/report/index.html
    fi
fi

echo -e "${BLUE}Done!${NC}"

