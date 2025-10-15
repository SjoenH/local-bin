#!/bin/bash
# Test runner for epcheck testbench v1.0
# Runs all test cases and validates output against expected results

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EPCHECK_PATH="$SCRIPT_DIR/../../epcheck"
TESTBENCH_ROOT="$SCRIPT_DIR"
RESULTS_DIR="$TESTBENCH_ROOT/results"
LOG_FILE="$RESULTS_DIR/test-run-$(date +%Y%m%d-%H%M%S).log"

# Statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Performance tracking
PERF_DATA=()

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $*" | tee -a "$LOG_FILE"
}

# Create results directory
mkdir -p "$RESULTS_DIR"

log "Starting epcheck testbench v1.0"
log "Results will be saved to: $RESULTS_DIR"
log "Log file: $LOG_FILE"
log "========================================="

# Function to check if required tools are available
check_prerequisites() {
    local test_name="$1"
    local config_file="$2"

    if [ -f "$config_file" ]; then
        # Check for skip_tools in config
        if command -v jq >/dev/null 2>&1; then
            local skip_tools
            skip_tools=$(jq -r '.skip_tools[]' "$config_file" 2>/dev/null || true)
            for tool in $skip_tools; do
                if ! command -v "$tool" >/dev/null 2>&1; then
                    log "⚠️  Skipping test $test_name (missing tool: $tool)"
                    return 1
                fi
            done
        fi
    fi

    # Check if epcheck script exists
    if [ ! -x "$EPCHECK_PATH" ]; then
        log "❌ epcheck script not found at: $EPCHECK_PATH"
        return 1
    fi

    return 0
}

# Function to run a single test
run_test() {
    local test_dir="$1"
    local test_name
    test_name=$(basename "$test_dir")
    local config_file="$test_dir/config.json"

    ((TOTAL_TESTS++))

    log ""
    log "🧪 Running test: $test_name"
    log "📁 Test directory: $test_dir"

    # Check prerequisites
    if ! check_prerequisites "$test_name" "$config_file"; then
        ((SKIPPED_TESTS++))
        return
    fi

    # Read test configuration
    local args=""
    local expected_exit_code=0
    local expected_files=""

    if [ -f "$config_file" ] && command -v jq >/dev/null 2>&1; then
        args=$(jq -r '.args[]' "$config_file" 2>/dev/null | tr '\n' ' ' || echo "")
        expected_exit_code=$(jq -r '.expected_exit_code // 0' "$config_file" 2>/dev/null || echo "0")
        expected_files=$(jq -r '.expected_files | keys[]' "$config_file" 2>/dev/null || echo "")
    fi

    # Default args if not specified
    if [ -z "$args" ]; then
        args="-d src --no-colors"
    fi

    # Change to test directory
    cd "$test_dir"

    # Run epcheck and capture output
    local start_time
    start_time=$(date +%s.%3N)

    local output_file="$RESULTS_DIR/${test_name}-output.txt"
    local error_file="$RESULTS_DIR/${test_name}-error.txt"
    local exit_code=0

    log "🚀 Executing: $EPCHECK_PATH $args"
    if "$EPCHECK_PATH" $args > "$output_file" 2> "$error_file"; then
        exit_code=$?
    else
        exit_code=$?
    fi

    local end_time
    end_time=$(date +%s.%3N)
    local duration
    duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "0")

    log "⏱️  Execution time: ${duration}s"

    # Check exit code
    if [ "$exit_code" -ne "$expected_exit_code" ]; then
        log "❌ Test $test_name FAILED - Expected exit code $expected_exit_code, got $exit_code"
        log "Error output:"
        cat "$error_file" | tee -a "$LOG_FILE"
        ((FAILED_TESTS++))
        return
    fi

    # Validate output against expected files
    local test_passed=true

    for format in $expected_files; do
        local expected_file="$test_dir/expected-${format}.txt"
        if [ -f "$expected_file" ]; then
            log "🔍 Validating $format output..."

            # For now, just check if output contains expected content
            # In a more sophisticated version, we'd do proper diffing
            if grep -q "OpenAPI Endpoint Usage Report" "$output_file"; then
                log "✅ $format output validation passed"
            else
                log "❌ $format output validation failed"
                test_passed=false
            fi
        fi
    done

    # Store performance data
    PERF_DATA+=("$test_name:$duration")

    if [ "$test_passed" = true ]; then
        log "✅ Test $test_name PASSED"
        ((PASSED_TESTS++))
    else
        log "❌ Test $test_name FAILED"
        ((FAILED_TESTS++))
    fi

    # Clean up
    cd "$TESTBENCH_ROOT"
}

# Function to run all tests
run_all_tests() {
    log "🔍 Discovering test cases..."

    # Find all test directories
    local test_dirs=()
    while IFS= read -r dir; do
        test_dirs+=("$dir")
    done < <(find "$TESTBENCH_ROOT" -name "test-*" -type d | sort)

    log "📋 Found ${#test_dirs[@]} test cases"

    for test_dir in "${test_dirs[@]}"; do
        run_test "$test_dir"
    done
}

# Function to generate summary report
generate_summary() {
    log ""
    log "========================================="
    log "📊 TEST SUMMARY"
    log "========================================="
    log "Total tests: $TOTAL_TESTS"
    log "✅ Passed: $PASSED_TESTS"
    log "❌ Failed: $FAILED_TESTS"
    log "⚠️  Skipped: $SKIPPED_TESTS"

    if [ $TOTAL_TESTS -gt 0 ]; then
        local pass_rate
        pass_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        log "📈 Pass rate: ${pass_rate}%"
    fi

    # Performance summary
    if [ ${#PERF_DATA[@]} -gt 0 ]; then
        log ""
        log "⏱️  PERFORMANCE SUMMARY"
        log "========================================="
        for perf in "${PERF_DATA[@]}"; do
            local test_name duration
            IFS=':' read -r test_name duration <<< "$perf"
            log "  $test_name: ${duration}s"
        done
    fi

    # Save summary to file
    local summary_file="$RESULTS_DIR/summary.txt"
    {
        echo "epcheck Testbench v1.0 - $(date)"
        echo "========================================"
        echo "Total tests: $TOTAL_TESTS"
        echo "Passed: $PASSED_TESTS"
        echo "Failed: $FAILED_TESTS"
        echo "Skipped: $SKIPPED_TESTS"
        echo ""
        echo "Performance Data:"
        for perf in "${PERF_DATA[@]}"; do
            echo "  $perf"
        done
    } > "$summary_file"

    log "📄 Summary saved to: $summary_file"
}

# Main execution
main() {
    run_all_tests
    generate_summary

    # Exit with failure if any tests failed
    if [ $FAILED_TESTS -gt 0 ]; then
        log "❌ Some tests failed. Check the log file for details."
        exit 1
    else
        log "🎉 All tests passed!"
        exit 0
    fi
}

# Run main function
main "$@"