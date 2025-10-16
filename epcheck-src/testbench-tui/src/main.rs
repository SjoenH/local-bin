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

#[derive(Parser)]
#[command(name = "testbench-tui")]
#[command(about = "TUI for running and visualizing epcheck testbench results")]
struct Args {
    /// Path to the testbench directory
    #[arg(short, long, default_value = "../../testbench/v1.0")]
    testbench_path: String,

    /// Path to epcheck binary
    #[arg(short, long, default_value = "../../epcheck")]
    epcheck_path: String,

    /// Run tests and exit without starting TUI
    #[arg(long)]
    run_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.run_only {
        // Run tests without TUI
        let testbench_path = PathBuf::from(&args.testbench_path);
        let epcheck_path = PathBuf::from(&args.epcheck_path);

        // Initialize database
        let results_dir = testbench_path.join("results");
        tokio::fs::create_dir_all(&results_dir).await?;
        let db_path = results_dir.join("testbench.db");
        let _db = Database::new(&db_path).await?;

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
                std::process::exit(0);
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
