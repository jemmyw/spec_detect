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

pub fn draw<B: Backend>(f: &mut Frame<B>) {
    // let chunks = Layout::default()
    //     .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
    //     .split(f.size());

    let text = vec![Spans::from("this is some text")];

    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "block title",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let p = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, f.size())
}
