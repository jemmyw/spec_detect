use crate::App;
use crate::ChangedFile;
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

fn changed_file_text(file: &ChangedFile, running: bool) -> Spans {
    let t = file.path.to_string_lossy();

    let status = match file.status {
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

    let running_text = if running { "> " } else { "  " };

    Spans::from(vec![
        Span::styled(running_text, Style::default().fg(Color::Yellow)),
        Span::raw(status),
        Span::raw(" "),
        Span::raw(t),
    ])
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // let chunks = Layout::default()
    //     .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
    //     .split(f.size());

    let files: Vec<ListItem> = app
        .changed_files
        .iter()
        .map(|c| ListItem::new(changed_file_text(c, true)))
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
