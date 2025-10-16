use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;
use tokio::task;

use crate::app::{PerformancePoint, TestRun};

pub struct Database {
    pub path: std::path::PathBuf,
}

impl Database {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        // Ensure database and tables exist
        task::spawn_blocking({
            let path = path.clone();
            move || {
                let conn = Connection::open(&path)?;
                conn.execute_batch(include_str!("../schema.sql"))?;
                Ok::<(), rusqlite::Error>(())
            }
        }).await??;

        Ok(Self { path })
    }

    fn get_conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)?;
        Ok(conn)
    }

    pub async fn get_recent_runs(&self, limit: i64) -> Result<Vec<TestRun>> {
        let path = self.path.clone();
        let runs = task::spawn_blocking(move || {
            let conn = Connection::open(&path)?;
            let mut stmt = conn.prepare(
                "SELECT id, run_timestamp, total_tests, passed_tests, failed_tests, skipped_tests, status
                 FROM test_runs ORDER BY run_timestamp DESC LIMIT ?"
            )?;

            let runs = stmt.query_map([limit], |row| {
                Ok(TestRun {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    total_tests: row.get(2)?,
                    passed_tests: row.get(3)?,
                    failed_tests: row.get(4)?,
                    skipped_tests: row.get(5)?,
                    status: row.get(6)?,
                })
            })?;

            runs.collect::<Result<Vec<_>, _>>()
        }).await??;

        Ok(runs)
    }

    pub async fn get_performance_data(&self) -> Result<Vec<PerformancePoint>> {
        let path = self.path.clone();
        let data = task::spawn_blocking(move || {
            let conn = Connection::open(&path)?;
            let mut stmt = conn.prepare(
                "SELECT te.test_name, te.duration_seconds, te.memory_mb, tr.run_timestamp
                 FROM test_executions te
                 JOIN test_runs tr ON te.test_run_id = tr.id
                 WHERE te.status = 'passed'
                 ORDER BY tr.run_timestamp DESC, te.test_name"
            )?;

            let data = stmt.query_map([], |row| {
                Ok(PerformancePoint {
                    test_name: row.get(0)?,
                    duration: row.get(1)?,
                    memory_mb: row.get(2)?,
                    timestamp: row.get(3)?,
                })
            })?;

            data.collect::<Result<Vec<_>, _>>()
        }).await??;

        Ok(data)
    }

    pub async fn get_test_executions_for_run(&self, run_id: i64) -> Result<Vec<TestExecution>> {
        let path = self.path.clone();
        let executions = task::spawn_blocking(move || {
            let conn = Connection::open(&path)?;
            let mut stmt = conn.prepare(
                "SELECT id, test_name, status, duration_seconds, memory_mb, exit_code
                 FROM test_executions WHERE test_run_id = ? ORDER BY test_name"
            )?;

            let executions = stmt.query_map([run_id], |row| {
                Ok(TestExecution {
                    id: row.get(0)?,
                    test_name: row.get(1)?,
                    status: row.get(2)?,
                    duration: row.get(3)?,
                    memory_mb: row.get(4)?,
                    exit_code: row.get(5)?,
                })
            })?;

            executions.collect::<Result<Vec<_>, _>>()
        }).await??;

        Ok(executions)
    }
}

#[derive(Debug, Clone)]
pub struct TestExecution {
    pub id: i64,
    pub test_name: String,
    pub status: String,
    pub duration: Option<f64>,
    pub memory_mb: Option<i32>,
    pub exit_code: i32,
}