use std::io;
use std::io::{Write as _, stdout};

use crate::population::Population;

use crossterm::{
    QueueableCommand as _,
    cursor::{MoveTo, RestorePosition, SavePosition},
    style::Print,
    terminal::{Clear, ClearType},
};

use crate::keyboard::Keyboard;

const OFFSET_Y: u16 = 1;

const POP_HEADER_Y: u16 = OFFSET_Y;

// const POP_STATS_NAME: &str = "Population Count: ";
// const POP_STATS_LEN: usize = POP_STATS_NAME.len();
// const POP_STATS_NUM_X: u16 = POP_STATS_LEN as u16;
const POP_STATS_Y: u16 = POP_HEADER_Y + 1;

// const AVG_NAME: &str = "Average Climber Score: ";
// const AVG_LEN: usize = AVG_NAME.len();
// const AVG_NUM_X: u16 = AVG_LEN as u16;
// const AVG_Y: u16 = POP_STATS_Y + 1;

const KB_HEADER_Y: u16 = POP_STATS_Y + 2;
const KB_INFO_Y: u16 = KB_HEADER_Y + 1;
const KB_NUM_Y: u16 = KB_INFO_Y + 1;
const KB_TOP_Y: u16 = KB_NUM_Y + 1;
const KB_HOME_Y: u16 = KB_TOP_Y + 1;
const KB_BOT_Y: u16 = KB_HOME_Y + 1;

const ITER_NAME: &str = "Iteration: ";
const ITER_LEN: usize = ITER_NAME.len();
const ITER_NUM_X: u16 = ITER_LEN as u16;
const ITER_Y: u16 = KB_BOT_Y + 2;

const CUR_POP_HEADER_Y: u16 = ITER_Y + 2;
const CUR_POP_STATS_Y: u16 = CUR_POP_HEADER_Y + 1;

const CUR_AVG_NAME: &str = "Average Climber Score: ";
const CUR_AVG_LEN: usize = CUR_AVG_NAME.len();
const CUR_AVG_NUM_X: u16 = CUR_AVG_LEN as u16;
const CUR_AVG_Y: u16 = CUR_POP_STATS_Y + 1;

const EVAL_NAME: &str = "Evaluating: ";
const EVAL_LEN: usize = EVAL_NAME.len();
const EVAL_NUM_X: u16 = EVAL_LEN as u16;
const EVAL_Y: u16 = CUR_AVG_Y + 1;

const CLIMB_HEADER_Y: u16 = EVAL_Y + 2;
const CLIMB_INFO_Y: u16 = CLIMB_HEADER_Y + 1;
const CLIMB_STATS_Y: u16 = CLIMB_INFO_Y + 1;

const QWERTY_NAME: &str = "Qwerty Score: ";
const QWERTY_LEN: usize = QWERTY_NAME.len();
const QWERTY_NUM_X: u16 = QWERTY_LEN as u16;
const QWERTY_Y: u16 = CLIMB_STATS_Y + 1;

const DVORAK_NAME: &str = "Dvorak Score: ";
const DVORAK_LEN: usize = DVORAK_NAME.len();
const DVORAK_NUM_X: u16 = DVORAK_LEN as u16;
const DVORAK_Y: u16 = QWERTY_Y + 1;

const CURSOR_Y: u16 = DVORAK_Y + 1;

// FUTURE: This probably all needs to be redone, but don't want to get deep into it until I know
// what the outputs actually are
pub fn initial_dsp() -> io::Result<()> {
    let pop_id = format!("Population ID: {:02}, ", 0);
    let pop_cnt = format!("Population Count: {:02}, ", 0);
    let mutation = format!("Mutation: {:01}, ", 0);
    let elite_cnt = format!("Elites: {:01}, ", 0);
    let climb_cnt = format!("Climbers: {:02}, ", 0);
    let k_temp = format!("K Temp: {:07.04}, ", 0);
    let score_decay = format!("Decay: {:05.03}, ", 0);
    let avg_climb_iter = format!("Avg. Climb Iter: {:09.02}", 0);

    stdout().queue(Clear(ClearType::All))?;

    stdout().queue(MoveTo(0, POP_HEADER_Y))?;
    stdout().queue(Print("-- Best Population --"))?;
    stdout().queue(MoveTo(0, POP_STATS_Y))?;
    stdout().queue(Print(format!(
        "{}{}{}{}{}{}{}{}",
        pop_id, pop_cnt, mutation, elite_cnt, climb_cnt, k_temp, score_decay, avg_climb_iter
    )))?;
    // stdout().queue(MoveTo(0, AVG_Y))?;
    // stdout().queue(Print(format!("{} --", AVG_NAME,)))?;

    stdout().queue(MoveTo(0, KB_HEADER_Y))?;
    stdout().queue(Print("-- Best Keyboard --"))?;

    stdout().queue(MoveTo(0, ITER_Y))?;
    stdout().queue(Print(format!("{}{:05}", ITER_NAME, 0_i32)))?;

    stdout().queue(MoveTo(0, CUR_POP_HEADER_Y))?;
    stdout().queue(Print("-- Current Population --"))?;
    stdout().queue(MoveTo(0, CUR_POP_STATS_Y))?;
    stdout().queue(Print(format!(
        "{}{}{}{}{}{}{}{}",
        pop_id, pop_cnt, mutation, elite_cnt, climb_cnt, k_temp, score_decay, avg_climb_iter
    )))?;
    stdout().queue(MoveTo(0, CUR_AVG_Y))?;
    stdout().queue(Print(format!("{} --", CUR_AVG_NAME,)))?;
    stdout().queue(MoveTo(0, EVAL_Y))?;
    stdout().queue(Print(format!("{} --", EVAL_NAME)))?;
    stdout().queue(MoveTo(0, CLIMB_HEADER_Y))?;

    stdout().queue(Print("Climb Info:"))?;
    stdout().queue(MoveTo(0, CLIMB_INFO_Y))?;
    stdout().queue(Print(" ".repeat(155)))?;
    stdout().queue(MoveTo(0, CLIMB_STATS_Y))?;
    stdout().queue(Print(" ".repeat(155)))?;

    stdout().queue(MoveTo(0, QWERTY_Y))?;
    stdout().queue(Print(QWERTY_NAME))?;
    stdout().queue(MoveTo(0, DVORAK_Y))?;
    stdout().queue(Print(DVORAK_NAME))?;

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

pub fn update_best_pop_dsp(population: &Population) -> io::Result<()> {
    let pop_id = format!("Population ID: {:05}, ", population.get_id());
    let pop_cnt = format!("Population Count: {:02}, ", population.get_pop_cnt());
    let mutation = format!("Mutation: {:01}, ", population.get_mutation());
    let elite_cnt = format!("Elites: {:01}, ", population.get_elite_cnt());
    let climb_cnt = format!("Climbers: {:02}, ", population.get_climb_cnt());
    let k_temp = format!("K Temp: {:08.04}, ", population.get_k_temp());
    let score_decay = format!("Decay: {:05.03}, ", population.get_score_decay());
    let avg_climb_iter = format!("Avg. Climb Iter: {:09.02}", population.get_avg_climb_iter());

    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(0, POP_STATS_Y))?;
    stdout().queue(Print(format!(
        "{}{}{}{}{}{}{}{}",
        pop_id, pop_cnt, mutation, elite_cnt, climb_cnt, k_temp, score_decay, avg_climb_iter
    )))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_cur_pop_dsp(population: &Population) -> io::Result<()> {
    let pop_id = format!("Population ID: {:05}, ", population.get_id());
    let pop_cnt = format!("Population Count: {:02}, ", population.get_pop_cnt());
    let mutation = format!("Mutation: {:01}, ", population.get_mutation());
    let elite_cnt = format!("Elites: {:01}, ", population.get_elite_cnt());
    let climb_cnt = format!("Climbers: {:02}, ", population.get_climb_cnt());
    let k_temp = format!("K Temp: {:08.04}, ", population.get_k_temp());
    let score_decay = format!("Decay: {:05.03}, ", population.get_score_decay());
    let avg_climb_iter = format!("Avg. Climb Iter: {:09.02}", population.get_avg_climb_iter());

    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(0, CUR_POP_STATS_Y))?;
    stdout().queue(Print(format!(
        "{}{}{}{}{}{}{}{}",
        pop_id, pop_cnt, mutation, elite_cnt, climb_cnt, k_temp, score_decay, avg_climb_iter
    )))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_qwerty(score: f64) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(QWERTY_NUM_X, QWERTY_Y))?;
    stdout().queue(Print(format!("{:05}", score)))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_dvorak(score: f64) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(DVORAK_NUM_X, DVORAK_Y))?;
    stdout().queue(Print(format!("{:05}", score)))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

// pub fn update_best_avg(score: f64) -> io::Result<()> {
//     stdout().queue(SavePosition)?;
//     stdout().queue(MoveTo(AVG_NUM_X, AVG_Y))?;
//     stdout().queue(Print(format!("{}", score)))?;
//     stdout().queue(RestorePosition)?;
//
//     stdout().flush()?;
//
//     return Ok(());
// }

pub fn update_cur_avg(score: f64) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(CUR_AVG_NUM_X, CUR_AVG_Y))?;
    stdout().queue(Print(format!("{}", score)))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

// At least for now, it would be more contrived to iterate through everything
// FUTURE: This is not a good long term solution though
pub fn update_best_kb(kb: &Keyboard) -> io::Result<()> {
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

pub fn update_climb_info(info: &str) -> io::Result<()> {
    stdout().queue(SavePosition)?;
    stdout().queue(MoveTo(0, CLIMB_INFO_Y))?;
    stdout().queue(Print(info))?;
    stdout().queue(RestorePosition)?;

    stdout().flush()?;

    return Ok(());
}

pub fn update_eval_dsp(num: usize) -> io::Result<()> {
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
