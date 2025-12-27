use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub struct PageTitle {
    pub text: &'static str,
    pub color: Color,
}

impl PageTitle {
    pub fn render(&self) -> Paragraph<'static> {
        Paragraph::new(self.text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Vector DB")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(self.color)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.color))
    }
}

pub struct OperationsList {
    pub items: Vec<ListItem<'static>>,
    pub title: &'static str,
    pub color: Color,
    pub selected: usize,
}

impl OperationsList {
    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let list = List::new(self.items.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title)
                    .border_style(Style::default().fg(self.color)),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(self.color).bg(Color::Gray));

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));

        f.render_stateful_widget(list, area, &mut list_state);
    }
}

pub fn create_instructions(spans: Vec<(String, Color)>) -> Paragraph<'static> {
    let mut line_spans = Vec::new();

    for (i, (text, color)) in spans.iter().enumerate() {
        if i > 0 {
            line_spans.push(Span::raw(" | "));
        }
        line_spans.push(Span::styled(text.clone(), Style::default().fg(*color)));
    }

    Paragraph::new(vec![Line::from(line_spans)])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
}

pub fn common_instructions() -> Vec<(String, Color)> {
    vec![
        ("↑ Up".to_string(), Color::Gray),
        ("↓ Down".to_string(), Color::Gray),
        ("Enter Select".to_string(), Color::Green),
        ("← Previous".to_string(), Color::Gray),
        ("q/Esc Quit".to_string(), Color::Red),
    ]
}
