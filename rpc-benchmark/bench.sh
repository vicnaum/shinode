#!/bin/bash
#
# RPC Performance Benchmark Script
# Tests eth_getLogs performance for a fixed block range
#

set -e

# Configuration
RPC_URL="http://localhost:8545"
FROM_BLOCK=24328130
TO_BLOCK=24328229          # 100 blocks (24328130 to 24328229 inclusive)
RPC_READY_TIMEOUT=60       # seconds to wait for RPC to become ready
CAST_TIMEOUT=60            # seconds timeout for cast logs command (increased for larger range)
ITERATIONS=3               # number of benchmark iterations
NODE_START_WAIT=5          # seconds to wait after starting node

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Use gtimeout on macOS, timeout on Linux
if command -v gtimeout &> /dev/null; then
    TIMEOUT_CMD="gtimeout"
elif command -v timeout &> /dev/null; then
    TIMEOUT_CMD="timeout"
else
    echo "Warning: Neither timeout nor gtimeout found, running without timeout"
    TIMEOUT_CMD=""
fi

# Parse arguments
NO_NODE=false
NO_BUILD=false
KILL_AFTER=true

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-node)
            NO_NODE=true
            KILL_AFTER=false
            shift
            ;;
        --no-build)
            NO_BUILD=true
            shift
            ;;
        --no-kill)
            KILL_AFTER=false
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--no-node] [--no-build] [--no-kill]"
            exit 1
            ;;
    esac
done

# Get current commit info
COMMIT_HASH=$(git rev-parse --short HEAD)
COMMIT_MSG=$(git log -1 --pretty=%s | head -c 50)

echo "========================================"
echo "RPC Performance Benchmark"
echo "========================================"
echo "Commit: $COMMIT_HASH - $COMMIT_MSG"
echo "Block range: $FROM_BLOCK - $TO_BLOCK"
echo "Iterations: $ITERATIONS"
echo ""

# Function to check if RPC is ready
check_rpc_ready() {
    curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "$RPC_URL" 2>/dev/null | grep -q "result"
}

# Function to wait for RPC to be ready
wait_for_rpc() {
    echo -n "Waiting for RPC to be ready..."
    local elapsed=0
    while [ $elapsed -lt $RPC_READY_TIMEOUT ]; do
        if check_rpc_ready; then
            echo -e " ${GREEN}ready${NC} (${elapsed}s)"
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
        echo -n "."
    done
    echo -e " ${RED}timeout${NC}"
    return 1
}

# Function to get time in milliseconds (portable for macOS/Linux)
get_time_ms() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use python for milliseconds
        python3 -c 'import time; print(int(time.time() * 1000))'
    else
        # Linux: use date with nanoseconds
        echo $(($(date +%s%N) / 1000000))
    fi
}

# Function to run a single benchmark
run_benchmark() {
    local start_time=$(get_time_ms)

    # Run cast logs with timeout (if available)
    local result
    local cmd_result
    if [ -n "$TIMEOUT_CMD" ]; then
        $TIMEOUT_CMD $CAST_TIMEOUT cast logs --rpc-url "$RPC_URL" --from-block $FROM_BLOCK --to-block $TO_BLOCK > /dev/null 2>&1
        cmd_result=$?
    else
        cast logs --rpc-url "$RPC_URL" --from-block $FROM_BLOCK --to-block $TO_BLOCK > /dev/null 2>&1
        cmd_result=$?
    fi

    if [ $cmd_result -eq 0 ]; then
        local end_time=$(get_time_ms)
        local duration=$((end_time - start_time))
        echo $duration
        return 0
    else
        if [ $cmd_result -eq 124 ]; then
            echo "TIMEOUT"
        else
            echo "ERROR:$cmd_result"
        fi
        return 1
    fi
}

# Function to kill any running node
kill_node() {
    pkill -f "stateless-history-node" 2>/dev/null || true
    sleep 1
}

# Function to start the node
start_node() {
    echo "Starting node..."

    # Kill any existing node first
    kill_node

    # Start node in background, redirect output to temp file
    cargo run --bin stateless-history-node --release --manifest-path node/Cargo.toml -- \
        --start-block 24320000 --shard-size 100 \
        > /tmp/node-benchmark.log 2>&1 &

    NODE_PID=$!
    echo "Node PID: $NODE_PID"

    # Wait a bit for process to start
    sleep $NODE_START_WAIT

    # Check if process is still running
    if ! kill -0 $NODE_PID 2>/dev/null; then
        echo -e "${RED}Node failed to start${NC}"
        echo "Last 20 lines of log:"
        tail -20 /tmp/node-benchmark.log
        return 1
    fi

    return 0
}

# Build if needed
if [ "$NO_BUILD" = false ] && [ "$NO_NODE" = false ]; then
    echo "Building..."
    if ! cargo build --bin stateless-history-node --release --manifest-path node/Cargo.toml 2>/tmp/build.log; then
        echo -e "${RED}Build failed${NC}"
        echo "Build log tail:"
        tail -20 /tmp/build.log
        echo ""
        echo "RESULT: [BUILD FAIL]"
        exit 1
    fi
    echo -e "${GREEN}Build successful${NC}"
    echo ""
fi

# Start node if needed
if [ "$NO_NODE" = false ]; then
    if ! start_node; then
        echo "RESULT: [START FAIL]"
        exit 1
    fi

    # Wait for RPC
    if ! wait_for_rpc; then
        echo "RESULT: [RPC TIMEOUT]"
        kill_node
        exit 1
    fi
fi

echo ""
echo "Running benchmark..."
echo ""

# Run benchmark iterations
times=()
errors=0

for i in $(seq 1 $ITERATIONS); do
    echo -n "  Iteration $i/$ITERATIONS: "
    result=$(run_benchmark)

    if [[ "$result" == "TIMEOUT" ]]; then
        echo -e "${RED}TIMEOUT (>${CAST_TIMEOUT}s)${NC}"
        errors=$((errors + 1))
    elif [[ "$result" == ERROR:* ]]; then
        echo -e "${RED}$result${NC}"
        errors=$((errors + 1))
    else
        echo -e "${GREEN}${result}ms${NC}"
        times+=($result)
    fi
done

echo ""

# Calculate results
if [ ${#times[@]} -gt 0 ]; then
    # Calculate average
    sum=0
    min=${times[0]}
    max=${times[0]}
    for t in "${times[@]}"; do
        sum=$((sum + t))
        [ $t -lt $min ] && min=$t
        [ $t -gt $max ] && max=$t
    done
    avg=$((sum / ${#times[@]}))

    echo "========================================"
    echo "Results for $COMMIT_HASH"
    echo "========================================"
    echo "  Successful runs: ${#times[@]}/$ITERATIONS"
    echo "  Min: ${min}ms"
    echo "  Max: ${max}ms"
    echo "  Avg: ${avg}ms"
    echo ""

    if [ $avg -lt 1000 ]; then
        echo -e "RESULT: ${GREEN}[OK]${NC} Time: ${avg}ms"
    elif [ $avg -lt 5000 ]; then
        echo -e "RESULT: ${YELLOW}[SLOW]${NC} Time: ${avg}ms"
    else
        echo -e "RESULT: ${RED}[VERY SLOW]${NC} Time: ${avg}ms"
    fi
else
    echo "========================================"
    echo -e "RESULT: ${RED}[NO DATA]${NC} All iterations failed"
    echo "========================================"
fi

# Cleanup
if [ "$KILL_AFTER" = true ] && [ "$NO_NODE" = false ]; then
    echo ""
    echo "Stopping node..."
    kill_node
fi

echo ""
echo "Done."
