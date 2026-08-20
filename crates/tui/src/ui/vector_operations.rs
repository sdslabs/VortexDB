use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Paragraph},
};

use super::components::{OperationsList, PageTitle, common_instructions, create_instructions};
use crate::app::App;

const VECTOR_OPERATIONS: &[&str] = &[
    "List All Vectors",
    "Delete Vector",
    "Search Similar Vectors",
    "Insert Text Embedding",
    "Insert Image Embedding",
];

fn get_vector_items() -> Vec<ListItem<'static>> {
    VECTOR_OPERATIONS
        .iter()
        .map(|&op| ListItem::new(op))
        .collect()
}

pub fn render_vector_operations(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3), // Add space for database info
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title = PageTitle {
        text: "Vector Operations",
        color: Color::Magenta,
    };

    // Database info section
    let db_info = if let Some(db_name) = app.database.get_selected_database_name() {
        Paragraph::new(vec![Line::from(vec![
            Span::styled("Selected Database: ", Style::default().fg(Color::Gray)),
            Span::styled(db_name, Style::default().fg(Color::Green)),
        ])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Database Info")
                .border_style(Style::default().fg(Color::Gray)),
        )
    } else {
        Paragraph::new("No database selected. Please select a database first.")
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Database Info")
                    .border_style(Style::default().fg(Color::Red)),
            )
    };

    let operations_list = OperationsList {
        items: get_vector_items(),
        title: "Available Operations",
        color: if app.database.is_database_selected() {
            Color::Magenta
        } else {
            Color::DarkGray
        },
        selected: app.vector_selected,
    };

    f.render_widget(title.render(), chunks[0]);
    f.render_widget(db_info, chunks[1]);
    operations_list.render(f, chunks[2]);

    let instructions = if app.show_modal() {
        app.modal_footer_items()
    } else {
        common_instructions()
    };

    f.render_widget(create_instructions(instructions), chunks[3]);
}

pub fn get_vector_operations_count() -> usize {
    VECTOR_OPERATIONS.len()
}
