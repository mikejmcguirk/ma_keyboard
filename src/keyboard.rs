use std::{
    collections::HashMap,
    io::{Write as _, stdout},
};

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, rngs::SmallRng},
};

use crate::{
    enums::{Col, CorpusErr, Finger, Hand, KeySetError, KeyTemplate, ListType, Row},
    kb_components::{HandInfo, Key, KeyList, KeySlot, LocInfo},
};

#[derive(Clone)]
// NOTE: RNG is not stored in the keyboards because, whenever the keyboards are cloned or mutated,
// the RNG in either the clone or the original needs to be re-seeded. This makes logging the
// original seed pointless
// NOTE: The keyboard and its components use u8 to represent characters. This makes the code a bit
// less straight forward, but results in an order of magnitude+ improvement in speed when reading
// through the long corpus strings
// TODO: Elite can just be a public field. It is read and set directly
// TODO: Instead of a HashMap, store an ASCII table. When you read a byte, use that as the ASCII
// table index to get the Slot
// TODO: FUlly blocked keys like the number row should not be considered for shuffling
pub struct Keyboard {
    keyslots: Vec<KeySlot>,
    slot_ref: HashMap<u8, usize>,
    last_slot_idx: Option<usize>,
    generation: usize,
    id: usize,
    lineage: String,
    evaluated: bool,
    score: f64,
    pub is_elite: bool,
}

impl Keyboard {
    // PERF: This can be optimized by pre-allocating keyslots and unsafely writing to it
    // TODO: At some point this logic will need to handle keys that are not totally randomized. As
    // much of this logic as possible should be tied to the enums. The key though is it needs to
    // flow intuitively. Right now, col.get_finger() intuitively makes sense because we know each
    // keyboard column has a finger mapped to it. You don't really need to jump to definition to
    // understand it
    // TODO: The logic to insert a key into a lot in make_origin should be common with the shuffle
    // logic. Too early right now, but do this eventually
    // TODO: Broad idea for key/slot rules is - There should be some kind of menu for which keys
    // are allowed in which slots that the various functions can check. This could create a
    // challenge in terms of tying it together, but separating the pieces of how this is managed
    // could cause an unforeseen contradiction in rules
    // TODO: Possible idea for more sophisticated keyboard building, hard code the elements of the
    // layout, like the number row, then flatten the array into the keyslot vec. We can use those
    // pieces to build the invalid swap indexes (like the number keys). Doing hard codes kinda
    // sucks, but it's better than building messes of rules where we know the final result anyway
    // Do still be judicious about key restrictions, at least to start. See what the algorithm does
    // before locking down
    // The idea then is that you can pull the swappable part of keyslots as a slice for the various
    // functions below, and then still use the length of the slice to assess the validity of
    // various arguments and indexes cleanly
    // TODO: After the population refactor is finished, it should be possible to make get_keyslots
    // private again
    // TODO: This should not error. It takes no external input
    // TODO: shuffle_all should be run automatically when making an original keyboard, but
    // want to wait on the architecture to settle in before doing this
    pub fn make_qwerty(id: usize) -> Self {
        let mut keyslots: Vec<KeySlot> = Vec::new();
        let qwerty_slots: Vec<KeySlot> = get_qwerty_slots();

        for slot in &qwerty_slots {
            keyslots.push(slot.clone());
        }

        let mut slot_ref: HashMap<u8, usize> = HashMap::new();
        let mut base_keys: Vec<u8> = Vec::new();
        for i in 0..keyslots.len() {
            let key: Key = keyslots[i].get_key();

            slot_ref.insert(key.get_base(), i);
            slot_ref.insert(key.get_shift(), i);

            base_keys.push(key.get_base());
        }

        let lineage: String = format!("0.{}", id);

        return Keyboard {
            keyslots,
            slot_ref,
            last_slot_idx: None,
            generation: 0,
            id,
            lineage,
            evaluated: false,
            score: 0.0,
            is_elite: false,
        };
    }

    // TODO: It is more idiomatic to this project for the keyboard object to spawn a clone of
    // itself, potentially with shuffling already built in. You could probably also just pass a
    // flag to it saying whether or not to increment the generation. But would have to think about
    // this and how to implement it. As is, while this function isn't theoretically good design, I
    // can manually control the generation as well as how I shuffle it
    // One issue that nags at me is, if you create using new, it sets generation to 1, which is
    // correct for how the code is currently used, but is inflexible. Alternatively, it's too
    // flexible, because outside of the initial 20 keyboards to get the process going, keyboards
    // should only spawn from each other rather than being airdropped in
    // The other is, this function requires exposing get methods that only exist for this one
    // purpose, which hurts encapsulation
    pub fn mutate(kb: &Keyboard, gen_input: usize, id: usize) -> Self {
        let keyslots: Vec<KeySlot> = kb.get_keyslots().to_vec();
        let slot_ref: HashMap<u8, usize> = kb.get_slot_ref().clone();
        let last_slot_idx: Option<usize> = None;
        let generation: usize = gen_input;
        let lineage: String = format!("{}-{}.{}", kb.get_lineage(), gen_input, id);

        let evaluated: bool = kb.get_eval_status();
        let score: f64 = kb.get_score();

        return Self {
            keyslots,
            slot_ref,
            last_slot_idx,
            generation,
            id,
            lineage,
            evaluated,
            score,
            is_elite: false,
        };
    }

    pub fn copy_kb(kb: &Keyboard) -> Self {
        let keyslots: Vec<KeySlot> = kb.get_keyslots().to_vec();
        let slot_ref: HashMap<u8, usize> = kb.get_slot_ref().clone();
        let last_slot_idx: Option<usize> = None;
        let generation: usize = kb.get_generation();
        let id: usize = kb.get_id();
        let lineage: String = kb.get_lineage();

        let evaluated: bool = kb.get_eval_status();
        let score: f64 = kb.get_score();
        let is_elite: bool = kb.is_elite;

        return Self {
            keyslots,
            slot_ref,
            last_slot_idx,
            generation,
            id,
            lineage,
            evaluated,
            score,
            is_elite,
        };
    }

    pub fn get_generation(&self) -> usize {
        return self.generation;
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    pub fn get_key_at_idx(&self, idx: usize) -> Key {
        return self.keyslots[idx].get_key();
    }

    pub fn get_slot_cnt(&self) -> usize {
        return self.keyslots.len();
    }

    // TODO: Clone bad
    pub fn get_lineage(&self) -> String {
        return self.lineage.clone();
    }

    pub fn get_keyslots(&self) -> &[KeySlot] {
        return &self.keyslots;
    }

    fn get_slot_ref(&self) -> &HashMap<u8, usize> {
        return &self.slot_ref;
    }

    pub fn get_eval_status(&self) -> bool {
        return self.evaluated;
    }

    // contains panic
    pub fn shuffle_all(&mut self, rng: &mut SmallRng) {
        self.evaluated = false;

        let mut i: usize = 0;
        while i < (self.keyslots.len() - 1) {
            let j: usize = rng.random_range((i + 1)..self.keyslots.len());

            let key_i: Key = self.keyslots[i].get_key();
            let key_j: Key = self.keyslots[j].get_key();

            match self.keyslots[i].set_key(key_j) {
                Ok(()) => {}
                Err(KeySetError::SingleKeySlot) => {
                    i += 1;
                    continue;
                }
                Err(KeySetError::InvalidKey) => {
                    continue;
                }
            }

            if let Ok(()) = self.keyslots[j].set_key(key_i) {
            } else {
                assert!(
                    self.keyslots[i].set_key(key_i).is_ok(),
                    "Key started in invalid slot"
                );

                continue;
            }

            self.slot_ref.insert(key_i.get_base(), j);
            self.slot_ref.insert(key_j.get_shift(), i);

            i += 1;
        }
    }

    // TODO: A shuffle amount of zero does not produce invalid behavior, so it is not an error. I
    // have a debug assert for now, but that issue needs to be handled somewhere more logical. A
    // good idea would probably be to return an error type for it, and then the caller can handle.
    // Not sure how you wrap that up with the keyslot errors
    pub fn shuffle_some(&mut self, rng: &mut SmallRng, amt: usize) {
        self.evaluated = false;

        debug_assert!(amt > 0);

        let mut swaps: usize = 0;
        while swaps < amt {
            let i: usize = rng.random_range(0..self.keyslots.len());
            let mut j: usize = rng.random_range(0..self.keyslots.len());

            loop {
                if j != i {
                    break;
                }

                j = rng.random_range(0..self.keyslots.len());
            }

            let key_i: Key = self.keyslots[i].get_key();
            let key_j: Key = self.keyslots[j].get_key();

            if self.keyslots[i].set_key(key_j).is_err() {
                continue;
            }

            if let Ok(()) = self.keyslots[j].set_key(key_i) {
            } else {
                assert!(
                    self.keyslots[i].set_key(key_i).is_ok(),
                    "Key started in invalid slot"
                );

                continue;
            }

            self.slot_ref.insert(key_i.get_base(), j);
            self.slot_ref.insert(key_j.get_shift(), i);

            swaps += 1;
        }
    }

    // TODO: The evaluation should be setup in such a manner that this is a private function. This
    // kind of internal state management should not be publicly exposed
    fn clear_last_slot(&mut self) {
        self.last_slot_idx = None;
    }

    // TODO: Something to keep in mind is - There are certain kinds of bad moves that are more
    // quantifiable, and thus easier to overweight the badness of. If the same finger is used
    // twice, you could calculate the distance of the movement, adding that on top of any other
    // demerits. Does this over-punish same-finger usage?
    // Broadly then, what should be done is to think of demerits in terms of classes. If you have a
    // finger demerit, you can apply that to all fingers easily. Same with a hand demerit or a row
    // jump demerit
    // So as an example, all scissors should be punished, and you might punish upward moving
    // scissors more harshly than downward moving scissors (though this feels situational). But
    // scissors involving the pinky and fing finger should not be punished particularly
    // harshly
    // Something else that needs to be considered is the "grain" issue of the left and right hands.
    // The home row of the left hand is not more difficult to hit than the right, but any row
    // jumping is more difficult because the shape is not natural.
    // TODO: This function is too long. Need a way to separate out, but I don't feel like the
    // overall architecture is settled in enough yet to do that
    // PERF: Might be faster to dereference input before sending it here
    fn get_efficiency(&mut self, input: u8) -> f64 {
        let last_slot_checked: Option<&KeySlot> = match self.last_slot_idx {
            Some(idx) => Some(&self.keyslots[idx]),
            None => None,
        };

        if input == b' ' {
            self.last_slot_idx = None;
            // TODO: Space is an interesting key because it gets to if you score for efficiency,
            // speed, hand pain, or some combination of the three. Space is a slow and cumbersome
            // key, but it causes no thumb pain
            // To some extent though, it might not matter, because I'm not sure if there's any
            // particular key combination that's harder to hit space from
            return 1.0;
        }

        if input == b'\n' {
            if let Some(last_slot) = last_slot_checked {
                let last_hand = last_slot.get_hand_info().get_hand();
                // TODO: This is not correct because it's easier to hit another key with your left
                // hand after return than your left
                self.last_slot_idx = None;
                if last_hand == Hand::Left {
                    return 0.9;
                }
            }

            self.last_slot_idx = None;
            return 0.8;
        }

        let slot_idx: &usize = match self.slot_ref.get(&input) {
            Some(slot) => slot,
            None => {
                self.last_slot_idx = None;
                return 0.0;
            } // Not a valid key, don't affect score
        };

        let slot: &KeySlot = &self.keyslots[*slot_idx];
        const DEFAULT_EFFICIENCY: f64 = 1.0;
        let mut efficiency: f64 = DEFAULT_EFFICIENCY;

        let this_row: Row = slot.get_loc_info().get_row();
        // I agree with Dvorak. The top row is easier to hit than the bottom
        if this_row == Row::Above {
            efficiency *= 0.92;
        }
        if this_row == Row::Below {
            efficiency *= 0.84;
        }
        if this_row == Row::Num {
            efficiency *= 0.75;
        }

        // Penalize index finger extensions
        let col: Col = slot.get_loc_info().get_col();
        if col == Col::Five || col == Col::Six {
            efficiency *= 0.9;
        }

        // These extensions are especially bad
        if (col == Col::Five && this_row == Row::Below)
            || (col == Col::Six && this_row == Row::Above)
        {
            efficiency *= 0.9;
        }

        let hand: Hand = slot.get_hand_info().get_hand();
        // Because the keyboard columns slope down-right, this goes against the grain of the left
        // hand, so we penalize it here. But, only slightly because left-handed typists are out
        // there and people have different preferences
        // TODO: This is a bit of a cheat
        if hand == Hand::Left {
            efficiency *= 0.95;
        }

        // The ring and pinky fingers are penalized evenly due to variance in personal preference.
        // Neither the index nor middle finger are preferenced for the same reason
        let this_finger: Finger = slot.get_hand_info().get_finger();
        if this_finger == Finger::Ring || this_finger == Finger::Pinky {
            efficiency *= 0.9;
        }

        if let Some(last_slot) = last_slot_checked {
            let last_row: Row = last_slot.get_loc_info().get_row();
            let last_hand: Hand = last_slot.get_hand_info().get_hand();

            // TODO: This is sloppy, but don't want to over-dial in because at some point the eval
            // function is going to be more fundamentally re-written I'm sure
            // TODO: Need a way to handle these types of conditions
            // Two row jumps are not good
            let last_finger: Finger = last_slot.get_hand_info().get_finger();
            let row_distance: u8 = this_row.get_num().abs_diff(last_row.get_num());
            if hand == last_hand && row_distance > 1 {
                efficiency *= 0.75;

                if row_distance == 3 {
                    efficiency *= 0.75;
                }

                //Scissor
                if this_finger.get_num().abs_diff(last_finger.get_num()) == 1 {
                    efficiency *= 0.75;
                }

                // Big jumps on the left hand are worse
                if hand == Hand::Left {
                    efficiency *= 0.8;
                }
            }

            // Any row jump is not preferred
            if hand == last_hand && last_row != this_row {
                efficiency *= 0.9;
            }

            if hand == last_hand
                && ((this_finger == Finger::Pinky && last_finger == Finger::Ring)
                    || (this_finger == Finger::Ring && last_finger == Finger::Pinky))
            {
                efficiency *= 0.80;
            }

            // Slow, causes pain
            if this_finger == last_finger && hand == last_hand {
                efficiency *= 0.75;
            }

            // let last_col: Col = last_slot.get_col();

            // TODO: This in particular needs refining. In general, rolls are good, but not all
            // rolls are created equal. Need to find general principles that work without
            // overtuning to individual typing preferences
            // For now, we simply go with Dvorak's notion that the typing motion should go in
            if hand == last_hand && last_finger.get_num() < this_finger.get_num() {
                efficiency *= 0.85;
            }

            if hand == last_hand && last_row.get_num() < last_row.get_num() {
                efficiency *= 0.85;
            }
        }

        self.last_slot_idx = Some(*slot_idx);

        return efficiency;
    }

    pub fn eval(&mut self, corpus: &[String]) {
        // TODO: Is there a better way to handle this? There should be some result return you can
        // use to say "I did a new evaluation" or "I have already been evaluated"
        if self.evaluated {
            return;
        }

        // TODO: This is fine for now, but as we add more corpus entries we might run into floating
        // point precision issues. Since we eventually want to weight the parts of the corpus
        // anyway, their efficiencies should be stored individually, weighted, then added
        self.score = 0.0;
        self.clear_last_slot();

        for entry in corpus {
            // TODO: Optimization ideas:
            // - Do this as_bytes rather than chars. Would maybe need to use byte literals but
            // would be faster because of not UTF-8 decoding
            for b in entry.as_bytes() {
                self.score += self.get_efficiency(*b);
            }
        }

        self.evaluated = true;
    }

    pub fn get_score(&self) -> f64 {
        return self.score;
    }

    // TODO: This is fine for drafting but needs a rework for more serious use
    pub fn display_keyboard(&self) {
        let mut number_vec: Vec<char> = Vec::new();
        let mut above_vec: Vec<char> = Vec::new();
        let mut home_vec: Vec<char> = Vec::new();
        let mut below_vec: Vec<char> = Vec::new();

        for slot in &self.keyslots {
            let this_row = slot.get_loc_info().get_row();
            let this_key = slot.get_key().get_base() as char;

            if this_row == Row::Num {
                number_vec.push(this_key);
            } else if this_row == Row::Above {
                above_vec.push(this_key);
            } else if this_row == Row::Home {
                home_vec.push(this_key);
            } else if this_row == Row::Below {
                below_vec.push(this_key);
            }
        }

        println!("{:?}", number_vec);
        println!("{:?}", above_vec);
        println!("{:?}", home_vec);
        println!("{:?}", below_vec);
    }
}

// TODO: Function too long
// TODO: Logically, this is indeed something the keyboard needs to be able to do to itself
pub fn hill_climb(
    rng: &mut SmallRng,
    keyboard: &Keyboard,
    corpus: &[String],
    iter: usize,
) -> Result<Keyboard> {
    let mut decay_factor: f64 = 1.0 - (1.0 / iter as f64);
    // TODO: This should be a hard code
    let clamp_value: f64 = 1.0 - (2.0_f64).powf(-53.0);
    decay_factor = decay_factor.min(clamp_value);
    if keyboard.is_elite {
        decay_factor *= decay_factor.powf(3.0);
    }
    println!("Climb Decay: {}", decay_factor);

    if keyboard.is_elite {
        let r: f64 = rng.random_range(0.0..=1.0);
        if r >= decay_factor {
            println!("Score: {}", keyboard.get_score());
            keyboard.display_keyboard();
            return Ok(keyboard.clone());
        }
    }

    const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 90;

    // TODO: I'm not sure if this is actually better than cloning, though the intention is more
    // explicit
    let mut kb: Keyboard = Keyboard::copy_kb(keyboard);
    let start: f64 = kb.get_score();

    let mut last_improvement: f64 = 0.0;
    let mut avg: f64 = 0.0;
    let mut weighted_avg: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;

    // One indexed for averaging math and display
    for i in 1..=10000 {
        let kb_score: f64 = kb.get_score();

        // Doing steps of one change works best. If you change two keys, the algorithm will find
        // bigger changes less frequently. This causes the decay to continue for about as many
        // iterations as it would if doing only one step, but fewer improvements will be found,
        // causing the improvement at the end of the hill climbing step to be lower
        let mut climb_kb: Keyboard = Keyboard::copy_kb(&kb);
        climb_kb.shuffle_some(rng, 1);
        climb_kb.eval(corpus);
        let climb_kb_score: f64 = climb_kb.get_score();

        let this_change = climb_kb_score - kb_score;
        let this_improvement: f64 = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta = this_improvement - last_improvement;
        last_improvement = this_improvement;
        let weight: f64 = get_weight(delta, kb.is_elite);

        sum_weights *= decay_factor;
        let weighted_avg_for_new: f64 = weighted_avg * sum_weights;
        sum_weights += weight;
        weighted_avg = (weighted_avg_for_new + this_improvement * weight) / sum_weights;

        // TODO: Debug only
        print!(
            "Iter: {} -- Start: {} -- Cur: {} -- Best: {} -- Avg: {} -- Weighted: {}\r",
            i, start, climb_kb_score, kb_score, avg, weighted_avg
        );
        stdout().flush()?;

        if climb_kb_score > kb_score {
            kb = climb_kb;
        }

        // NOTE: An edge case can occur where, if the first improvement is on the first iteration,
        // the weighted average can be smaller than the unweighted due to floating point
        // imprecision
        // We get around this with an iteration minimum, but it does paste over the underlying
        // issue
        // TODO: Is there a better solution?
        let plateauing: bool = weighted_avg < avg && i > 1;
        let not_starting: bool = avg <= 0.0 && i >= MAX_ITER_WITHOUT_IMPROVEMENT;
        if plateauing || not_starting {
            break;
        }
    }

    // TODO: For debugging
    println!();
    if kb.is_elite {
        kb.display_keyboard();
    }

    return Ok(kb);
}

fn get_new_avg(new_value: f64, old_avg: f64, new_count: usize) -> f64 {
    let new_value_for_new_avg: f64 = new_value / (new_count as f64);
    let old_avg_for_new_avg: f64 = old_avg * ((new_count as f64 - 1.0) / new_count as f64);

    return new_value_for_new_avg + old_avg_for_new_avg;
}

fn get_weight(delta: f64, is_old: bool) -> f64 {
    const K: f64 = 0.01;

    if delta <= 0.0 {
        return 1.0;
    }

    if is_old {
        // return 1.0 + K * delta.ln(); // Less scaling for positive values
        return 1.0 + K * delta.powf(0.0001); // Even less scaling for positive values
    }

    return 1.0 + K * delta.sqrt();
}

// FUTURE: It would be better if this could be done with iteration, but I'm not sure how to do that
// in a way that isn't more contrived than building it manually
fn get_qwerty_slots() -> Vec<KeySlot> {
    let mut slots: Vec<KeySlot> = Vec::new();

    {
        let key: Key = Key::from_template(KeyTemplate::One);
        let row: Row = Row::Num;
        let col: Col = Col::One;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Two);
        let row: Row = Row::Num;
        let col: Col = Col::Two;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Three);
        let row: Row = Row::Num;
        let col: Col = Col::Three;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Four);
        let row: Row = Row::Num;
        let col: Col = Col::Four;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Five);
        let row: Row = Row::Num;
        let col: Col = Col::Five;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Six);
        let row: Row = Row::Num;
        let col: Col = Col::Six;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Seven);
        let row: Row = Row::Num;
        let col: Col = Col::Seven;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Eight);
        let row: Row = Row::Num;
        let col: Col = Col::Eight;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Nine);
        let row: Row = Row::Num;
        let col: Col = Col::Nine;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Zero);
        let row: Row = Row::Num;
        let col: Col = Col::Ten;

        slots.push(build_slot(key, row, col));
    }

    {
        let key: Key = Key::from_template(KeyTemplate::Q);
        let row: Row = Row::Above;
        let col: Col = Col::One;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::W);
        let row: Row = Row::Above;
        let col: Col = Col::Two;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::E);
        let row: Row = Row::Above;
        let col: Col = Col::Three;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::R);
        let row: Row = Row::Above;
        let col: Col = Col::Four;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::T);
        let row: Row = Row::Above;
        let col: Col = Col::Five;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Y);
        let row: Row = Row::Above;
        let col: Col = Col::Six;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::U);
        let row: Row = Row::Above;
        let col: Col = Col::Seven;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::I);
        let row: Row = Row::Above;
        let col: Col = Col::Eight;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::O);
        let row: Row = Row::Above;
        let col: Col = Col::Nine;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::P);
        let row: Row = Row::Above;
        let col: Col = Col::Ten;

        slots.push(build_slot(key, row, col));
    }

    {
        let key: Key = Key::from_template(KeyTemplate::A);
        let row: Row = Row::Home;
        let col: Col = Col::One;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::S);
        let row: Row = Row::Home;
        let col: Col = Col::Two;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::D);
        let row: Row = Row::Home;
        let col: Col = Col::Three;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::F);
        let row: Row = Row::Home;
        let col: Col = Col::Four;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::G);
        let row: Row = Row::Home;
        let col: Col = Col::Five;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::H);
        let row: Row = Row::Home;
        let col: Col = Col::Six;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::J);
        let row: Row = Row::Home;
        let col: Col = Col::Seven;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::K);
        let row: Row = Row::Home;
        let col: Col = Col::Eight;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::L);
        let row: Row = Row::Home;
        let col: Col = Col::Nine;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::SemiColon);
        let row: Row = Row::Home;
        let col: Col = Col::Ten;

        slots.push(build_slot(key, row, col));
    }

    {
        let key: Key = Key::from_template(KeyTemplate::Z);
        let row: Row = Row::Below;
        let col: Col = Col::One;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::X);
        let row: Row = Row::Below;
        let col: Col = Col::Two;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::C);
        let row: Row = Row::Below;
        let col: Col = Col::Three;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::V);
        let row: Row = Row::Below;
        let col: Col = Col::Four;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::B);
        let row: Row = Row::Below;
        let col: Col = Col::Five;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::N);
        let row: Row = Row::Below;
        let col: Col = Col::Six;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::M);
        let row: Row = Row::Below;
        let col: Col = Col::Seven;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Comma);
        let row: Row = Row::Below;
        let col: Col = Col::Eight;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::Period);
        let row: Row = Row::Below;
        let col: Col = Col::Nine;

        slots.push(build_slot(key, row, col));
    }
    {
        let key: Key = Key::from_template(KeyTemplate::ForwardSlash);
        let row: Row = Row::Below;
        let col: Col = Col::Ten;

        slots.push(build_slot(key, row, col));
    }

    return slots;
}

fn build_slot(key: Key, row: Row, col: Col) -> KeySlot {
    let loc_info: LocInfo = LocInfo::from_row_col(row, col);

    let hand: Hand = col.get_hand();
    let finger: Finger = col.get_finger();
    let hand_info: HandInfo = HandInfo::from_hand_finger(hand, finger);

    let restrictions = get_key_restrictions(loc_info);
    let key_list: KeyList = KeyList::from_vec(restrictions.0, restrictions.1);

    return KeySlot::new(key, loc_info, hand_info, key_list);
}

// ****NOTE**** : Both qwerty and dvorak initialize with keys that would be disallowed in a final
// build. Need to handle
fn get_key_restrictions(loc_info: LocInfo) -> (Vec<Key>, ListType) {
    let this_loc: (Row, Col) = (loc_info.get_row(), loc_info.get_col());

    return match this_loc {
        (Row::Num, Col::One) => (vec![Key::from_template(KeyTemplate::One)], ListType::Allow),
        (Row::Num, Col::Two) => (vec![Key::from_template(KeyTemplate::Two)], ListType::Allow),
        (Row::Num, Col::Three) => (
            vec![Key::from_template(KeyTemplate::Three)],
            ListType::Allow,
        ),
        (Row::Num, Col::Four) => (vec![Key::from_template(KeyTemplate::Four)], ListType::Allow),
        (Row::Num, Col::Five) => (vec![Key::from_template(KeyTemplate::Five)], ListType::Allow),
        (Row::Num, Col::Six) => (vec![Key::from_template(KeyTemplate::Six)], ListType::Allow),
        (Row::Num, Col::Seven) => (
            vec![Key::from_template(KeyTemplate::Seven)],
            ListType::Allow,
        ),
        (Row::Num, Col::Eight) => (
            vec![Key::from_template(KeyTemplate::Eight)],
            ListType::Allow,
        ),
        (Row::Num, Col::Nine) => (vec![Key::from_template(KeyTemplate::Nine)], ListType::Allow),
        (Row::Num, Col::Ten) => (vec![Key::from_template(KeyTemplate::Zero)], ListType::Allow),
        _ => (Vec::new(), ListType::Deny),
    };
}
