use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};

pub(crate) fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(render)?;
        if let Event::Key(key) = event::read()?
            && should_quit(key)
        {
            return Ok(());
        }
    }
}

fn should_quit(key: KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && match key.code {
            KeyCode::Char('q') | KeyCode::Esc => key.modifiers.is_empty(),
            KeyCode::Char('c') => key.modifiers == KeyModifiers::CONTROL,
            _ => false,
        }
}

fn render(frame: &mut Frame) {
    let welcome = Paragraph::new("Ready.\n\nq / Esc / Ctrl+C to quit")
        .block(Block::bordered().title(" lml "));
    frame.render_widget(welcome, frame.area());
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn quit_keys_exit_only_on_press() {
        for (code, modifiers) in [
            (KeyCode::Char('q'), KeyModifiers::NONE),
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
            (KeyCode::Char('c'), KeyModifiers::NONE),
            (KeyCode::Char('q'), KeyModifiers::ALT),
            (KeyCode::Enter, KeyModifiers::NONE),
        ] {
            assert!(!should_quit(KeyEvent::new(code, modifiers)));
        }
    }

    #[test]
    fn welcome_screen_shows_exit_keys() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        terminal.draw(render).expect("render welcome screen");
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("lml"));
        assert!(text.contains("q / Esc / Ctrl+C to quit"));
    }

    #[test]
    fn tiny_terminal_can_render() {
        let mut terminal = Terminal::new(TestBackend::new(1, 1)).expect("test terminal");
        terminal.draw(render).expect("render tiny screen");
    }
}
