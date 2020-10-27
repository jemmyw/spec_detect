use crate::app_state::{AppState, AppStateManager, Event, WatchedFile};
use crate::input;
use crate::program::Program;

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::io;
use termion::event::Key;
use termion::raw::IntoRawMode;
use tokio::stream::{Stream, StreamExt};
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, ListItem, ListState,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

fn file_text(file: &WatchedFile) -> Spans {
    let t = file.changed_file.path.to_string_lossy();

    let status = match file.changed_file.status {
        git2::Delta::Unmodified => "U",
        git2::Delta::Added => "A",
        git2::Delta::Deleted => "D",
        git2::Delta::Modified => "M",
        git2::Delta::Renamed => "R",
        git2::Delta::Copied => "R",
        git2::Delta::Ignored => "I",
        git2::Delta::Untracked => "-",
        git2::Delta::Typechange => "M",
        git2::Delta::Unreadable => "X",
        git2::Delta::Conflicted => "C",
    };

    let running_text = match file.test_status {
        crate::app_state::TestStatus::Unknown => "> ",
        crate::app_state::TestStatus::Running => "> ",
        crate::app_state::TestStatus::Passed => "  ",
        crate::app_state::TestStatus::Failed => "x  ",
    };

    Spans::from(vec![
        Span::styled(running_text, Style::default().fg(Color::Yellow)),
        Span::raw(status),
        Span::raw(" "),
        Span::raw(t),
    ])
}

pub fn draw<B: Backend>(f: &mut Frame<B>, state: &AppState) {
    // let chunks = Layout::default()
    //     .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
    //     .split(f.size());

    let files: Vec<ListItem> = state
        .watched_files
        .iter()
        .map(|c| ListItem::new(file_text(c)))
        .collect();
    let list = List::new(files).block(
        Block::default().borders(Borders::ALL).title(Span::styled(
            "Changed files",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
    );

    f.render_widget(list, f.size())
}

pub struct TuiApp {}

#[async_trait]
impl Program for TuiApp {
    async fn run<'stream>(&self, app: AppStateManager) -> Result<()> {
        let state_stream = app.stream();
        tokio::pin!(state_stream);

        let mut input_rx = input::listen();

        let stdout = io::stdout()
            .into_raw_mode()
            .context("Could not open stdout")?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("Could not create a terminal")?;
        terminal.clear().context("Could not clear the terminal")?;

        loop {
            tokio::select! {
                app_state = state_stream.next() => {
                    match app_state {
                        Some((_, app_state)) => {
                            if app_state.should_quit {
                                break;
                            }
                            terminal
                                .draw(|f| draw(f, &app_state))
                                .context("Error while updating UI")?;
                        },
                        None => {break;}
                    }
                }
                key = input_rx.recv() => {
                    let key = key.unwrap();

                    if let Key::Char('q') = key {
                        app.dispatch(Event::Quit).await?;
                    }
                }
            }
        }

        terminal.clear().ok();

        Ok(())
    }
}
