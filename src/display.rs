use std::io::{Write, stdin, stdout};

use crossterm::{
    QueueableCommand,
    cursor::{MoveTo, RestorePosition, SavePosition},
    style::Print,
    // terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    // terminal::{Clear, ClearType, LeaveAlternateScreen},
    terminal::{Clear, ClearType},
};

use crate::keyboard::Keyboard;

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

// TODO: The consts should be a compile time macro
// TODO: You could print a theoretical climb count then the actual climbers separately. For
// simplicity, it will all go on one line for now
// TODO: Need a better solution for preventing typing while running
impl Display {
    const OFFSET_Y: u16 = 1;

    const POP_NAME: &str = "Population: ";
    const POP_Y: u16 = 0 + Self::OFFSET_Y;

    const CLIMB_NAME: &str = "Climber Count: ";
    // const CLIMB_LEN: usize = Self::CLIMB_NAME.len();
    // const ITER_CLIMB_X: u16 = Self::CLIMB_LEN as u16;
    const CLIMB_Y: u16 = Self::POP_Y + 1;

    const ELITE_NAME: &str = "Elite Count: ";
    // const ELITE_LEN: usize = Self::ELITE_NAME.len();
    // const ELITE_NUM_X: u16 = Self::ELITE_LEN as u16;
    const ELITE_Y: u16 = Self::CLIMB_Y + 1;

    const AVG_NAME: &str = "Average Climber Score: ";
    const AVG_LEN: usize = Self::AVG_NAME.len();
    const AVG_NUM_X: u16 = Self::AVG_LEN as u16;
    const AVG_Y: u16 = Self::ELITE_Y + 1;

    const KB_HEADER_Y: u16 = Self::AVG_Y + 2;
    const KB_INFO_Y: u16 = Self::KB_HEADER_Y + 1;
    const KB_NUM_Y: u16 = Self::KB_INFO_Y + 1;
    const KB_TOP_Y: u16 = Self::KB_NUM_Y + 1;
    const KB_HOME_Y: u16 = Self::KB_TOP_Y + 1;
    const KB_BOT_Y: u16 = Self::KB_HOME_Y + 1;

    const ITER_NAME: &str = "Iteration: ";
    const ITER_LEN: usize = Self::ITER_NAME.len();
    const ITER_NUM_X: u16 = Self::ITER_LEN as u16;
    const ITER_Y: u16 = Self::KB_BOT_Y + 2;

    const MUT_NAME: &str = "Mutation Ranges: ";
    const MUT_LEN: usize = Self::MUT_NAME.len();
    const MUT_NUM_X: u16 = Self::MUT_LEN as u16;
    const MUT_Y: u16 = Self::ITER_Y + 1;

    const EVAL_NAME: &str = "Evaluating: ";
    const EVAL_LEN: usize = Self::EVAL_NAME.len();
    const EVAL_NUM_X: u16 = Self::EVAL_LEN as u16;
    const EVAL_Y: u16 = Self::MUT_Y + 1;

    const CLIMB_HEADER_Y: u16 = Self::EVAL_Y + 2;
    const CLIMB_INFO_Y: u16 = Self::CLIMB_HEADER_Y + 1;
    const CLIMB_STATS_Y: u16 = Self::CLIMB_INFO_Y + 1;

    const CURSOR_Y: u16 = Self::CLIMB_STATS_Y + 1;

    pub fn new() -> Self {
        #[cfg(unix)]
        {
            let stdin = stdin();
            let orig_termios =
                termios::tcgetattr(&stdin).expect("Failed to get terminal attributes");

            let mut new_termios = orig_termios.clone();
            new_termios.local_flags.remove(termios::LocalFlags::ECHO);
            // termios::tcsetattr(&stdin, termios::SetArg::TCSANOW, &new_termios)
            //     .expect("Failed to set terminal attributes");

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

    // TODO: Fine with unwrap for now, but these display functions should return errors since
    // they're interacting with external IO, so then that propagation needs to be handled
    // TODO: For the format numbers, get the correct amounts at compile time. This will also help
    // out windows because the display struct will have a value there
    // TODO: In the inevitable refactor, only setting the alternate screen and clearing the
    // terminal are really inherent to the process of setting up the ability to draw. Everything
    // else can be broken out and maybe even reused
    // TODO: I would guess as well that we don't want to trigger this until user input is handled
    // TODO: The climb info and climb stats should show a place holder when no active climb is
    // happening
    pub fn draw_initial(&self, pop: &Population) {
        stdout()
            // .queue(EnterAlternateScreen)
            // .unwrap()
            .queue(Clear(ClearType::All))
            .unwrap()
            .queue(MoveTo(0, Self::POP_Y))
            .unwrap()
            .queue(Print(format!(
                "{}{:03}",
                Self::POP_NAME,
                pop.get_pop_size()
            )))
            .unwrap()
            .queue(MoveTo(0, Self::CLIMB_Y))
            .unwrap()
            .queue(Print(format!(
                "{}{:02}",
                Self::CLIMB_NAME,
                pop.get_climb_cnt()
            )))
            .unwrap()
            .queue(MoveTo(0, Self::ELITE_Y))
            .unwrap()
            .queue(Print(format!(
                "{}{:01}",
                Self::ELITE_NAME,
                pop.get_elite_cnt()
            )))
            .unwrap()
            .queue(MoveTo(0, Self::AVG_Y))
            .unwrap()
            .queue(Print(format!("{} --", Self::AVG_NAME,)))
            .unwrap()
            .queue(MoveTo(0, Self::KB_HEADER_Y))
            .unwrap()
            .queue(Print("-- Best Keyboard --"))
            .unwrap()
            .queue(MoveTo(0, Self::ITER_Y))
            .unwrap()
            .queue(Print(format!("{}{:05}", Self::ITER_NAME, 0)))
            .unwrap()
            .queue(MoveTo(0, Self::MUT_Y))
            .unwrap()
            .queue(Print(format!(
                "{}{:02}, {:02} | {:02}, {:02} | {:02}, {:02}",
                Self::MUT_NAME,
                0,
                0,
                0,
                0,
                0,
                0,
            )))
            .unwrap()
            .queue(MoveTo(0, Self::EVAL_Y))
            .unwrap()
            .queue(Print(format!("{} --", Self::EVAL_NAME)))
            .unwrap()
            .queue(MoveTo(0, Self::CLIMB_HEADER_Y))
            .unwrap()
            .queue(Print(format!("Climb Info:")))
            .unwrap()
            .queue(MoveTo(0, Self::CLIMB_INFO_Y))
            .unwrap()
            .queue(Print(format!("{}", " ".repeat(155))))
            .unwrap()
            .queue(MoveTo(0, Self::CLIMB_STATS_Y))
            .unwrap()
            .queue(Print(format!("{}", " ".repeat(155))))
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

    pub fn update_avg(&mut self, score: f64) {
        stdout()
            .queue(SavePosition)
            .unwrap()
            .queue(MoveTo(Self::AVG_NUM_X, Self::AVG_Y))
            .unwrap()
            .queue(Print(format!("{}", score)))
            .unwrap()
            .queue(RestorePosition)
            .unwrap();

        stdout().flush().unwrap();
    }

    pub fn update_kb(&mut self, kb: &Keyboard) {
        let info: String = format!(
            "Generation: {:05}, ID: {:07}, Score: {:18}, Positive Iterations: {:05}",
            kb.get_generation(),
            kb.get_id(),
            kb.get_score(),
            kb.get_pos_iter()
        );

        let kb_chars: Vec<Vec<char>> = kb.get_display_chars();

        // The padding in the KB strings is incase the single quotes escape changes the size of a
        // row
        stdout()
            .queue(SavePosition)
            .unwrap()
            .queue(MoveTo(0, Self::KB_INFO_Y))
            .unwrap()
            .queue(Print(info))
            .unwrap()
            .queue(MoveTo(0, Self::KB_NUM_Y))
            .unwrap()
            .queue(Print(format!("{:?}   ", kb_chars[0])))
            .unwrap()
            .queue(MoveTo(0, Self::KB_TOP_Y))
            .unwrap()
            .queue(Print(format!("{:?}   ", kb_chars[1])))
            .unwrap()
            .queue(MoveTo(0, Self::KB_HOME_Y))
            .unwrap()
            .queue(Print(format!("{:?}   ", kb_chars[2])))
            .unwrap()
            .queue(MoveTo(0, Self::KB_BOT_Y))
            .unwrap()
            .queue(Print(format!("{:?}   ", kb_chars[3])))
            .unwrap()
            .queue(RestorePosition)
            .unwrap();

        stdout().flush().unwrap();
    }

    // TODO: Rather than just update the whole line, it should be possible to feed this struct the
    // individual pieces of climb data and update them in place. More logic, but less terminal IO
    // In particular, we are allocating strings to pass into these functions, when it really should
    // be possible to paass in the stack numbers. We can see here and in the climb_stats fn that
    // Print() is accepting &strs, so should it not be possible to have a permanently allocated
    // &str we edit in place?
    pub fn update_climb_info(&mut self, info: &str) {
        stdout()
            .queue(SavePosition)
            .unwrap()
            .queue(MoveTo(0, Self::CLIMB_INFO_Y))
            .unwrap()
            .queue(Print(info))
            .unwrap()
            .queue(RestorePosition)
            .unwrap();

        stdout().flush().unwrap();
    }

    // TODO: Rather than just update the whole line, it should be possible to feed this struct the
    // individual pieces of climb data and update them in place. More logic, but less terminal IO
    pub fn update_climb_stats(&mut self, stats: &str) {
        stdout()
            .queue(SavePosition)
            .unwrap()
            .queue(MoveTo(0, Self::CLIMB_STATS_Y))
            .unwrap()
            .queue(Print(stats))
            .unwrap()
            .queue(RestorePosition)
            .unwrap();

        stdout().flush().unwrap();
    }

    // TODO: Long function signature
    pub fn update_mut_values(
        &mut self,
        low_b: usize,
        low_t: usize,
        mid_b: usize,
        mid_t: usize,
        high_b: usize,
        high_t: usize,
        huge_b: usize,
        huge_t: usize,
    ) {
        stdout()
            .queue(SavePosition)
            .unwrap()
            .queue(MoveTo(Self::MUT_NUM_X, Self::MUT_Y))
            .unwrap()
            .queue(Print(format!(
                "{:02}, {:02} | {:02}, {:02} | {:02}, {:02} | {:02}, {:02}",
                low_b, low_t, mid_b, mid_t, high_b, high_t, huge_b, huge_t
            )))
            .unwrap()
            .queue(RestorePosition)
            .unwrap();

        stdout().flush().unwrap();
    }

    pub fn update_eval(&mut self, num: usize) {
        let to_print = if num > 0 {
            format!("{:03}", num)
        } else {
            format!("---")
        };

        stdout()
            .queue(SavePosition)
            .unwrap()
            .queue(MoveTo(Self::EVAL_NUM_X, Self::EVAL_Y))
            .unwrap()
            .queue(Print(to_print))
            .unwrap()
            .queue(RestorePosition)
            .unwrap();

        stdout().flush().unwrap();
    }
}

// TODO: How to make this fun on ctrl+c?
// impl Drop for Display {
//     fn drop(&mut self) {
//         stdout()
//             .queue(Clear(ClearType::All))
//             .unwrap()
//             .queue(LeaveAlternateScreen)
//             .unwrap();
//
//         stdout().flush().unwrap();
//
//         #[cfg(unix)]
//         {
//             termios::tcsetattr(stdin(), termios::SetArg::TCSANOW, &self.orig_termios)
//                 .expect("Failed to restore terminal attributes");
//         }
//
//         #[cfg(windows)]
//         {
//             crossterm::execute!(
//                 stdout(),
//                 crossterm::terminal::SetConsoleMode(
//                     crossterm::terminal::ConsoleMode::ENABLE_ECHO_INPUT
//                 )
//             )
//             .expect("Failed to restore console mode");
//         }
//     }
// }
