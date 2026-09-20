use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};

pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) runtime: String,
    pub(crate) model: Option<String>,
    pub(crate) command: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Decision {
    Continue,
    Quit,
    Launch(usize),
}

pub(crate) struct Picker {
    entries: Vec<Entry>,
    query: String,
    matches: Vec<usize>,
    selection: ListState,
    details_scroll: u16,
    details_page_size: u16,
}

impl Picker {
    pub(crate) fn new(entries: Vec<Entry>) -> Self {
        let matches: Vec<_> = (0..entries.len()).collect();
        let selected = if matches.is_empty() { None } else { Some(0) };
        Self {
            entries,
            query: String::new(),
            matches,
            selection: ListState::default().with_selected(selected),
            details_scroll: 0,
            details_page_size: 1,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Decision {
        if should_quit(key) {
            return Decision::Quit;
        }
        if key.kind != KeyEventKind::Press {
            return Decision::Continue;
        }
        match key.code {
            KeyCode::Char(character)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.query.push(character);
                self.filter();
            }
            KeyCode::Backspace if key.modifiers.is_empty() => {
                self.query.pop();
                self.filter();
            }
            KeyCode::Down if !self.matches.is_empty() => {
                self.details_scroll = 0;
                let next = self
                    .selection
                    .selected()
                    .map_or(0, |selected| (selected + 1).min(self.matches.len() - 1));
                self.selection.select(Some(next));
            }
            KeyCode::Up => {
                self.details_scroll = 0;
                self.selection.select(
                    self.selection
                        .selected()
                        .map(|selected| selected.saturating_sub(1)),
                );
            }
            KeyCode::Enter if key.modifiers.is_empty() => {
                if let Some(index) = self
                    .selection
                    .selected()
                    .and_then(|selected| self.matches.get(selected))
                {
                    return Decision::Launch(*index);
                }
            }
            KeyCode::PageDown => {
                self.details_scroll = self.details_scroll.saturating_add(self.details_page_size);
            }
            KeyCode::PageUp => {
                self.details_scroll = self.details_scroll.saturating_sub(self.details_page_size);
            }
            _ => {}
        }
        Decision::Continue
    }

    fn filter(&mut self) {
        self.details_scroll = 0;
        let query = self.query.to_lowercase();
        self.matches = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let candidate =
                    format!("{} {} {}", entry.name, entry.runtime, entry.id).to_lowercase();
                let mut characters = candidate.chars();
                query
                    .chars()
                    .all(|wanted| characters.by_ref().any(|character| character == wanted))
                    .then_some(index)
            })
            .collect();
        self.selection = ListState::default().with_selected(if self.matches.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    pub(crate) fn render(&mut self, frame: &mut Frame) {
        let details = self
            .selection
            .selected()
            .and_then(|selected| self.matches.get(selected))
            .and_then(|index| self.entries.get(*index))
            .map_or_else(String::new, |entry| {
                let model = entry
                    .model
                    .as_deref()
                    .unwrap_or("Configured models (managed in oMLX)");
                format!(
                    "ID: {}\nModel: {model}\nCommand: {}",
                    entry.id, entry.command
                )
            });
        let paragraph = Paragraph::new(details).wrap(Wrap { trim: false });
        // Terminal geometry and Paragraph scrolling use u16; saturate oversized content.
        let lines = u16::try_from(paragraph.line_count(frame.area().width.saturating_sub(2)))
            .unwrap_or(u16::MAX);
        let details_height = lines
            .saturating_add(2)
            .min(frame.area().height.saturating_sub(6));
        let areas = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(details_height),
        ])
        .split(frame.area());
        frame.render_widget(
            Paragraph::new(format!("Filter: {}", self.query))
                .block(Block::bordered().title(" lml · Enter launches · Esc / Ctrl+C exits ")),
            areas[0],
        );
        let items: Vec<_> = self
            .matches
            .iter()
            .filter_map(|index| self.entries.get(*index))
            .map(|entry| ListItem::new(format!("{}  [{}]", entry.name, entry.runtime)))
            .collect();
        if items.is_empty() {
            let message = if self.entries.is_empty() {
                "No launch profiles. Run `lml setup` or `lml import`, or edit your catalog."
            } else {
                "No matching profiles. Backspace changes the filter; Esc exits."
            };
            frame.render_widget(
                Paragraph::new(message)
                    .wrap(Wrap { trim: false })
                    .block(Block::bordered().title(" Profiles ")),
                areas[1],
            );
        } else {
            frame.render_stateful_widget(
                List::new(items)
                    .block(Block::bordered().title(" Profiles "))
                    .highlight_symbol("> ")
                    .highlight_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                areas[1],
                &mut self.selection,
            );
        }
        self.details_page_size = areas[2].height.saturating_sub(2).max(1);
        self.details_scroll = self
            .details_scroll
            .min(lines.saturating_sub(self.details_page_size));
        frame.render_widget(
            paragraph
                .scroll((self.details_scroll, 0))
                .block(Block::bordered().title(" Details · PgUp/PgDn scroll ")),
            areas[2],
        );
    }
}

pub(crate) fn run(
    terminal: &mut DefaultTerminal,
    picker: &mut Picker,
) -> io::Result<Option<usize>> {
    loop {
        terminal.draw(|frame| picker.render(frame))?;
        if let Event::Key(key) = event::read()? {
            match picker.handle_key(key) {
                Decision::Continue => {}
                Decision::Quit => return Ok(None),
                Decision::Launch(index) => return Ok(Some(index)),
            }
        }
    }
}

fn should_quit(key: KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && match key.code {
            KeyCode::Esc => key.modifiers.is_empty(),
            KeyCode::Char('c') => key.modifiers == KeyModifiers::CONTROL,
            _ => false,
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn quit_keys_exit_only_on_press() {
        for (code, modifiers) in [
            (KeyCode::Esc, KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            assert!(should_quit(KeyEvent::new(code, modifiers)));
            for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
                assert!(!should_quit(KeyEvent::new_with_kind(code, modifiers, kind)));
            }
        }
    }

    #[test]
    fn unrelated_keys_do_not_quit() {
        for (code, modifiers) in [
            (KeyCode::Char('q'), KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::NONE),
            (KeyCode::Char('q'), KeyModifiers::ALT),
            (KeyCode::Enter, KeyModifiers::NONE),
        ] {
            assert!(!should_quit(KeyEvent::new(code, modifiers)));
        }
    }

    #[test]
    fn empty_catalog_shows_setup_guidance_and_exit_keys() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        let mut picker = Picker::new(Vec::new());
        terminal
            .draw(|frame| picker.render(frame))
            .expect("render empty catalog");
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("lml setup"));
        assert!(text.contains("Esc / Ctrl+C"));
    }

    #[test]
    fn tiny_terminal_can_render() {
        let mut terminal = Terminal::new(TestBackend::new(1, 1)).expect("test terminal");
        let mut picker = Picker::new(Vec::new());
        terminal
            .draw(|frame| picker.render(frame))
            .expect("render tiny screen");
    }

    #[test]
    fn typing_filters_immediately_and_enter_launches_the_selected_match() {
        let entries = ["Alpha", "Qwen small", "Qwen large"]
            .into_iter()
            .map(|name| Entry {
                id: name.into(),
                name: name.into(),
                runtime: "llama.cpp".into(),
                model: Some(format!("/models/{name}.gguf")),
                command: "llama-server -m model.gguf".into(),
            })
            .collect();
        let mut picker = Picker::new(entries);
        assert_eq!(
            picker.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            Decision::Continue
        );
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        terminal
            .draw(|frame| picker.render(frame))
            .expect("render filtered picker");
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("Qwen small"));
        assert!(!text.contains("Alpha"));
        picker.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(
            picker.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Decision::Launch(2)
        );
        assert_eq!(
            picker.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Decision::Quit
        );
    }

    #[test]
    fn no_matches_cannot_launch_and_backspace_restores_the_selection() {
        let mut picker = Picker::new(vec![Entry {
            id: "alpha".into(),
            name: "Alpha".into(),
            runtime: "llama.cpp".into(),
            model: None,
            command: "server".into(),
        }]);
        picker.handle_key(KeyEvent::new(KeyCode::Char('é'), KeyModifiers::NONE));
        picker.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(
            picker.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Decision::Continue
        );
        picker.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(
            picker.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Decision::Launch(0)
        );
        picker.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        assert_eq!(
            picker.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Decision::Quit
        );
    }

    #[test]
    fn long_commands_expand_and_can_be_scrolled_to_the_last_argument() {
        let entry = Entry {
            id: "long-profile".into(),
            name: "Long profile".into(),
            runtime: "llama.cpp".into(),
            model: Some(format!("/models/{}/model.gguf", "nested/".repeat(30))),
            command: format!(
                "llama-server {} FINAL_MTP_FLAG",
                "--option argument ".repeat(20)
            ),
        };
        let mut picker = Picker::new(vec![entry]);
        let mut terminal = Terminal::new(TestBackend::new(80, 40)).expect("test terminal");
        terminal
            .draw(|frame| picker.render(frame))
            .expect("render tall picker");
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(
            text.contains("FINAL_MTP_FLAG"),
            "the details should use available vertical space"
        );
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).expect("short terminal");
        terminal
            .draw(|frame| picker.render(frame))
            .expect("render short picker");
        for _ in 0..30 {
            picker.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
            terminal
                .draw(|frame| picker.render(frame))
                .expect("scroll details");
        }
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(
            text.contains("FINAL_MTP_FLAG"),
            "the final arguments must remain accessible in a short terminal"
        );
    }
}
