use std::io::{Write, stdin, stdout};

use crossterm::{
    QueueableCommand,
    cursor::{MoveTo, RestorePosition, SavePosition},
    style::Print,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

#[cfg(windows)]
use crossterm::terminal::SetConsoleMode;
#[cfg(unix)]
use nix::sys::termios;

use crate::population::Population;

// TODO: Will not work on Windows. No items
pub struct Display {
    #[cfg(unix)]
    orig_termios: termios::Termios,
}

impl Display {
    const OFFSET_Y: u16 = 1;

    const POP_Y: u16 = 0 + Self::OFFSET_Y;

    const ITER_NUM_X: u16 = 11;
    const ITER_Y: u16 = 2 + Self::OFFSET_Y;

    const CURSOR_Y: u16 = 3 + Self::OFFSET_Y;

    pub fn new() -> Self {
        #[cfg(unix)]
        {
            let stdin = stdin();
            let orig_termios =
                termios::tcgetattr(&stdin).expect("Failed to get terminal attributes");

            let mut new_termios = orig_termios.clone();
            new_termios.local_flags.remove(termios::LocalFlags::ECHO);
            termios::tcsetattr(&stdin, termios::SetArg::TCSANOW, &new_termios)
                .expect("Failed to set terminal attributes");

            return Display { orig_termios };
        }

        #[cfg(windows)]
        {
            // On Windows, use crossterm to disable echo
            crossterm::execute!(
                stdout(),
                crossterm::terminal::SetConsoleMode(
                    crossterm::terminal::ConsoleMode::ENABLE_PROCESSED_INPUT
                )
            )
            .expect("Failed to set console mode");

            Display {
                current_iter: 0,
                prev_iter: 0,
                x,
                y,
            }
        }
    }

    pub fn draw_initial(&self, pop: &Population) {
        stdout()
            .queue(EnterAlternateScreen)
            .unwrap()
            .queue(Clear(ClearType::All))
            .unwrap()
            .queue(MoveTo(0, Self::POP_Y))
            .unwrap()
            .queue(Print(format!("Population: {:03}", pop.get_pop_size())))
            .unwrap()
            .queue(MoveTo(0, Self::ITER_Y))
            .unwrap()
            .queue(Print(format!("Iteration: {:05}", 0)))
            .unwrap()
            .queue(MoveTo(0, Self::CURSOR_Y))
            .unwrap();

        stdout().flush().unwrap();
    }

    pub fn update_iter(&mut self, iter: usize) {
        stdout()
            .queue(SavePosition)
            .unwrap()
            .queue(MoveTo(Self::ITER_NUM_X, Self::ITER_Y))
            .unwrap()
            .queue(Print(format!("{:05}", iter)))
            .unwrap()
            .queue(RestorePosition)
            .unwrap();

        stdout().flush().unwrap();
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        stdout()
            .queue(Clear(ClearType::All))
            .unwrap()
            .queue(LeaveAlternateScreen)
            .unwrap();

        stdout().flush().unwrap();

        #[cfg(unix)]
        {
            termios::tcsetattr(stdin(), termios::SetArg::TCSANOW, &self.orig_termios)
                .expect("Failed to restore terminal attributes");
        }

        #[cfg(windows)]
        {
            crossterm::execute!(
                stdout(),
                crossterm::terminal::SetConsoleMode(
                    crossterm::terminal::ConsoleMode::ENABLE_ECHO_INPUT
                )
            )
            .expect("Failed to restore console mode");
        }
    }
}
