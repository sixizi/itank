use crate::game::{Game, Position, WallKind, FIELD_HEIGHT, FIELD_WIDTH};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

pub struct Renderer<'a, W> {
    output: W,
    _lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a, W> Renderer<'a, W>
where
    W: Write,
{
    pub fn new(output: W) -> Self {
        Self {
            output,
            _lifetime: std::marker::PhantomData,
        }
    }

    pub fn draw(&mut self, game: &Game) -> io::Result<()> {
        queue!(self.output, MoveTo(0, 0), Clear(ClearType::FromCursorDown))?;
        self.draw_hud(game)?;
        self.draw_field(game)?;
        self.draw_footer(game)?;
        self.output.flush()
    }

    fn draw_hud(&mut self, game: &Game) -> io::Result<()> {
        queue!(
            self.output,
            SetForegroundColor(Color::White),
            Print(format!(
                "Score {:4}  Lives {}  Wave {}/{}",
                game.score,
                game.lives,
                game.wave,
                crate::game::MAX_WAVE
            )),
            ResetColor,
            Print("\r\n")
        )
    }

    fn draw_field(&mut self, game: &Game) -> io::Result<()> {
        for y in 0..FIELD_HEIGHT {
            queue!(self.output, Print("|"))?;
            for x in 0..FIELD_WIDTH {
                let position = Position::new(x, y);
                if position == game.base {
                    queue!(
                        self.output,
                        SetForegroundColor(Color::Yellow),
                        Print('B'),
                        ResetColor
                    )?;
                } else if let Some(bullet) = game.bullets.iter().find(|b| b.pos == position) {
                    queue!(
                        self.output,
                        SetForegroundColor(if bullet.friendly {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                        Print('*'),
                        ResetColor
                    )?;
                } else if game.enemies.iter().any(|enemy| enemy.pos == position) {
                    queue!(
                        self.output,
                        SetForegroundColor(Color::Red),
                        Print('X'),
                        ResetColor
                    )?;
                } else if position == game.player.pos {
                    queue!(
                        self.output,
                        SetForegroundColor(Color::Green),
                        Print('P'),
                        ResetColor
                    )?;
                } else if let Some(kind) = game.walls.get(&position) {
                    let (symbol, color) = match kind {
                        WallKind::Brick => ('#', Color::DarkYellow),
                        WallKind::Steel => ('%', Color::DarkGrey),
                        WallKind::Water => ('~', Color::Blue),
                    };
                    queue!(
                        self.output,
                        SetForegroundColor(color),
                        Print(symbol),
                        ResetColor
                    )?;
                } else {
                    queue!(self.output, Print(' '))?;
                }
            }
            queue!(self.output, Print("|\r\n"))?;
        }
        Ok(())
    }

    fn draw_footer(&mut self, game: &Game) -> io::Result<()> {
        queue!(
            self.output,
            SetForegroundColor(Color::Grey),
            Print("Move: WASD/arrows  Fire: Space  Pause: P  Quit: Q"),
            ResetColor,
            Print("\r\n")
        )?;

        match game.status {
            crate::game::GameStatus::Paused => {
                queue!(
                    self.output,
                    SetForegroundColor(Color::Cyan),
                    Print("PAUSED - press P to resume"),
                    ResetColor
                )?;
            }
            crate::game::GameStatus::Victory => {
                queue!(
                    self.output,
                    SetForegroundColor(Color::Green),
                    Print("VICTORY - press Q to exit"),
                    ResetColor
                )?;
            }
            crate::game::GameStatus::Defeat => {
                queue!(
                    self.output,
                    SetForegroundColor(Color::Red),
                    Print("GAME OVER - press Q to exit"),
                    ResetColor
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
