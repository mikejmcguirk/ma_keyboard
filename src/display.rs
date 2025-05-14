use std::io;
use std::io::{Write as _, stdout};

use crate::population::Population;

use {
    // anyhow::Result,
    crossterm::{
        QueueableCommand as _,
        cursor::{MoveTo, RestorePosition, SavePosition},
        style::Print,
        // terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
        // terminal::{Clear, ClearType, LeaveAlternateScreen},
        terminal::{Clear, ClearType},
    },
};

use crate::keyboard::Keyboard;

const OFFSET_Y: u16 = 1;

const POP_NAME: &str = "Population: ";
const POP_Y: u16 = OFFSET_Y;

const CLIMB_NAME: &str = "Climber Count: ";
// const CLIMB_LEN: usize = CLIMB_NAME.len();
// const ITER_CLIMB_X: u16 = CLIMB_LEN as u16;
const CLIMB_Y: u16 = POP_Y + 1;

const ELITE_NAME: &str = "Elite Count: ";
// const ELITE_LEN: usize = ELITE_NAME.len();
// const ELITE_NUM_X: u16 = ELITE_LEN as u16;
const ELITE_Y: u16 = CLIMB_Y + 1;

const AVG_NAME: &str = "Average Climber Score: ";
const AVG_LEN: usize = AVG_NAME.len();
// The as conversion takes place at compile time
#[expect(clippy::as_conversions)]
#[expect(clippy::cast_possible_truncation)]
const AVG_NUM_X: u16 = AVG_LEN as u16;
const AVG_Y: u16 = ELITE_Y + 1;

const KB_HEADER_Y: u16 = AVG_Y + 2;
const KB_INFO_Y: u16 = KB_HEADER_Y + 1;
const KB_NUM_Y: u16 = KB_INFO_Y + 1;
const KB_TOP_Y: u16 = KB_NUM_Y + 1;
const KB_HOME_Y: u16 = KB_TOP_Y + 1;
const KB_BOT_Y: u16 = KB_HOME_Y + 1;

const ITER_NAME: &str = "Iteration: ";
const ITER_LEN: usize = ITER_NAME.len();
// The as conversion takes place at compile time
#[expect(clippy::as_conversions)]
#[expect(clippy::cast_possible_truncation)]
const ITER_NUM_X: u16 = ITER_LEN as u16;
const ITER_Y: u16 = KB_BOT_Y + 2;

const MUT_NAME: &str = "Mutation Ranges: ";
const MUT_LEN: usize = MUT_NAME.len();
// The as conversion takes place at compile time
#[expect(clippy::as_conversions)]
#[expect(clippy::cast_possible_truncation)]
const MUT_NUM_X: u16 = MUT_LEN as u16;
const MUT_Y: u16 = ITER_Y + 1;

const EVAL_NAME: &str = "Evaluating: ";
const EVAL_LEN: usize = EVAL_NAME.len();
// The as conversion takes place at compile time
#[expect(clippy::as_conversions)]
#[expect(clippy::cast_possible_truncation)]
const EVAL_NUM_X: u16 = EVAL_LEN as u16;
const EVAL_Y: u16 = MUT_Y + 1;

const CLIMB_HEADER_Y: u16 = EVAL_Y + 2;
const CLIMB_INFO_Y: u16 = CLIMB_HEADER_Y + 1;
const CLIMB_STATS_Y: u16 = CLIMB_INFO_Y + 1;

const CURSOR_Y: u16 = CLIMB_STATS_Y + 1;

// TODO: Need a better solution for preventing typing while running
// TODO: For the format numbers, get the correct amounts at compile time. This will also help
// out windows because the display struct will have a value there

// TODO: In the inevitable refactor, only setting the alternate screen and clearing the
// terminal are really inherent to the process of setting up the ability to draw. Everything
// else can be broken out and maybe even reused
// TODO: I would guess as well that we don't want to trigger this until user input is handled
pub fn draw_initial(pop: &Population) -> io::Result<()> {
    stdout().queue(Clear(ClearType::All))?;

    stdout().queue(MoveTo(0, POP_Y))?;
    stdout().queue(Print(format!("{}{:03}", POP_NAME, pop.get_pop_size())))?;
    stdout().queue(MoveTo(0, CLIMB_Y))?;
    stdout().queue(Print(format!("{}{:02}", CLIMB_NAME, pop.get_climb_cnt())))?;
    stdout().queue(MoveTo(0, ELITE_Y))?;
    stdout().queue(Print(format!("{}{:01}", ELITE_NAME, pop.get_elite_cnt())))?;
    stdout().queue(MoveTo(0, AVG_Y))?;
    stdout().queue(Print(format!("{} --", AVG_NAME,)))?;

    stdout().queue(MoveTo(0, KB_HEADER_Y))?;
    stdout().queue(Print("-- Best Keyboard --"))?;

    stdout().queue(MoveTo(0, ITER_Y))?;
    stdout().queue(Print(format!("{}{:05}", ITER_NAME, 0_i32)))?;
    stdout().queue(MoveTo(0, MUT_Y))?;
    stdout().queue(Print(format!(
        "{}{:02}, {:02} | {:02}, {:02} | {:02}, {:02}",
        MUT_NAME, 0_i32, 0_i32, 0_i32, 0_i32, 0_i32, 0_i32,
    )))?;
    stdout().queue(MoveTo(0, EVAL_Y))?;
    stdout().queue(Print(format!("{} --", EVAL_NAME)))?;
    stdout().queue(MoveTo(0, CLIMB_HEADER_Y))?;

    stdout().queue(Print("Climb Info:"))?;
    stdout().queue(MoveTo(0, CLIMB_INFO_Y))?;
    stdout().queue(Print(" ".repeat(155)))?;
    stdout().queue(MoveTo(0, CLIMB_STATS_Y))?;
    stdout().queue(Print(" ".repeat(155)))?;
    stdout().queue(MoveTo(0, CURSOR_Y))?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_iter(iter: usize) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(ITER_NUM_X, ITER_Y))?;
    stdout().queue(Print(format!("{:05}", iter)))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_avg(score: f64) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(AVG_NUM_X, AVG_Y))?;
    stdout().queue(Print(format!("{}", score)))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_kb(kb: &Keyboard) -> io::Result<()> {
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
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(0, KB_INFO_Y))?;
    stdout().queue(Print(info))?;
    stdout().queue(MoveTo(0, KB_NUM_Y))?;
    stdout().queue(Print(format!("{:?}   ", kb_chars[0])))?;
    stdout().queue(MoveTo(0, KB_TOP_Y))?;
    stdout().queue(Print(format!("{:?}   ", kb_chars[1])))?;
    stdout().queue(MoveTo(0, KB_HOME_Y))?;
    stdout().queue(Print(format!("{:?}   ", kb_chars[2])))?;
    stdout().queue(MoveTo(0, KB_BOT_Y))?;
    stdout().queue(Print(format!("{:?}   ", kb_chars[3])))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

// TODO: You could print a theoretical climb count then the actual climbers separately. For
// simplicity, it will all go on one line for now
// TODO: The climb info and climb stats should show a place holder when no active climb is
// happening
pub fn update_climb_info(info: &str) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(0, CLIMB_INFO_Y))?;
    stdout().queue(Print(info))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

// TODO: Rather than just update the whole line, it should be possible to feed this struct the
// individual pieces of climb data and update them in place. More logic, but less terminal IO
// In particular, we are allocating strings to pass into these functions, when it really should
// be possible to paass in the stack numbers. We can see here and in the climb_stats fn that
// Print() is accepting &strs, so should it not be possible to have a permanently allocated
// &str we edit in place?
pub fn update_climb_stats(stats: &str) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(0, CLIMB_STATS_Y))?;
    stdout().queue(Print(stats))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

// TODO: Long function signature
pub fn update_mut_values(
    low_b: usize,
    low_t: usize,
    mid_b: usize,
    mid_t: usize,
    high_b: usize,
    high_t: usize,
    huge_b: usize,
    huge_t: usize,
) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(MUT_NUM_X, MUT_Y))?;
    stdout().queue(Print(format!(
        "{:02}, {:02} | {:02}, {:02} | {:02}, {:02} | {:02}, {:02}",
        low_b, low_t, mid_b, mid_t, high_b, high_t, huge_b, huge_t
    )))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_eval(num: usize) -> io::Result<()> {
    let to_print = if num > 0 {
        format!("{:03}", num)
    } else {
        "---".to_owned()
    };

    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(EVAL_NUM_X, EVAL_Y))?;
    stdout().queue(Print(to_print))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}
