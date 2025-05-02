// use std::{fs::File, process::ExitCode};
use std::process::ExitCode;

use anyhow::Result;

use crate::structs::Keyboard;

// TODO: Come up with something to log so we can use the actual function signature/imports
// pub fn setup(handle: &mut File) -> Result<ExitCode> {
pub fn setup() -> Result<ExitCode> {
    let keyboard: Keyboard = Keyboard::new()?;
    keyboard.print_keyslots();

    // TODO: Now that we have a basic keyboard, we need to read the corpus
    //
    // NOTE: Don't check clippy during these steps
    //
    // A dumb problem we need to solve is how to keep the test and production corpus data the same.
    // The simplest way is to do it with an rsync script, but I'm wondering if you can make Cargo
    // do it
    //
    // For reading through the input itself, just do it the most intuitive way to get the basic
    // logic down. I'm not sure yet how I want to use async/channels for optimization, so the
    // simpler the test logic is the easier it is to unwind
    // For reference, here is a single-threaded byte-by-byte read. Maybe not the most relevant now,
    // but perhaps for later:
    // https://github.com/coravacav/AdventOfCode/blob/main/2023-02/rust/src/part2.rs
    //
    // Once we can read in the corpus, score it. Just use the current basic enum info. It might be
    // a good idea to save the last key info, even if we don't use it right now, so we know roughly
    // how that needs to be done. I suppose you could just do a basic out-to-in test like what
    // Dvorak wanted
    //
    // Note - How to keep track of keyboard scoring. I guess it should be part of the struct. This
    // could simplify things, because when you clone the keyboard, you can also make sure to wipe
    // its score, so you only traverse corpus during the mutation phase on unscored keyboards. I
    // would guess then that every keyboard should try to hill climb. But if we use weighted
    // averaging, and we know they have already hit diminishing returns, maybe not. Questions
    // questions...
    //
    // Then implement keyboard randomization (in this case, don't worry about any of the forbidden
    // key logic. Too complicated for now). So we can randomize, score, then do a hill climbing to
    // improve it. This algorithm is the hardest part so better to just get it out of the way
    //
    // Then we can add population management. First do it by just creating random keyboards, then
    // add the lambda calcs/logic for different amounts of keys
    //
    // Add a display. Should show enough stats that you can see the progression. A visual chart
    // like the ones I've seen in the training videos would be nice but is a stretch goal. It
    // should also show the layout of the highest scoring keyboard. The biggest thing with the
    // stats is being able to intuit the amount of convergence
    //
    // We also need a method to save and load data. Including a way to press a key in the middle of
    // the program to finish the current iteration and save. It would be helpful to be able to scan
    // a directory and be able to find the best keyboard in one of the save files, but I'm not sure
    // if that's a Rust thing or a Python thing. The best and average each iteration should also be
    // saved so they can be visualized later.
    //
    // The visualization/saving part is very feature heavy, but it lets us complete the pipeline
    // and the data will likely be helpful later for testing.
    //
    // From there the rest of the details should be able to be filled in.

    return Ok(ExitCode::SUCCESS);
}
