use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

#[allow(unused_variables)]
pub fn render_dashboard(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(size);

    let title_lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "██╗   ██╗███████╗ ██████╗████████╗ ██████╗ ██████╗ ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "██║   ██║██╔════╝██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "██║   ██║█████╗  ██║        ██║   ██║   ██║██████╔╝",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "╚██╗ ██╔╝██╔══╝  ██║        ██║   ██║   ██║██╔══██╗",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            " ╚████╔╝ ███████╗╚██████╗   ██║   ╚██████╔╝██║  ██║",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "  ╚═══╝  ╚══════╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "██████╗ ██████╗ ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "██╔══██╗██╔══██╗",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "██║  ██║██████╔╝",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "██║  ██║██╔══██╗",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "██████╔╝██████╔╝",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "╚═════╝ ╚═════╝ ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(""),
    ];

    let title = Paragraph::new(title_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Welcome")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);

    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("→ Enter/Right", Style::default().fg(Color::Green)),
            Span::raw(" to navigate to Database Management"),
        ]),
        Line::from(vec![Span::styled(
            "Press 'q' or 'Esc' to quit",
            Style::default()
                .fg(Color::Red)
                .add_modifier(ratatui::style::Modifier::ITALIC),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Navigation")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(title, chunks[0]);
    f.render_widget(instructions, chunks[1]);
}
