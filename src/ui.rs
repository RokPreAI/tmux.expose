use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::model::{App, Session};

pub const MIN_CARD_WIDTH: u16 = 32;
pub const MIN_CARD_HEIGHT: u16 = 10;
const CARD_GAP: u16 = 2;
const FOOTER_HEIGHT: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridLayout {
    pub columns: usize,
    pub rows: usize,
    pub cards: Vec<Rect>,
}

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    if area.width < 20 || area.height < 6 {
        render_centered_message(frame, area, "Terminal too small");
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(FOOTER_HEIGHT)])
        .split(area);

    if let Some(error) = &app.error {
        render_centered_message(frame, chunks[0], error);
    } else if app.sessions.is_empty() {
        render_centered_message(
            frame,
            chunks[0],
            "No tmux sessions found.\nPress q or Esc to quit.",
        );
    } else {
        render_grid(frame, app, chunks[0]);
    }

    let footer = Paragraph::new("↑/↓/←/→ or hjkl to move · Enter to switch · q/Esc/Ctrl-C to quit")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[1]);
}

pub fn render_grid(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let grid = calculate_grid(area, app.sessions.len());

    for (index, card_area) in grid.cards.iter().enumerate() {
        if let Some(session) = app.sessions.get(index) {
            render_card(frame, session, index == app.selected_index, *card_area);
        }
    }
}

pub fn render_card(frame: &mut Frame<'_>, session: &Session, selected: bool, area: Rect) {
    let status = if session.attached {
        "attached"
    } else {
        "detached"
    };
    let title = format!(
        " {} ",
        truncate(&session.name, area.width.saturating_sub(12) as usize)
    );
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().add_modifier(if selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        ))
        .title_bottom(Span::styled(
            format!(" {status} "),
            Style::default().fg(Color::DarkGray),
        ))
        .borders(Borders::ALL)
        .border_type(if selected {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(if selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let preview_height = area.height.saturating_sub(5) as usize;
    let mut lines = Vec::new();
    let window = session.current_window.as_deref().unwrap_or("unknown");
    lines.push(Line::from(vec![Span::styled(
        format!("{} · {} windows", window, session.window_count),
        Style::default().fg(Color::Cyan),
    )]));
    lines.push(Line::from(""));

    if session.preview_error.is_some() {
        lines.push(Line::from(Span::styled(
            "Preview unavailable",
            Style::default().fg(Color::Red),
        )));
    } else if session.preview.is_empty() {
        lines.push(Line::from(Span::styled(
            "No visible content",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let start = session.preview.len().saturating_sub(preview_height);
        for line in session.preview.iter().skip(start) {
            lines.push(Line::from(truncate(
                line,
                area.width.saturating_sub(4) as usize,
            )));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(if selected {
            Style::default().fg(Color::White)
        } else {
            Style::default()
        });

    frame.render_widget(paragraph, area);
}

pub fn calculate_grid(area: Rect, item_count: usize) -> GridLayout {
    if item_count == 0 || area.width == 0 || area.height == 0 {
        return GridLayout {
            columns: 1,
            rows: 0,
            cards: Vec::new(),
        };
    }

    let max_columns = ((area.width + CARD_GAP) / (MIN_CARD_WIDTH + CARD_GAP)).max(1) as usize;
    let columns = max_columns.min(item_count).max(1);
    let rows = item_count.div_ceil(columns);
    let total_gap_width = CARD_GAP.saturating_mul(columns.saturating_sub(1) as u16);
    let card_width = area
        .width
        .saturating_sub(total_gap_width)
        .checked_div(columns as u16)
        .unwrap_or(area.width)
        .max(1);
    let total_gap_height = CARD_GAP.saturating_mul(rows.saturating_sub(1) as u16);
    let card_height = area
        .height
        .saturating_sub(total_gap_height)
        .checked_div(rows as u16)
        .unwrap_or(area.height)
        .max(1);

    let mut cards = Vec::with_capacity(item_count);
    for index in 0..item_count {
        let col = index % columns;
        let row = index / columns;
        cards.push(Rect::new(
            area.x + col as u16 * (card_width + CARD_GAP),
            area.y + row as u16 * (card_height + CARD_GAP),
            card_width,
            card_height,
        ));
    }

    GridLayout {
        columns,
        rows,
        cards,
    }
}

fn render_centered_message(frame: &mut Frame<'_>, area: Rect, message: &str) {
    let paragraph = Paragraph::new(message)
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn truncate(value: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let mut output = String::new();
    for ch in value.chars().take(max_width) {
        output.push(ch);
    }
    output
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::*;

    #[test]
    fn grid_uses_as_many_min_width_columns_as_fit() {
        let grid = calculate_grid(Rect::new(0, 0, 100, 30), 6);

        assert_eq!(grid.columns, 3);
        assert_eq!(grid.rows, 2);
        assert_eq!(grid.cards.len(), 6);
        assert!(grid.cards[0].width >= MIN_CARD_WIDTH);
    }

    #[test]
    fn grid_always_has_one_column_for_narrow_terminals() {
        let grid = calculate_grid(Rect::new(0, 0, 20, 30), 2);

        assert_eq!(grid.columns, 1);
        assert_eq!(grid.rows, 2);
        assert_eq!(grid.cards.len(), 2);
    }
}
