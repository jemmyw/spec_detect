use crate::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, ListItem, Paragraph, Row,
        Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // let chunks = Layout::default()
    //     .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
    //     .split(f.size());

    let files: Vec<ListItem> = app
        .changed_files
        .iter()
        .map(|c| {
            let t = c.path.to_string_lossy();
            ListItem::new(vec![Spans::from(Span::raw(t))])
        })
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
