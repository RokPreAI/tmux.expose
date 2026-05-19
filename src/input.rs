use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::model::App;

pub fn handle_key(app: &mut App, key: KeyEvent, columns: usize) {
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.should_quit = true,
        (KeyCode::Esc, _) | (KeyCode::Char('q'), _) => app.should_quit = true,
        (KeyCode::Enter, _) => app.should_switch = true,
        (KeyCode::Left, _) | (KeyCode::Char('h'), _) => move_left(app, columns),
        (KeyCode::Right, _) | (KeyCode::Char('l'), _) => move_right(app, columns),
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.move_up(columns),
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.move_down(columns),
        _ => {}
    }
}

fn move_left(app: &mut App, columns: usize) {
    let columns = columns.max(1);
    if !app.selected_index.is_multiple_of(columns) {
        app.move_left();
    }
}

fn move_right(app: &mut App, columns: usize) {
    let columns = columns.max(1);
    if app.selected_index % columns != columns - 1 {
        app.move_right();
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::model::{App, Session};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn session(name: &str) -> Session {
        Session {
            id: format!("${name}"),
            name: name.to_string(),
            attached: false,
            window_count: 1,
            current_window: None,
            last_activity: None,
            preview: Vec::new(),
            preview_error: None,
        }
    }

    #[test]
    fn arrow_keys_move_selection() {
        let mut app = App::new(vec![session("one"), session("two"), session("three")], None);

        handle_key(&mut app, key(KeyCode::Right), 2);
        assert_eq!(app.selected_index, 1);

        handle_key(&mut app, key(KeyCode::Down), 2);
        assert_eq!(app.selected_index, 2);

        handle_key(&mut app, key(KeyCode::Left), 2);
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn hjkl_keys_move_selection() {
        let mut app = App::new(vec![session("one"), session("two"), session("three")], None);

        handle_key(&mut app, key(KeyCode::Char('l')), 2);
        handle_key(&mut app, key(KeyCode::Char('j')), 2);
        assert_eq!(app.selected_index, 2);

        handle_key(&mut app, key(KeyCode::Char('h')), 2);
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn quit_keys_mark_app_for_exit() {
        let mut app = App::new(vec![session("one")], None);

        handle_key(&mut app, key(KeyCode::Char('q')), 1);

        assert!(app.should_quit);
    }

    #[test]
    fn enter_marks_app_for_switch() {
        let mut app = App::new(vec![session("one")], None);

        handle_key(&mut app, key(KeyCode::Enter), 1);

        assert!(app.should_switch);
    }

    #[test]
    fn ctrl_c_marks_app_for_exit() {
        let mut app = App::new(vec![session("one")], None);

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            1,
        );

        assert!(app.should_quit);
    }

    #[test]
    fn horizontal_navigation_clamps_at_row_edges() {
        let mut app = App::new(
            vec![
                session("one"),
                session("two"),
                session("three"),
                session("four"),
            ],
            None,
        );
        app.selected_index = 2;

        handle_key(&mut app, key(KeyCode::Right), 3);
        assert_eq!(app.selected_index, 2);

        app.selected_index = 3;
        handle_key(&mut app, key(KeyCode::Left), 3);
        assert_eq!(app.selected_index, 3);
    }
}
