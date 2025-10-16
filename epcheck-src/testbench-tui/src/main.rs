use std::collections::HashMap;
use std::io::{self, stdout, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

mod app;
mod db;

use app::{App, CurrentScreen};
use db::Database;

async fn perform_performance_check(db: &Database, results_dir: &PathBuf) -> Result<()> {
    let baseline_file = results_dir.join("performance-baseline.json");

    // Get current performance data
    let current_perf = db.get_performance_data().await?;

    // Calculate average performance per test
    let mut current_averages: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut current_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for point in &current_perf {
        *current_counts.entry(point.test_name.clone()).or_insert(0) += 1;
        *current_averages.entry(point.test_name.clone()).or_insert(0.0) += point.duration;
    }

    for (test_name, count) in &current_counts {
        if let Some(avg) = current_averages.get_mut(test_name) {
            *avg /= *count as f64;
        }
    }

    // Load baseline if it exists
    let baseline_averages: std::collections::HashMap<String, f64> = if baseline_file.exists() {
        let content = tokio::fs::read_to_string(&baseline_file).await?;
        serde_json::from_str(&content)?
    } else {
        println!("📊 No performance baseline found, creating new baseline...");
        // Save current performance as baseline
        let json = serde_json::to_string_pretty(&current_averages)?;
        tokio::fs::write(&baseline_file, json).await?;
        println!("✅ Performance baseline saved");
        return Ok(());
    };

    // Compare performance
    const PERFORMANCE_THRESHOLD: f64 = 0.10; // 10% degradation allowed
    let mut total_degradation = 0.0;
    let mut degradation_count = 0;
    let mut warnings = Vec::new();

    for (test_name, current_avg) in &current_averages {
        if let Some(baseline_avg) = baseline_averages.get(test_name) {
            if *baseline_avg > 0.0 {
                let degradation = (current_avg - baseline_avg) / baseline_avg;
                if degradation > PERFORMANCE_THRESHOLD {
                    let percent = (degradation * 100.0).round();
                    warnings.push(format!(
                        "⚠️  {}: {:.3}s → {:.3}s ({:.1}% slower)",
                        test_name, baseline_avg, current_avg, percent
                    ));
                    total_degradation += degradation;
                    degradation_count += 1;
                }
            } else if *current_avg > 0.0 {
                // Baseline was 0, now has performance data - not a degradation
                println!("📊 Test '{}' now has performance data: {:.3}s", test_name, current_avg);
            }
        } else {
            println!("📊 New test '{}' added to performance tracking", test_name);
        }
    }

    let avg_degradation = if degradation_count > 0 { total_degradation / degradation_count as f64 } else { 0.0 };

    if !warnings.is_empty() {
        println!("🚨 Performance degradation detected:");
        for warning in warnings {
            println!("{}", warning);
        }
        println!("📈 Average degradation: {:.1}%", avg_degradation * 100.0);

        if avg_degradation > PERFORMANCE_THRESHOLD {
            println!("❌ Performance degradation exceeds threshold ({}%)", PERFORMANCE_THRESHOLD * 100.0);
            println!("💡 Consider optimizing the code or updating the baseline if this is expected");
            return Err(anyhow::anyhow!("Performance regression detected"));
        } else {
            println!("✅ Performance degradation within acceptable limits");
        }
    } else {
        println!("✅ No significant performance changes detected");
    }

    // Update baseline with current performance
    let json = serde_json::to_string_pretty(&current_averages)?;
    tokio::fs::write(&baseline_file, json).await?;
    println!("📊 Performance baseline updated");

    Ok(())
}

#[derive(Parser)]
#[command(name = "testbench-tui")]
#[command(about = "TUI for running and visualizing epcheck testbench results")]
struct Args {
    /// Path to the testbench directory
    #[arg(short, long, default_value = "testbench/v1.0")]
    testbench_path: String,

    /// Path to epcheck binary
    #[arg(short, long, default_value = "epcheck-src/target/release/epcheck")]
    epcheck_path: String,

    /// Run tests and exit without starting TUI
    #[arg(long)]
    run_only: bool,

    /// Run performance check and compare against baseline
    #[arg(long)]
    performance_check: bool,

    /// Reset performance baseline (removes existing baseline)
    #[arg(long)]
    reset_baseline: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.run_only || args.performance_check {
        // Run tests without TUI
        let testbench_path = PathBuf::from(&args.testbench_path);
        let epcheck_path = PathBuf::from(&args.epcheck_path);

        // Initialize database
        let results_dir = testbench_path.join("results");
        tokio::fs::create_dir_all(&results_dir).await?;
        let db_path = results_dir.join("testbench.db");
        let db = Database::new(&db_path).await?;

        // Create a temporary app just to use its test running logic
        let mut app = App::new(args.testbench_path.clone(), args.epcheck_path.clone()).await?;

        match app.run_tests().await {
            Ok(_) => {
                println!("✅ Tests completed successfully!");
                // Print summary
                if let Some(latest) = app.test_runs.first() {
                    println!("Latest run: {} tests, {} passed, {} failed, {} skipped",
                        latest.total_tests, latest.passed_tests, latest.failed_tests, latest.skipped_tests);
                }

                        if args.reset_baseline {
                    // Reset performance baseline
                    let baseline_file = results_dir.join("performance-baseline.json");
                    if baseline_file.exists() {
                        tokio::fs::remove_file(&baseline_file).await?;
                        println!("✅ Performance baseline reset");
                    } else {
                        println!("ℹ️  No performance baseline found to reset");
                    }
                    std::process::exit(0);
                } else if args.performance_check {
                    // Perform performance check
                    match perform_performance_check(&db, &results_dir).await {
                        Ok(_) => {
                            println!("✅ Performance check passed!");
                            std::process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("❌ Performance check failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    std::process::exit(0);
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to run tests: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Start TUI
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // create app and run it
        let mut app = App::new(args.testbench_path, args.epcheck_path).await?;
        let res = run_app(&mut terminal, &mut app).await;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Right => app.next_tab(),
                        KeyCode::Left => app.previous_tab(),
                        KeyCode::Up => {
                            if app.current_tab == 1 { // Test Runs tab
                                app.previous_run();
                            }
                        }
                        KeyCode::Down => {
                            if app.current_tab == 1 { // Test Runs tab
                                app.next_run();
                            }
                        }
                        KeyCode::Char('r') => {
                            if let Err(e) = app.run_tests().await {
                                app.error_message = Some(format!("Failed to run tests: {}", e));
                            }
                        }
                        KeyCode::Char('h') => app.current_screen = CurrentScreen::Help,
                        _ => {}
                    },
                    CurrentScreen::Help => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    match app.current_screen {
        CurrentScreen::Main => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(size);

            let titles: Vec<Line> = vec!["Overview", "Test Runs", "Performance", "Graphs"]
                .iter()
                .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
                .collect();

            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Testbench TUI"))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(app.current_tab);

            f.render_widget(tabs, chunks[0]);

            match app.current_tab {
                0 => app.draw_overview(f, chunks[1]),
                1 => app.draw_test_runs(f, chunks[1]),
                2 => app.draw_performance(f, chunks[1]),
                3 => app.draw_graphs(f, chunks[1]),
                _ => {}
            }
        }
        CurrentScreen::Help => {
            let block = Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            let help_text = vec![
                Line::from("Navigation:"),
                Line::from("  ← → : Switch tabs"),
                Line::from("  r   : Run tests"),
                Line::from("  h   : Show this help"),
                Line::from("  q   : Quit"),
                Line::from(""),
                Line::from("Tabs:"),
                Line::from("  Overview    : Summary statistics"),
                Line::from("  Test Runs   : Detailed test execution results"),
                Line::from("  Performance : Performance metrics"),
                Line::from("  Graphs      : Visual performance trends"),
            ];
            let paragraph = Paragraph::new(help_text)
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(Clear, size);
            f.render_widget(paragraph, size);
        }
    }

    // Show error message if any
    if let Some(ref error) = app.error_message {
        let block = Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red));
        let paragraph = Paragraph::new(error.as_str()).block(block);
        let area = centered_rect(60, 20, size);
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
