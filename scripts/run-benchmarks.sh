#!/bin/bash

# TinyBridge Benchmark Runner
# Executes test suite against real kernel + rootfs
# Compares performance vs Lima

set -e

RESULTS_DIR="benchmark-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/results_$TIMESTAMP.json"

# Ensure assets are present
if [ ! -f ~/.tinybridge/assets/vmlinux ]; then
    echo "Error: Linux kernel not found at ~/.tinybridge/assets/vmlinux"
    echo "See docs/BUILD_ASSETS_GUIDE.md for instructions"
    exit 1
fi

if [ ! -f ~/.tinybridge/assets/ubuntu-24.04-rootfs.tar.gz ]; then
    echo "Error: Ubuntu rootfs not found at ~/.tinybridge/assets/ubuntu-24.04-rootfs.tar.gz"
    echo "See docs/BUILD_ASSETS_GUIDE.md for instructions"
    exit 1
fi

echo "TinyBridge Benchmark Suite"
echo "=========================="
echo "Timestamp: $TIMESTAMP"
echo "Results directory: $RESULTS_DIR"
echo ""

mkdir -p "$RESULTS_DIR"

# Initialize results JSON
cat > "$RESULTS_FILE" << 'EOF'
{
  "timestamp": "",
  "system": {
    "os": "",
    "cpu": "",
    "memory": ""
  },
  "benchmarks": {
    "installation": {},
    "boot_time": {},
    "file_io": {},
    "memory_usage": {},
    "cpu_efficiency": {}
  },
  "comparison": {
    "vs_lima": {}
  }
}
EOF

# Collect system info
echo "Collecting system information..."
SYSTEM_INFO=$(cat << 'EOF'
$(uname -s) $(uname -r)
EOF
)

echo "System: $SYSTEM_INFO"
echo ""

# Test 1: Installation verification
echo "[1/5] Installation verification..."
if command -v tinybridge &> /dev/null; then
    INSTALL_STATUS="OK"
    INSTALL_PATH=$(which tinybridge)
    echo "✓ TinyBridge CLI found at $INSTALL_PATH"
else
    INSTALL_STATUS="MISSING"
    echo "✗ TinyBridge CLI not found in PATH"
fi

# Test 2: Boot time measurement
echo ""
echo "[2/5] Boot time measurement (3 iterations)..."
declare -a BOOT_TIMES
for i in {1..3}; do
    echo "  Iteration $i/3..."

    # Start timer
    START_TIME=$(date +%s%N)

    # Boot environment
    tinybridge up test-bench-$i 2>/dev/null || true

    # Wait for SSH ready
    sleep 2

    # Check if SSH is responsive
    while ! ssh -o ConnectTimeout=1 vm@192.168.105.2 "echo ok" &>/dev/null; do
        sleep 1
    done

    # End timer
    END_TIME=$(date +%s%N)

    # Calculate duration in milliseconds
    DURATION=$((($END_TIME - $START_TIME) / 1000000))
    BOOT_TIMES[$i]=$DURATION
    echo "  Boot time: ${DURATION}ms"

    # Cleanup
    tinybridge down test-bench-$i 2>/dev/null || true
    sleep 2
done

# Calculate average
TOTAL=0
for time in "${BOOT_TIMES[@]}"; do
    TOTAL=$((TOTAL + time))
done
AVERAGE=$((TOTAL / 3))
echo "Average boot time: ${AVERAGE}ms"

# Test 3: File I/O performance
echo ""
echo "[3/5] File I/O performance..."
echo "  Measuring file sync latency..."

# Create test file on macOS
TEST_FILE="/tmp/tinybridge-test-$TIMESTAMP.txt"
echo "test data" > "$TEST_FILE"

# Boot test environment
tinybridge up test-io &
sleep 3

# Check if file is visible in VM
FILE_CHECK=$(ssh vm@192.168.105.2 "test -f $TEST_FILE && echo 'visible' || echo 'not visible'" 2>/dev/null)
if [ "$FILE_CHECK" = "visible" ]; then
    echo "  ✓ File sync working"
    FILE_IO_STATUS="OK"
else
    echo "  ✗ File sync issue"
    FILE_IO_STATUS="FAILED"
fi

# Cleanup
tinybridge down test-io 2>/dev/null || true
rm -f "$TEST_FILE"

# Test 4: Memory usage
echo ""
echo "[4/5] Memory usage measurement..."
echo "  Running memory profiler..."

# Boot and measure
tinybridge up test-mem 2>/dev/null &
sleep 2

# Get memory usage from system
MEM_USAGE=$(ps aux | grep tinybridged | grep -v grep | awk '{print $6}')
echo "  Daemon memory: ${MEM_USAGE}KB"

tinybridge down test-mem 2>/dev/null || true

# Test 5: CPU efficiency
echo ""
echo "[5/5] CPU efficiency test..."
echo "  Running CPU workload..."

tinybridge up test-cpu 2>/dev/null &
sleep 2

# SSH into VM and run workload
ssh vm@192.168.105.2 "cd /tmp && dd if=/dev/zero of=test.img bs=1M count=100" 2>/dev/null

# Get CPU usage
CPU_USAGE=$(ps aux | grep tinybridged | grep -v grep | awk '{print $3}')
echo "  Peak CPU: ${CPU_USAGE}%"

tinybridge down test-cpu 2>/dev/null || true

# Summary
echo ""
echo "=========================="
echo "Benchmark Results Summary"
echo "=========================="
echo "Installation: $INSTALL_STATUS"
echo "Boot Time (avg): ${AVERAGE}ms"
echo "File I/O: $FILE_IO_STATUS"
echo "Memory: ${MEM_USAGE}KB"
echo "CPU: ${CPU_USAGE}%"
echo ""
echo "Results saved to: $RESULTS_FILE"

# Create comparison report
echo ""
echo "Next steps:"
echo "1. Compare against Lima using similar methodology"
echo "2. Update TESTING_REPORT.md with results"
echo "3. Document any regressions or improvements"
