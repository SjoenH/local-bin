#!/bin/bash
# Test runner for epcheck testbench v1.0
# Runs all test cases and validates output against expected results

set -e

# Colors for output (inherited from epcheck script)

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EPCHECK_PATH="/Users/henry/.local/bin/epcheck"
TESTBENCH_ROOT="$SCRIPT_DIR"
RESULTS_DIR="$TESTBENCH_ROOT/results"
DATABASE_FILE="$RESULTS_DIR/testbench.db"
SCHEMA_FILE="$SCRIPT_DIR/schema.sql"
LOG_FILE="$RESULTS_DIR/test-run-$(date +%Y%m%d-%H%M%S).log"

# Statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Performance tracking
PERF_DATA=()

# Database variables
CURRENT_TEST_RUN_ID=""

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $*" | tee -a "$LOG_FILE"
}

# Database initialization function
init_database() {
    log "🗄️  Initializing SQLite database..."

    # Create results directory if it doesn't exist
    mkdir -p "$RESULTS_DIR"

    # Check if sqlite3 is available
    if ! command -v sqlite3 >/dev/null 2>&1; then
        log "❌ sqlite3 command not found. Please install SQLite3."
        exit 1
    fi

    # Create database and tables
    if [ -f "$SCHEMA_FILE" ]; then
        if sqlite3 "$DATABASE_FILE" < "$SCHEMA_FILE"; then
            log "✅ Database initialized successfully"
        else
            log "❌ Failed to initialize database"
            exit 1
        fi
    else
        log "❌ Schema file not found: $SCHEMA_FILE"
        exit 1
    fi
}

# Function to start a new test run
start_test_run() {
    log "📝 Starting new test run..."

    # Insert new test run record
    CURRENT_TEST_RUN_ID=$(sqlite3 "$DATABASE_FILE" "
        INSERT INTO test_runs (run_timestamp, status)
        VALUES (datetime('now'), 'running');
        SELECT last_insert_rowid();
    ")

    if [ -n "$CURRENT_TEST_RUN_ID" ]; then
        log "✅ Test run started with ID: $CURRENT_TEST_RUN_ID"
    else
        log "❌ Failed to start test run"
        exit 1
    fi
}

# Function to load expected results into database
load_expected_results() {
    local test_name="$1"
    local test_dir="$2"

    log "📥 Loading expected results for $test_name..."

    for expected_file in "$test_dir"/expected-*.txt; do
        if [ -f "$expected_file" ]; then
            local format
            format=$(basename "$expected_file" | sed 's/expected-\(.*\)\.txt/\1/')

            # Read file content and escape for SQL
            local content
            content=$(sed "s/'/''/g" < "$expected_file")

            # Insert or replace expected result
            sqlite3 "$DATABASE_FILE" "
                INSERT OR REPLACE INTO expected_results (test_name, output_format, expected_content, last_updated)
                VALUES ('$test_name', '$format', '$content', datetime('now'));
            "
        fi
    done
}

# Function to complete a test run
complete_test_run() {
    local final_status="$1"

    log "📝 Completing test run..."

    # Calculate final counts from test executions
    local final_counts
    final_counts=$(sqlite3 "$DATABASE_FILE" "
        SELECT
            COUNT(*) as total,
            SUM(CASE WHEN status = 'passed' THEN 1 ELSE 0 END) as passed,
            SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed,
            SUM(CASE WHEN status = 'skipped' THEN 1 ELSE 0 END) as skipped
        FROM test_executions
        WHERE test_run_id = $CURRENT_TEST_RUN_ID;
    ")

    IFS='|' read -r TOTAL_TESTS PASSED_TESTS FAILED_TESTS SKIPPED_TESTS <<< "$final_counts"

    # Update test run with final status and counts
    sqlite3 "$DATABASE_FILE" "
        UPDATE test_runs
        SET status = '$final_status',
            total_tests = $TOTAL_TESTS,
            passed_tests = $PASSED_TESTS,
            failed_tests = $FAILED_TESTS,
            skipped_tests = $SKIPPED_TESTS,
            log_file = '$LOG_FILE'
        WHERE id = $CURRENT_TEST_RUN_ID;
    "

    log "✅ Test run completed with status: $final_status"
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

    # Load expected results into database
    load_expected_results "$test_name" "$test_dir"

    # Run epcheck and capture output
    local start_time
    start_time=$(date +%s)

    local output_content=""
    local exit_code=0

    log "🚀 Executing: $EPCHECK_PATH $args"

    # Use /usr/bin/time to capture memory usage if available
    if false && command -v /usr/bin/time >/dev/null 2>&1; then
        # Capture both stdout and stderr from time
        time_output=$(/usr/bin/time -l "$EPCHECK_PATH" "$args" 2>&1)
        exit_code=$?

        # Extract the program's stdout (everything before time statistics)
        output_content=$(echo "$time_output" | sed '/^[[:space:]]*[0-9]\+\.[0-9]/,$d' | grep -v "Using npx")

        # Extract the program's stderr (the "Using npx" line)
        # error_content=$(echo "$time_output" | grep "Using npx")  # Not currently used

        # Extract time statistics (not currently used)
        # time_content=$(echo "$time_output" | grep -A 20 "maximum resident set size")
    else
        output_content=$("$EPCHECK_PATH" "$args" 2>&1 | grep -v "Using npx")
        exit_code=$?
        memory_bytes=0
    fi

    local end_time
    end_time=$(date +%s)
    local duration
    duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "0")

    # Parse memory usage from time output (in bytes)
    # Note: Memory monitoring disabled in current implementation

    local memory_mb=0
    if [ "$memory_bytes" -gt 0 ]; then
        memory_mb=$(echo "$memory_bytes / 1024 / 1024" | bc 2>/dev/null || echo "0")
    fi

    if [ "$memory_mb" -gt 0 ]; then
        log "⏱️  Execution time: ${duration}s | 🧠 Memory: ${memory_mb}MB"
    else
        log "⏱️  Execution time: ${duration}s"
    fi

    # Insert test execution into database
    local test_execution_id
    test_execution_id=$(sqlite3 "$DATABASE_FILE" "
        INSERT INTO test_executions (
            test_run_id, test_name, test_directory, start_time, end_time,
            duration_seconds, memory_mb, exit_code, expected_exit_code, status
        ) VALUES (
            $CURRENT_TEST_RUN_ID, '$test_name', '$test_dir', $start_time, $end_time,
            $duration, $memory_mb, $exit_code, $expected_exit_code, 'running'
        );
        SELECT last_insert_rowid();
    ")

    log "📝 Test execution recorded with ID: $test_execution_id"

    # Check exit code
    if [ "$exit_code" -ne "$expected_exit_code" ]; then
        log "❌ Test $test_name FAILED - Expected exit code $expected_exit_code, got $exit_code"
        ((FAILED_TESTS++))
        return
    fi

    # Validate output against expected results in database
    local test_passed=true

    for format in $expected_files; do
        log "🔍 Validating $format output..."

        # Get expected content from database
        local expected_content
        expected_content=$(sqlite3 "$DATABASE_FILE" "
            SELECT expected_content FROM expected_results
            WHERE test_name = '$test_name' AND output_format = '$format';
        ")

        if [ -n "$expected_content" ]; then
            # Normalize actual output for comparison (remove timestamps, absolute paths)
            local normalized_output
            normalized_output=$(echo "$output_content" | sed 's/Generated on.*/Generated on [DATE]/g; s|API Spec: .*/test-[^/]*/[^ ]*|API Spec: [SPEC_PATH]|g; s|Search Dir: .*/test-[^/]*/[^ ]*|Search Dir: [DIR_PATH]|g; s|.*/testbench/v1.0/test-basic/src/||g; /DEBUG/d; /^[[:space:]]*[0-9][0-9]*\.[0-9]/,$d')

            # Compare normalized outputs
            if [ "$normalized_output" = "$expected_content" ]; then
                log "✅ $format output validation passed"

                # Insert validation result
                sqlite3 "$DATABASE_FILE" "
                    INSERT INTO test_validations (
                        test_execution_id, output_format, validation_status,
                        expected_content, actual_content
                    ) VALUES (
                        $test_execution_id, '$format', 'passed',
                        '$expected_content', '$normalized_output'
                    );
                "
            else
                log "❌ $format output validation failed"

                # Calculate differences
                local differences
                differences=$(diff -u <(echo "$expected_content") <(echo "$normalized_output") 2>/dev/null || echo "diff failed")

                # Insert validation result with failure
                sqlite3 "$DATABASE_FILE" "
                    INSERT INTO test_validations (
                        test_execution_id, output_format, validation_status,
                        expected_content, actual_content, differences
                    ) VALUES (
                        $test_execution_id, '$format', 'failed',
                        '$expected_content', '$normalized_output', '$differences'
                    );
                "

                test_passed=false
            fi
        else
            log "⚠️  No expected results found for $format format"
            test_passed=false
        fi
    done

    # Store performance data
    if [ "$memory_mb" -gt 0 ]; then
        PERF_DATA+=("$test_name:$duration:${memory_mb}MB")
    else
        PERF_DATA+=("$test_name:$duration")
    fi

    # Update test execution status in database
    if [ "$test_passed" = true ]; then
        sqlite3 "$DATABASE_FILE" "
            UPDATE test_executions SET status = 'passed' WHERE id = $test_execution_id;
        "
        log "✅ Test $test_name PASSED"
        ((PASSED_TESTS++))
    else
        sqlite3 "$DATABASE_FILE" "
            UPDATE test_executions SET status = 'failed' WHERE id = $test_execution_id;
        "
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

    # Calculate summary data from test executions
    local db_summary
    db_summary=$(sqlite3 "$DATABASE_FILE" "
        SELECT
            COUNT(*) as total,
            SUM(CASE WHEN status = 'passed' THEN 1 ELSE 0 END) as passed,
            SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed,
            SUM(CASE WHEN status = 'skipped' THEN 1 ELSE 0 END) as skipped
        FROM test_executions
        WHERE test_run_id = $CURRENT_TEST_RUN_ID;
    ")

    IFS='|' read -r TOTAL_TESTS PASSED_TESTS FAILED_TESTS SKIPPED_TESTS <<< "$db_summary"

    log "Total tests: $TOTAL_TESTS"
    log "✅ Passed: $PASSED_TESTS"
    log "❌ Failed: $FAILED_TESTS"
    log "⚠️  Skipped: $SKIPPED_TESTS"

    if [ "$TOTAL_TESTS" -gt 0 ]; then
        local pass_rate
        pass_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        log "📈 Pass rate: ${pass_rate}%"
    fi

    # Performance summary from database
    local perf_data
    perf_data=$(sqlite3 "$DATABASE_FILE" -separator ':' "
        SELECT
            test_name,
            printf('%.3f', duration_seconds),
            CASE WHEN memory_mb > 0 THEN printf('%dMB', memory_mb) ELSE '' END
        FROM test_executions
        WHERE test_run_id = $CURRENT_TEST_RUN_ID
        ORDER BY test_name;
    ")

    if [ -n "$perf_data" ]; then
        log ""
        log "⏱️  PERFORMANCE SUMMARY"
        log "========================================="
        echo "$perf_data" | while IFS=':' read -r test_name duration memory; do
            if [ -n "$memory" ]; then
                log "  $test_name: ${duration}s | ${memory}"
            else
                log "  $test_name: ${duration}s"
            fi
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
        echo "$perf_data" | while IFS=':' read -r test_name duration memory; do
            if [ -n "$memory" ]; then
                echo "  $test_name: ${duration}s | ${memory}"
            else
                echo "  $test_name: ${duration}s"
            fi
        done
    } > "$summary_file"

    log "📄 Summary saved to: $summary_file"
}

# Main execution
main() {
    # Initialize database
    init_database

    # Start test run
    start_test_run

    # Run all tests
    run_all_tests

    # Generate summary
    generate_summary

    # Complete test run
    if [ "$FAILED_TESTS" -gt 0 ]; then
        complete_test_run "failed"
        log "❌ Some tests failed. Check the log file for details."
        exit 1
    else
        complete_test_run "completed"
        log "🎉 All tests passed!"
        exit 0
    fi
}

# Database query functions

# Get recent test runs
query_recent_runs() {
    local limit="${1:-10}"
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            id,
            run_timestamp,
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            status
        FROM test_runs
        ORDER BY run_timestamp DESC
        LIMIT $limit;
    "
}

# Get test execution details for a specific run
query_test_run_details() {
    local run_id="$1"
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            te.test_name,
            te.status,
            printf('%.3f', te.duration_seconds) as duration,
            CASE WHEN te.memory_mb > 0 THEN printf('%dMB', te.memory_mb) ELSE 'N/A' END as memory,
            te.exit_code
        FROM test_executions te
        WHERE te.test_run_id = $run_id
        ORDER BY te.test_name;
    "
}

# Get validation failures for a specific run
query_validation_failures() {
    local run_id="$1"
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            te.test_name,
            tv.output_format,
            tv.validation_status,
            substr(tv.differences, 1, 100) || '...' as diff_preview
        FROM test_executions te
        JOIN test_validations tv ON te.id = tv.test_execution_id
        WHERE te.test_run_id = $run_id AND tv.validation_status = 'failed'
        ORDER BY te.test_name, tv.output_format;
    "
}

# Show usage for database queries
show_query_help() {
    cat << EOF
Database Query Functions:
  query_recent_runs [limit]     - Show recent test runs (default: 10)
  query_test_run_details <id>   - Show details for a specific test run
  query_validation_failures <id> - Show validation failures for a test run

Example usage:
  $0 && query_recent_runs 5
  $0 && query_test_run_details 1
EOF
}

# Run main function
main "$@"