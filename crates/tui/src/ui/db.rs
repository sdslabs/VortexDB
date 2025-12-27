use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Color,
    widgets::ListItem,
};

use super::components::{OperationsList, PageTitle, common_instructions, create_instructions};
use crate::app::App;

const DB_OPERATIONS: &[&str] = &["Create New Database", "Select Database", "Delete Database"];

fn get_db_items() -> Vec<ListItem<'static>> {
    DB_OPERATIONS.iter().map(|&op| ListItem::new(op)).collect()
}

pub fn render_database(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title = PageTitle {
        text: "Database Management",
        color: Color::Green,
    };

    let operations_list = OperationsList {
        items: get_db_items(),
        title: "Database Operations",
        color: Color::Green,
        selected: app.db_selected,
    };

    let instructions = if app.show_modal() {
        app.modal_footer_items()
    } else {
        let mut base = common_instructions();
        base.insert(4, ("→ Next".to_string(), Color::Gray));
        base
    };

    f.render_widget(title.render(), chunks[0]);
    operations_list.render(f, chunks[1]);
    f.render_widget(create_instructions(instructions), chunks[2]);
}

pub fn get_db_operations_count() -> usize {
    DB_OPERATIONS.len()
}
