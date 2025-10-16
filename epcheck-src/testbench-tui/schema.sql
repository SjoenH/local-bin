-- SQLite schema for epcheck testbench results
-- This database stores test execution results, performance metrics, and validation data

-- Test runs table - each execution of the full test suite
CREATE TABLE IF NOT EXISTS test_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    total_tests INTEGER NOT NULL DEFAULT 0,
    passed_tests INTEGER NOT NULL DEFAULT 0,
    failed_tests INTEGER NOT NULL DEFAULT 0,
    skipped_tests INTEGER NOT NULL DEFAULT 0,
    log_file TEXT,
    status TEXT CHECK(status IN ('running', 'completed', 'failed')) DEFAULT 'running'
);

-- Individual test executions
CREATE TABLE IF NOT EXISTS test_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    test_run_id INTEGER NOT NULL,
    test_name TEXT NOT NULL,
    test_directory TEXT NOT NULL,
    start_time REAL,
    end_time REAL,
    duration_seconds REAL,
    memory_mb INTEGER,
    exit_code INTEGER NOT NULL DEFAULT 0,
    expected_exit_code INTEGER NOT NULL DEFAULT 0,
    status TEXT CHECK(status IN ('passed', 'failed', 'skipped', 'running')) NOT NULL,
    error_message TEXT,
    FOREIGN KEY (test_run_id) REFERENCES test_runs(id) ON DELETE CASCADE
);

-- Test output validation results
CREATE TABLE IF NOT EXISTS test_validations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    test_execution_id INTEGER NOT NULL,
    output_format TEXT NOT NULL, -- 'table', 'csv', 'json'
    validation_status TEXT CHECK(validation_status IN ('passed', 'failed')) NOT NULL,
    expected_content TEXT,
    actual_content TEXT,
    differences TEXT, -- diff output if validation failed
    FOREIGN KEY (test_execution_id) REFERENCES test_executions(id) ON DELETE CASCADE
);

-- Expected test results (static data loaded from expected-*.txt files)
CREATE TABLE IF NOT EXISTS expected_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    test_name TEXT NOT NULL,
    output_format TEXT NOT NULL, -- 'table', 'csv', 'json'
    expected_content TEXT NOT NULL,
    last_updated TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(test_name, output_format)
);

-- Performance benchmarks for comparison
CREATE TABLE IF NOT EXISTS performance_benchmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    test_name TEXT NOT NULL,
    benchmark_type TEXT NOT NULL, -- 'max_time', 'max_memory', 'avg_time', etc.
    value REAL NOT NULL,
    unit TEXT NOT NULL, -- 'seconds', 'MB', etc.
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_test_executions_test_run_id ON test_executions(test_run_id);
CREATE INDEX IF NOT EXISTS idx_test_executions_test_name ON test_executions(test_name);
CREATE INDEX IF NOT EXISTS idx_test_validations_test_execution_id ON test_validations(test_execution_id);
CREATE INDEX IF NOT EXISTS idx_expected_results_test_name ON expected_results(test_name);