mod game;
mod renderer;

use crossterm::{
    cursor::Hide,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use game::{Direction, Game, InputAction};
use renderer::Renderer;
use std::{
    io::{self, Write},
    time::Duration,
};

fn main() -> io::Result<()> {
    let _terminal = TerminalGuard::new()?;
    let stdout = io::stdout();
    let mut renderer = Renderer::new(stdout.lock());
    let mut game = Game::new();

    renderer.draw(&game)?;
    loop {
        let action = read_action()?;
        if action == InputAction::Quit {
            break;
        }
        game.tick(action);
        renderer.draw(&game)?;

        if matches!(
            game.status,
            game::GameStatus::Victory | game::GameStatus::Defeat
        ) {
            wait_for_exit()?;
            break;
        }
    }

    Ok(())
}

fn read_action() -> io::Result<InputAction> {
    if !event::poll(Duration::from_millis(30))? {
        return Ok(InputAction::None);
    }
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(InputAction::None);
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(InputAction::Quit);
        }
        return Ok(match key.code {
            KeyCode::Char('w') | KeyCode::Up => InputAction::Move(Direction::Up),
            KeyCode::Char('s') | KeyCode::Down => InputAction::Move(Direction::Down),
            KeyCode::Char('a') | KeyCode::Left => InputAction::Move(Direction::Left),
            KeyCode::Char('d') | KeyCode::Right => InputAction::Move(Direction::Right),
            KeyCode::Char(' ') => InputAction::Fire,
            KeyCode::Char('p') => InputAction::Pause,
            KeyCode::Char('q') | KeyCode::Esc => InputAction::Quit,
            _ => InputAction::None,
        });
    }
    Ok(InputAction::None)
}

fn wait_for_exit() -> io::Result<()> {
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter)
                {
                    return Ok(());
                }
            }
        }
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
        let _ = io::stdout().flush();
    }
}
