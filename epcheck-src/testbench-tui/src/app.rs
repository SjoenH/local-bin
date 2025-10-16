use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Chart, Dataset, Gauge, GraphType, List, ListItem, ListState, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::db::Database;

#[derive(Debug)]
enum TestResult {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrentScreen {
    Main,
    Help,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub current_tab: usize,
    pub testbench_path: PathBuf,
    pub epcheck_path: PathBuf,
    pub db: Database,
    pub error_message: Option<String>,
    pub test_runs: Vec<TestRun>,
    pub selected_run: ListState,
    pub overview_table_state: TableState,
    pub performance_data: Vec<PerformancePoint>,
}

#[derive(Debug, Clone)]
pub struct TestRun {
    pub id: i64,
    pub timestamp: String,
    pub total_tests: i32,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub skipped_tests: i32,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct PerformancePoint {
    pub test_name: String,
    pub duration: f64,
    pub memory_mb: Option<i32>,
    pub timestamp: String,
}

impl App {
    pub async fn new(testbench_path: String, epcheck_path: String) -> Result<Self> {
        let testbench_path = PathBuf::from(testbench_path);
        let epcheck_path = PathBuf::from(epcheck_path);

        let db_path = testbench_path.join("results").join("testbench.db");
        let db = Database::new(&db_path).await?;

        let test_runs = db.get_recent_runs(20).await?;
        let performance_data = db.get_performance_data().await?;

        Ok(Self {
            current_screen: CurrentScreen::Main,
            current_tab: 0,
            testbench_path,
            epcheck_path,
            db,
            error_message: None,
            test_runs,
            selected_run: ListState::default(),
            overview_table_state: TableState::default(),
            performance_data,
        })
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 4;
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = 3;
        }
    }

    pub fn next_run(&mut self) {
        let i = match self.selected_run.selected() {
            Some(i) => {
                if i >= self.test_runs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_run.select(Some(i));
    }

    pub fn previous_run(&mut self) {
        let i = match self.selected_run.selected() {
            Some(i) => {
                if i == 0 {
                    self.test_runs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_run.select(Some(i));
    }

    pub async fn run_tests(&mut self) -> Result<()> {
        self.error_message = None;

        // Initialize database if needed
        let results_dir = self.testbench_path.join("results");
        tokio::fs::create_dir_all(&results_dir).await?;
        let db_path = results_dir.join("testbench.db");
        let temp_db = Database::new(&db_path).await?;

        // Start test run
        let run_id = Self::start_test_run(&temp_db).await?;

        // Find and run tests
        let test_dirs = Self::find_test_dirs(&self.testbench_path).await?;
        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut failed_tests = 0;
        let mut skipped_tests = 0;

        let epcheck_path = PathBuf::from(&self.epcheck_path);
        let epcheck_abs_path = if epcheck_path.is_absolute() {
            epcheck_path
        } else {
            std::env::current_dir()?.join(epcheck_path)
        };

        for test_dir in test_dirs {
            total_tests += 1;
            match Self::run_single_test(&temp_db, &epcheck_abs_path, &test_dir, run_id).await {
                Ok(TestResult::Passed) => passed_tests += 1,
                Ok(TestResult::Failed) => failed_tests += 1,
                Ok(TestResult::Skipped) => skipped_tests += 1,
                Err(_) => skipped_tests += 1,
            }
        }

        // Complete test run
        Self::complete_test_run(&temp_db, run_id, total_tests, passed_tests, failed_tests, skipped_tests).await?;

        // Refresh data
        self.test_runs = self.db.get_recent_runs(20).await?;
        self.performance_data = self.db.get_performance_data().await?;
        Ok(())
    }

    async fn start_test_run(db: &Database) -> Result<i64> {
        let path = db.path.clone();
        let run_id = tokio::task::spawn_blocking(move || -> Result<i64> {
            let conn = rusqlite::Connection::open(&path)?;
            conn.execute(
                "INSERT INTO test_runs (run_timestamp, status) VALUES (datetime('now'), 'running')",
                [],
            )?;
            Ok(conn.last_insert_rowid())
        }).await??;
        Ok(run_id)
    }

    async fn find_test_dirs(testbench_path: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        let mut entries = tokio::fs::read_dir(testbench_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() && path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("test-"))
                .unwrap_or(false) {
                dirs.push(path);
            }
        }

        dirs.sort();
        Ok(dirs)
    }

    async fn run_single_test(db: &Database, epcheck_path: &PathBuf, test_dir: &Path, run_id: i64) -> Result<TestResult> {
        use tokio::process::Command;

        let test_name = test_dir.file_name()
            .and_then(|n| n.to_str())
            .context("Invalid test directory name")?;

        // Check prerequisites
        if !Self::check_prerequisites(test_dir).await? {
            // Record as skipped
            let execution_id = Self::record_test_execution(db, run_id, test_name, test_dir, 0.0, 0).await?;
            Self::update_test_status(db, execution_id, "skipped").await?;
            return Ok(TestResult::Skipped);
        }

        // Load config
        let config_path = test_dir.join("config.json");
        let config: serde_json::Value = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            serde_json::from_str(&content)?
        } else {
            serde_json::json!({})
        };

        let args = if let Some(args_array) = config.get("args").and_then(|v| v.as_array()) {
            args_array.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            "-d src --no-colors".to_string()
        };
        let expected_exit_code = config.get("expected_exit_code")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as i32;

        // Run the test
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs_f64();

        let output = Command::new(epcheck_path)
            .args(args.split_whitespace())
            .current_dir(test_dir)
            .output()
            .await?;

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs_f64();
        let duration = end_time - start_time;

        let exit_code = output.status.code().unwrap_or(-1);

        // Record execution
        let execution_id = Self::record_test_execution(db, run_id, test_name, test_dir, duration, exit_code).await?;

        // Check exit code
        if exit_code != expected_exit_code {
            Self::update_test_status(db, execution_id, "failed").await?;
            return Ok(TestResult::Failed);
        }

        Self::update_test_status(db, execution_id, "passed").await?;
        Ok(TestResult::Passed)
    }

    async fn check_prerequisites(test_dir: &Path) -> Result<bool> {
        let config_path = test_dir.join("config.json");
        if !config_path.exists() {
            return Ok(true);
        }

        let content = tokio::fs::read_to_string(&config_path).await?;
        let config: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(skip_tools) = config.get("skip_tools").and_then(|v| v.as_array()) {
            for tool in skip_tools {
                if let Some(tool_name) = tool.as_str() {
                    if !Self::command_exists(tool_name).await {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    async fn command_exists(command: &str) -> bool {
        tokio::process::Command::new("which")
            .arg(command)
            .stdout(std::process::Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn record_test_execution(db: &Database, run_id: i64, test_name: &str, test_dir: &Path, duration: f64, exit_code: i32) -> Result<i64> {
        let path = db.path.clone();
        let test_dir_str = test_dir.to_string_lossy().to_string();
        let test_name = test_name.to_string();
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs_f64();

        let execution_id = tokio::task::spawn_blocking(move || -> Result<i64> {
            let conn = rusqlite::Connection::open(&path)?;
            conn.execute(
                "INSERT INTO test_executions (
                    test_run_id, test_name, test_directory, start_time, end_time,
                    duration_seconds, exit_code, status
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 'running')",
                rusqlite::params![
                    run_id,
                    test_name,
                    test_dir_str,
                    start_time,
                    start_time + duration,
                    duration,
                    exit_code,
                ],
            )?;
            Ok(conn.last_insert_rowid())
        }).await??;

        Ok(execution_id)
    }

    async fn update_test_status(db: &Database, execution_id: i64, status: &str) -> Result<()> {
        let path = db.path.clone();
        let status = status.to_string();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = rusqlite::Connection::open(&path)?;
            conn.execute(
                "UPDATE test_executions SET status = ? WHERE id = ?",
                rusqlite::params![status, execution_id],
            )?;
            Ok(())
        }).await??;
        Ok(())
    }

    async fn complete_test_run(db: &Database, run_id: i64, total: i32, passed: i32, failed: i32, skipped: i32) -> Result<()> {
        let path = db.path.clone();
        let status = if failed > 0 { "failed" } else { "completed" }.to_string();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = rusqlite::Connection::open(&path)?;
            conn.execute(
                "UPDATE test_runs SET status = ?, total_tests = ?, passed_tests = ?, failed_tests = ?, skipped_tests = ? WHERE id = ?",
                rusqlite::params![status, total, passed, failed, skipped, run_id],
            )?;
            Ok(())
        }).await??;

        Ok(())
    }

    pub fn draw_overview(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        // Latest run summary
        if let Some(latest) = self.test_runs.first() {
            let pass_rate = if latest.total_tests > 0 {
                (latest.passed_tests as f64 / latest.total_tests as f64 * 100.0) as u16
            } else {
                0
            };

            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Pass Rate"))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(pass_rate);
            f.render_widget(gauge, chunks[0]);

            let status_text = format!(
                "Latest Run: {} | Total: {} | Passed: {} | Failed: {} | Skipped: {}",
                latest.timestamp, latest.total_tests, latest.passed_tests,
                latest.failed_tests, latest.skipped_tests
            );
            let paragraph = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(paragraph, chunks[1]);
        }

        // Recent runs table
        let header = vec!["ID", "Timestamp", "Total", "Passed", "Failed", "Skipped", "Status"];
        let rows: Vec<Row> = self.test_runs.iter().take(10).map(|run| {
            Row::new(vec![
                run.id.to_string(),
                run.timestamp.clone(),
                run.total_tests.to_string(),
                run.passed_tests.to_string(),
                run.failed_tests.to_string(),
                run.skipped_tests.to_string(),
                run.status.clone(),
            ])
        }).collect();

        let header_row = Row::new(header.iter().map(|h| Span::styled(*h, Style::default().add_modifier(Modifier::BOLD))));

        let table = Table::new(rows, &[Constraint::Length(5), Constraint::Length(20), Constraint::Length(6), Constraint::Length(6), Constraint::Length(6), Constraint::Length(7), Constraint::Length(8)])
            .header(header_row)
            .block(Block::default().borders(Borders::ALL).title("Recent Test Runs"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(table, chunks[2], &mut self.overview_table_state);
    }

    pub fn draw_test_runs(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Test runs list
        let items: Vec<ListItem> = self.test_runs.iter().map(|run| {
            let content = format!(
                "{} | {} tests | {} passed | {} failed | {} skipped | {}",
                run.timestamp, run.total_tests, run.passed_tests,
                run.failed_tests, run.skipped_tests, run.status
            );
            ListItem::new(content)
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Test Runs"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[0], &mut self.selected_run);

        // Details of selected run
        if let Some(selected) = self.selected_run.selected() {
            if let Some(run) = self.test_runs.get(selected) {
                let details = format!(
                    "Run ID: {}\nTimestamp: {}\nStatus: {}\n\nTests: {}\nPassed: {}\nFailed: {}\nSkipped: {}\n\nPass Rate: {:.1}%",
                    run.id,
                    run.timestamp,
                    run.status,
                    run.total_tests,
                    run.passed_tests,
                    run.failed_tests,
                    run.skipped_tests,
                    if run.total_tests > 0 {
                        run.passed_tests as f64 / run.total_tests as f64 * 100.0
                    } else {
                        0.0
                    }
                );

                let paragraph = Paragraph::new(details)
                    .block(Block::default().borders(Borders::ALL).title("Run Details"));
                f.render_widget(paragraph, chunks[1]);
            }
        } else {
            let paragraph = Paragraph::new("Select a test run to view details")
                .block(Block::default().borders(Borders::ALL).title("Run Details"));
            f.render_widget(paragraph, chunks[1]);
        }
    }

    pub fn draw_performance(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Performance summary
        let perf_text = self.performance_data.iter()
            .take(10)
            .map(|p| format!("{}: {:.3}s{}", p.test_name, p.duration, p.memory_mb.map_or(String::new(), |m| format!(" | {}MB", m))))
            .collect::<Vec<_>>()
            .join("\n");

        let paragraph = Paragraph::new(perf_text)
            .block(Block::default().borders(Borders::ALL).title("Performance Summary"));
        f.render_widget(paragraph, chunks[0]);

        // Performance table
        let header = vec!["Test", "Duration (s)", "Memory (MB)"];
        let rows: Vec<Row> = self.performance_data.iter().take(15).map(|p| {
            Row::new(vec![
                p.test_name.clone(),
                format!("{:.3}", p.duration),
                p.memory_mb.map_or("N/A".to_string(), |m| m.to_string()),
            ])
        }).collect();

        let header_row = Row::new(header.iter().map(|h| Span::styled(*h, Style::default().add_modifier(Modifier::BOLD))));

        let table = Table::new(rows, &[Constraint::Percentage(40), Constraint::Percentage(30), Constraint::Percentage(30)])
            .header(header_row)
            .block(Block::default().borders(Borders::ALL).title("Performance Data"));

        f.render_widget(table, chunks[1]);
    }

    pub fn draw_graphs(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Duration over time chart
        let duration_data: Vec<(f64, f64)> = self.performance_data.iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.duration))
            .collect();

        let datasets = vec![
            Dataset::default()
                .name("Duration (s)")
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&duration_data),
        ];

        let chart = Chart::new(datasets)
            .block(Block::default().borders(Borders::ALL).title("Test Duration Over Time"))
            .x_axis(ratatui::widgets::Axis::default()
                .title("Test Run")
                .bounds([0.0, duration_data.len() as f64]))
            .y_axis(ratatui::widgets::Axis::default()
                .title("Duration (seconds)")
                .bounds([0.0, duration_data.iter().map(|(_, d)| *d).fold(0.0, f64::max)]));

        f.render_widget(chart, chunks[0]);

        // Memory usage chart (if available)
        let memory_data: Vec<(f64, f64)> = self.performance_data.iter()
            .enumerate()
            .filter_map(|(i, p)| p.memory_mb.map(|m| (i as f64, m as f64)))
            .collect();

        if !memory_data.is_empty() {
            let memory_datasets = vec![
                Dataset::default()
                    .name("Memory (MB)")
                    .marker(ratatui::symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Green))
                    .data(&memory_data),
            ];

            let memory_chart = Chart::new(memory_datasets)
                .block(Block::default().borders(Borders::ALL).title("Memory Usage Over Time"))
                .x_axis(ratatui::widgets::Axis::default()
                    .title("Test Run")
                    .bounds([0.0, memory_data.len() as f64]))
                .y_axis(ratatui::widgets::Axis::default()
                    .title("Memory (MB)")
                    .bounds([0.0, memory_data.iter().map(|(_, m)| *m).fold(0.0, f64::max)]));

            f.render_widget(memory_chart, chunks[1]);
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Memory Usage (No data available)");
            f.render_widget(block, chunks[1]);
        }
    }
}