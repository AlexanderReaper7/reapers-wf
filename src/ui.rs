use ratatui::{prelude::*, widgets::*};

use crate::{app::App, models::Fissure};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(f.size());
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| text::Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    f.render_widget(tabs, chunks[0]);
    draw_footer(f, chunks[2]);
    match app.tabs.index {
        0 => draw_console_tab(f, app, chunks[1]),
        1 => draw_fissures_tab(f, app, chunks[1]),
        _ => {}
    };
}

fn draw_console_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Percentage(100), // log
            Constraint::Min(1),          // current cmd
        ])
        .split(area);
    let log = List::new(
        app.console_log
            .list
            .iter()
            .map(|t| ListItem::new(t.clone()))
            .collect::<Vec<ListItem>>(),
    )
    .highlight_style(Style::default().bold());
    f.render_stateful_widget(log, chunks[0], &mut app.console_log.state);
    let current_command = Paragraph::new(Line::from(vec![
        format!("> {}", app.current_cmd.clone()).into(),
        "█".into(),
    ]));
    f.render_widget(current_command, chunks[1]);
}

fn draw_fissures_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let fissure_table = Table::new(app.fissures.iter().map(|fissure| {
        Row::new(
            fissure
                .table_string()
                .iter()
                .map(|s| Cell::from(Span::styled(s.clone(), Style::default().fg(Color::Green)))),
        )
    }))
    .header(Row::new(Fissure::table_headers().iter().map(|s| {
        Cell::from(Span::styled(s.clone(), Style::default().fg(Color::Green)))
    })));
    f.render_widget(fissure_table, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    const BUILT_ON: &str = compile_time::datetime_str!();
    let text = format!(
        "Press ESC To Exit | Reaper's Warframe Tools v{} (compiled {})",
        env!("CARGO_PKG_VERSION"),
        BUILT_ON.replace('T', " ").replace('Z', "")
    );

    let block = Block::default().borders(Borders::TOP);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
