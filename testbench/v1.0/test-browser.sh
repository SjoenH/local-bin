#!/bin/bash
# Test Browser CLI Tool for epcheck testbench
# Browse and compare test runs stored in SQLite database

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'

CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATABASE_FILE="$SCRIPT_DIR/results/testbench.db"

# Check if database exists
check_database() {
    if [ ! -f "$DATABASE_FILE" ]; then
        echo -e "${RED}❌ Database file not found: $DATABASE_FILE${NC}"
        echo "Run the test suite first to create the database."
        exit 1
    fi
}

# Show usage
show_usage() {
    cat << EOF
${BOLD}${CYAN}epcheck Test Browser${NC} - Browse and compare test runs

${BOLD}USAGE:${NC}
    $0 <command> [options]

${BOLD}COMMANDS:${NC}
    ${GREEN}list${NC} [limit]              List recent test runs (default: 10)
    ${GREEN}show${NC} <run_id>             Show details of a specific test run
    ${GREEN}compare${NC} <run_id1> <run_id2> Compare two test runs
    ${GREEN}failures${NC} <run_id>         Show validation failures for a test run
    ${GREEN}stats${NC}                     Show overall statistics
    ${GREEN}trends${NC} [days]             Show performance trends (default: 7 days)
    ${GREEN}export${NC} <run_id> <format>  Export test run data (json|csv)
    ${GREEN}help${NC}                      Show this help message

${BOLD}EXAMPLES:${NC}
    $0 list 5                    # Show last 5 test runs
    $0 show 1                    # Show details of test run 1
    $0 compare 1 2               # Compare test runs 1 and 2
    $0 failures 1                # Show failures in test run 1
    $0 stats                     # Show overall statistics
    $0 trends 3                  # Show trends for last 3 days
    $0 export 1 json             # Export test run 1 as JSON

${BOLD}DATABASE:${NC} $DATABASE_FILE
EOF
}

# List recent test runs
list_runs() {
    local limit="${1:-10}"
    check_database

    echo -e "${BOLD}${CYAN}Recent Test Runs${NC}"
    echo "=================="

    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            id as 'Run ID',
            run_timestamp as 'Date/Time',
            total_tests as 'Total',
            passed_tests as 'Passed',
            failed_tests as 'Failed',
            skipped_tests as 'Skipped',
            status as 'Status'
        FROM test_runs
        ORDER BY run_timestamp DESC
        LIMIT $limit;
    "
}

# Show test run details
show_run() {
    local run_id="$1"
    check_database

    if [ -z "$run_id" ]; then
        echo -e "${RED}❌ Test run ID required${NC}"
        exit 1
    fi

    # Check if run exists
    local run_exists
    run_exists=$(sqlite3 "$DATABASE_FILE" "SELECT COUNT(*) FROM test_runs WHERE id = $run_id;")
    if [ "$run_exists" -eq 0 ]; then
        echo -e "${RED}❌ Test run $run_id not found${NC}"
        exit 1
    fi

    echo -e "${BOLD}${CYAN}Test Run #$run_id Details${NC}"
    echo "======================"

    # Run summary
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            run_timestamp as 'Date/Time',
            total_tests as 'Total Tests',
            passed_tests as 'Passed',
            failed_tests as 'Failed',
            skipped_tests as 'Skipped',
            status as 'Status',
            log_file as 'Log File'
        FROM test_runs
        WHERE id = $run_id;
    "

    echo
    echo -e "${BOLD}Test Execution Details:${NC}"

    # Test executions
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            test_name as 'Test Name',
            status as 'Status',
            printf('%.2f', duration_seconds) as 'Duration (s)',
            CASE WHEN memory_mb > 0 THEN printf('%dMB', memory_mb) ELSE 'N/A' END as 'Memory',
            exit_code as 'Exit Code'
        FROM test_executions
        WHERE test_run_id = $run_id
        ORDER BY test_name;
    "
}

# Compare two test runs
compare_runs() {
    local run_id1="$1"
    local run_id2="$2"
    check_database

    if [ -z "$run_id1" ] || [ -z "$run_id2" ]; then
        echo -e "${RED}❌ Two test run IDs required${NC}"
        exit 1
    fi

    # Check if runs exist
    for run_id in "$run_id1" "$run_id2"; do
        local run_exists
        run_exists=$(sqlite3 "$DATABASE_FILE" "SELECT COUNT(*) FROM test_runs WHERE id = $run_id;")
        if [ "$run_exists" -eq 0 ]; then
            echo -e "${RED}❌ Test run $run_id not found${NC}"
            exit 1
        fi
    done

    echo -e "${BOLD}${CYAN}Comparing Test Runs #$run_id1 vs #$run_id2${NC}"
    echo "=========================================="

    # Summary comparison
    echo -e "${BOLD}Summary Comparison:${NC}"
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            'Run $run_id1' as 'Run',
            run_timestamp as 'Date/Time',
            total_tests as 'Total',
            passed_tests as 'Passed',
            failed_tests as 'Failed',
            status as 'Status'
        FROM test_runs
        WHERE id = $run_id1
        UNION ALL
        SELECT
            'Run $run_id2' as 'Run',
            run_timestamp as 'Date/Time',
            total_tests as 'Total',
            passed_tests as 'Passed',
            failed_tests as 'Failed',
            status as 'Status'
        FROM test_runs
        WHERE id = $run_id2;
    "

    echo
    echo -e "${BOLD}Test-by-Test Comparison:${NC}"

    # Test comparison
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            COALESCE(t1.test_name, t2.test_name) as 'Test Name',
            t1.status as 'Run $run_id1',
            t2.status as 'Run $run_id2',
            printf('%.2f', t1.duration_seconds) as 'Time $run_id1',
            printf('%.2f', t2.duration_seconds) as 'Time $run_id2',
            CASE
                WHEN t1.duration_seconds > 0 AND t2.duration_seconds > 0 THEN
                    printf('%.1f%%', ((t2.duration_seconds - t1.duration_seconds) / t1.duration_seconds) * 100)
                ELSE 'N/A'
            END as 'Time Change'
        FROM test_executions t1
        FULL OUTER JOIN test_executions t2 ON t1.test_name = t2.test_name
        WHERE t1.test_run_id = $run_id1 AND t2.test_run_id = $run_id2
        ORDER BY COALESCE(t1.test_name, t2.test_name);
    "
}

# Show validation failures
show_failures() {
    local run_id="$1"
    check_database

    if [ -z "$run_id" ]; then
        echo -e "${RED}❌ Test run ID required${NC}"
        exit 1
    fi

    echo -e "${BOLD}${CYAN}Validation Failures for Test Run #$run_id${NC}"
    echo "==========================================="

    local failure_count
    failure_count=$(sqlite3 "$DATABASE_FILE" "
        SELECT COUNT(*) FROM test_validations tv
        JOIN test_executions te ON tv.test_execution_id = te.id
        WHERE te.test_run_id = $run_id AND tv.validation_status = 'failed';
    ")

    if [ "$failure_count" -eq 0 ]; then
        echo -e "${GREEN}✅ No validation failures found for this test run${NC}"
        return
    fi

    echo "Found $failure_count validation failure(s):"
    echo

    sqlite3 "$DATABASE_FILE" "
        SELECT
            te.test_name || ' (' || tv.output_format || ')' as test_info,
            substr(tv.differences, 1, 200) || '...' as diff_preview
        FROM test_validations tv
        JOIN test_executions te ON tv.test_execution_id = te.id
        WHERE te.test_run_id = $run_id AND tv.validation_status = 'failed'
        ORDER BY te.test_name, tv.output_format;
    " | while IFS='|' read -r test_info diff_preview; do
        echo -e "${RED}❌ $test_info${NC}"
        echo "   $diff_preview"
        echo
    done
}

# Show overall statistics
show_stats() {
    check_database

    echo -e "${BOLD}${CYAN}Overall Test Statistics${NC}"
    echo "======================="

    # Total runs
    local total_runs
    total_runs=$(sqlite3 "$DATABASE_FILE" "SELECT COUNT(*) FROM test_runs;")

    if [ "$total_runs" -eq 0 ]; then
        echo "No test runs found."
        return
    fi

    echo "Total test runs: $total_runs"
    echo

    # Overall test counts
    sqlite3 "$DATABASE_FILE" "
        SELECT
            SUM(total_tests) as total_tests,
            SUM(passed_tests) as total_passed,
            SUM(failed_tests) as total_failed,
            SUM(skipped_tests) as total_skipped
        FROM test_runs;
    " | while IFS='|' read -r total passed failed skipped; do
        echo -e "${BOLD}Cumulative Results:${NC}"
        echo "  Total tests executed: $total"
        echo -e "  ${GREEN}Passed: $passed${NC}"
        echo -e "  ${RED}Failed: $failed${NC}"
        echo -e "  ${YELLOW}Skipped: $skipped${NC}"

        if [ "$total" -gt 0 ]; then
            local pass_rate
            pass_rate=$((passed * 100 / total))
            echo "  Pass rate: ${pass_rate}%"
        fi
    done

    echo
    echo -e "${BOLD}Recent Performance (last 10 runs):${NC}"

    # Recent performance
    sqlite3 -header -column "$DATABASE_FILE" "
        SELECT
            id as 'Run ID',
            run_timestamp as 'Date/Time',
            total_tests as 'Tests',
            passed_tests as 'Passed',
            failed_tests as 'Failed',
            status as 'Status'
        FROM test_runs
        ORDER BY run_timestamp DESC
        LIMIT 10;
    "
}

# Show performance trends
show_trends() {
    local days="${1:-7}"
    check_database

    echo -e "${BOLD}${CYAN}Performance Trends (Last $days Days)${NC}"
    echo "==================================="

    # Test execution trends
    sqlite3 "$DATABASE_FILE" "
        SELECT
            te.test_name,
            COUNT(*) as run_count,
            AVG(te.duration_seconds) as avg_duration,
            MIN(te.duration_seconds) as min_duration,
            MAX(te.duration_seconds) as max_duration,
            SUM(CASE WHEN te.status = 'passed' THEN 1 ELSE 0 END) * 100.0 / COUNT(*) as pass_rate
        FROM test_executions te
        JOIN test_runs tr ON te.test_run_id = tr.id
        WHERE tr.run_timestamp >= datetime('now', '-$days days')
        GROUP BY te.test_name
        ORDER BY te.test_name;
    " | while IFS='|' read -r test_name run_count avg_duration min_duration max_duration pass_rate; do
        echo -e "${BOLD}$test_name:${NC}"
        echo "  Runs: $run_count"
        echo "  Avg duration: ${avg_duration}s (min: ${min_duration}s, max: ${max_duration}s)"
        echo "  Pass rate: ${pass_rate}%"
        echo
    done
}

# Export test run data
export_run() {
    local run_id="$1"
    local format="$2"
    check_database

    if [ -z "$run_id" ] || [ -z "$format" ]; then
        echo -e "${RED}❌ Test run ID and format required${NC}"
        exit 1
    fi

    case "$format" in
        json)
            export_json "$run_id"
            ;;
        csv)
            export_csv "$run_id"
            ;;
        *)
            echo -e "${RED}❌ Unsupported format: $format. Use 'json' or 'csv'${NC}"
            exit 1
            ;;
    esac
}

# Export as JSON
export_json() {
    local run_id="$1"
    local output_file="test_run_${run_id}.json"

    echo -e "${CYAN}Exporting test run $run_id as JSON to $output_file...${NC}"

    # Create JSON structure
    cat > "$output_file" << EOF
{
  "test_run": {
EOF

    # Run info
    sqlite3 "$DATABASE_FILE" -json "
        SELECT
            id, run_timestamp, total_tests, passed_tests, failed_tests,
            skipped_tests, status, log_file
        FROM test_runs
        WHERE id = $run_id;
    " | sed 's/^\[//' | sed 's/\]$//' >> "$output_file"

    {
        echo '  },'
        echo '  "test_executions": ['
    } >> "$output_file"

    # Test executions
    sqlite3 "$DATABASE_FILE" -json "
        SELECT
            test_name, status, duration_seconds, memory_mb, exit_code
        FROM test_executions
        WHERE test_run_id = $run_id
        ORDER BY test_name;
    " | sed 's/^\[//' | sed 's/\]$//' >> "$output_file"

    {
        echo '  ],'
        echo '  "validations": ['
    } >> "$output_file"

    # Validations
    sqlite3 "$DATABASE_FILE" -json "
        SELECT
            tv.output_format, tv.validation_status,
            substr(tv.differences, 1, 500) as differences
        FROM test_validations tv
        JOIN test_executions te ON tv.test_execution_id = te.id
        WHERE te.test_run_id = $run_id
        ORDER BY te.test_name, tv.output_format;
    " | sed 's/^\[//' | sed 's/\]$//' >> "$output_file"

    echo '  ]' >> "$output_file"
    echo '}' >> "$output_file"

    echo -e "${GREEN}✅ Exported to $output_file${NC}"
}

# Export as CSV
export_csv() {
    local run_id="$1"
    local output_file="test_run_${run_id}.csv"

    echo -e "${CYAN}Exporting test run $run_id as CSV to $output_file...${NC}"

    # CSV header
    cat > "$output_file" << EOF
Test Run ID,Test Name,Status,Duration (s),Memory (MB),Exit Code,Validation Format,Validation Status
EOF

    # CSV data
    sqlite3 -csv "$DATABASE_FILE" "
        SELECT
            te.test_run_id,
            te.test_name,
            te.status,
            te.duration_seconds,
            te.memory_mb,
            te.exit_code,
            COALESCE(tv.output_format, ''),
            COALESCE(tv.validation_status, '')
        FROM test_executions te
        LEFT JOIN test_validations tv ON te.id = tv.test_execution_id
        WHERE te.test_run_id = $run_id
        ORDER BY te.test_name, tv.output_format;
    " >> "$output_file"

    echo -e "${GREEN}✅ Exported to $output_file${NC}"
}

# Main command dispatcher
main() {
    local command="$1"
    shift

    case "$command" in
        list)
            list_runs "$@"
            ;;
        show)
            show_run "$@"
            ;;
        compare)
            compare_runs "$@"
            ;;
        failures)
            show_failures "$@"
            ;;
        stats)
            show_stats "$@"
            ;;
        trends)
            show_trends "$@"
            ;;
        export)
            export_run "$@"
            ;;
        help|--help|-h)
            show_usage
            ;;
        "")
            show_usage
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $command${NC}"
            echo
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"