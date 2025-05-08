use std::{
    io::{Write as _, stdout},
    ptr,
};

use {
    anyhow::Result,
    rand::{Rng as _, rngs::SmallRng},
    strum::IntoEnumIterator,
};

use crate::{key::Key, key_template::KeyTemplate};

#[derive(Clone)]
pub struct Keyboard {
    kb_vec: Vec<Vec<Key>>,
    slot_ascii: Vec<Option<(usize, usize)>>,
    last_key_idx: Option<usize>,
    generation: usize,
    id: usize,
    lineage: String,
    evaluated: bool,
    score: f64,
    is_elite: bool,
}

// TODO: The meta issue with this struct is how much it relies on the KeyTemplate enum. It's
// compile time, which is good, but it's exterior, which is bad
// TODO: Need to rebuild the qwerty creation and add dvorak
impl Keyboard {
    // TODO: The memory safety case here is weak
    pub fn create_origin(id_in: usize) -> Self {
        const NUM_ROW_CAPACITY: usize = 12;
        const TOP_ROW_CAPACITY: usize = 12;
        const HOME_ROW_CAPACITY: usize = 11;
        const BOT_ROW_CAPACITY: usize = 10;

        let mut kb_vec: Vec<Vec<Key>> = vec![
            Vec::with_capacity(NUM_ROW_CAPACITY),
            Vec::with_capacity(TOP_ROW_CAPACITY),
            Vec::with_capacity(HOME_ROW_CAPACITY),
            Vec::with_capacity(BOT_ROW_CAPACITY),
        ];

        let ptr: *mut Vec<Key> = kb_vec.as_mut_ptr();

        for template in KeyTemplate::iter() {
            let location: (usize, usize) = template.get_starting_location();
            let this_key = Key::from_template(template);
            println!("{}, {}", location.0, location.1);

            // SAFETY: The indexes to write come from the KeyTemplate structs, with methods built
            // at compile time
            unsafe {
                let row_ptr: *mut Vec<Key> = ptr.add(location.0);
                let inner_vec: &mut Vec<Key> = &mut *row_ptr;
                let elem_ptr: *mut Key = inner_vec.as_mut_ptr();
                elem_ptr.add(location.1).write(this_key);
            }
        }

        // SAFETY: Compile time values
        unsafe {
            kb_vec[0].set_len(NUM_ROW_CAPACITY);
            kb_vec[1].set_len(TOP_ROW_CAPACITY);
            kb_vec[2].set_len(HOME_ROW_CAPACITY);
            kb_vec[3].set_len(BOT_ROW_CAPACITY);
        }

        let mut slot_ascii: Vec<Option<(usize, usize)>> = vec![None; 128];
        for i in 0..kb_vec.len() {
            for j in 0..kb_vec[i].len() {
                slot_ascii[kb_vec[i][j].get_base() as usize] = Some((i, j));
                slot_ascii[kb_vec[i][j].get_shift() as usize] = Some((i, j));
            }
        }

        let generation = 0;
        let id: usize = id_in;
        let lineage: String = format!("{}.{}", generation, id);

        return Self {
            kb_vec,
            slot_ascii,
            last_key_idx: None,
            generation,
            id,
            lineage,
            evaluated: false,
            score: 0.0,
            is_elite: false,
        };
    }

    pub fn mutate_from(kb: &Keyboard, gen_input: usize, id_in: usize) -> Self {
        let kb_vec: Vec<Vec<Key>> = kb.get_kb_vec().to_vec();
        let slot_ascii: Vec<Option<(usize, usize)>> = kb.get_slot_ascii().to_vec();
        let last_slot_idx: Option<usize> = None;

        let generation: usize = gen_input;
        let id: usize = id_in;
        let lineage: String = format!("{}-{}.{}", kb.get_lineage(), generation, id);

        let evaluated: bool = kb.get_eval_status();
        let score: f64 = kb.get_score();
        let is_elite: bool = kb.is_elite();

        return Self {
            kb_vec,
            slot_ascii,
            last_key_idx: last_slot_idx,
            generation,
            id,
            lineage,
            evaluated,
            score,
            is_elite,
        };
    }

    // NOTE: This function assumes the keys are properly setup such that there is always at least
    // one valid option to shuffle to
    pub fn shuffle(&mut self, rng: &mut SmallRng, amt: usize) {
        self.evaluated = false;

        let mut shuffled: usize = 0;
        while shuffled < amt {
            // The top row and the side symbol keys are purposefully avoided
            let row = rng.random_range(1..4);
            let col = rng.random_range(0..10);
            if self.kb_vec[row][col].is_static() {
                continue;
            }

            self.kb_vec[row][col].shuffle_valid_locations(rng);
            let cnt_valid = self.kb_vec[row][col].get_cnt_valid_locations();

            for i in 0..cnt_valid {
                let test = self.kb_vec[row][col].get_valid_location_at_idx(i);
                if test.0 == row && test.1 == col {
                    continue;
                }

                if self.kb_vec[test.0][test.1]
                    .get_valid_locations()
                    .contains(&(row, col))
                {
                    unsafe {
                        let ptr = self.kb_vec.as_mut_ptr();
                        let row1_ptr = ptr.add(row);
                        let row2_ptr = ptr.add(test.0);
                        let elem1_ptr = (*row1_ptr).as_mut_ptr().add(col);
                        let elem2_ptr = (*row2_ptr).as_mut_ptr().add(test.1);

                        ptr::swap(elem1_ptr, elem2_ptr);
                    }

                    self.slot_ascii[self.kb_vec[row][col].get_base() as usize] = Some((row, col));
                    self.slot_ascii[self.kb_vec[row][col].get_shift() as usize] = Some((row, col));
                    self.slot_ascii[self.kb_vec[test.0][test.1].get_base() as usize] =
                        Some((test.0, test.1));
                    self.slot_ascii[self.kb_vec[test.0][test.1].get_shift() as usize] =
                        Some((test.0, test.1));

                    shuffled += 1;
                    break;
                }
            }
        }
    }

    // TODO: Fix unused variable
    // TODO: Unsure of how to handle space and return
    // TODO: How to factor out...
    fn get_efficiency(&mut self, _input: u8) -> f64 {
        const DEFAULT_EFFICIENCY: f64 = 1.0;

        // let key_idx: usize = if let Some(&Some(slot)) = self.slot_ascii.get(input as usize) {
        //     slot
        // } else {
        //     self.last_key_idx = None;
        //     return 0.0;
        // };

        // let this_key: &UpdatedKey = &self.keys[key_idx];
        // let mut efficiency: f64 = DEFAULT_EFFICIENCY;
        let efficiency: f64 = DEFAULT_EFFICIENCY;

        // let last_key: &UpdatedKey = if let Some(key) = self
        //     .last_key_idx
        //     .and_then(|slot| return self.keys.get(slot))
        // {
        //     key
        // } else {
        //     self.last_key_idx = Some(key_idx);
        //     return efficiency;
        // };
        //
        // self.last_key_idx = Some(key_idx);

        return efficiency;
    }

    pub fn eval(&mut self, corpus: &[String]) {
        if self.evaluated {
            return;
        }

        self.score = 0.0;
        self.last_key_idx = None;

        for entry in corpus {
            for b in entry.as_bytes() {
                self.score += self.get_efficiency(*b);
            }
        }

        self.evaluated = true;
    }

    // TODO: Better, but will still need to be redone for final display
    pub fn display_keyboard(&self) {
        for row in &self.kb_vec {
            let mut chars: Vec<char> = Vec::new();
            for element in row {
                let char = element.get_base() as char;
                chars.push(char);
            }
            println!("{:?}", chars);
        }
    }

    // Pieces for mutation
    fn get_kb_vec(&self) -> &[Vec<Key>] {
        return &self.kb_vec;
    }

    fn get_slot_ascii(&self) -> &[Option<(usize, usize)>] {
        return &self.slot_ascii;
    }

    // Info display
    pub fn get_lineage(&self) -> &str {
        return &self.lineage;
    }

    pub fn get_eval_status(&self) -> bool {
        return self.evaluated;
    }

    pub fn get_score(&self) -> f64 {
        return self.score;
    }

    pub fn get_generation(&self) -> usize {
        return self.generation;
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    // For population management
    pub fn get_vec_ref(&self) -> &[Vec<Key>] {
        return &self.kb_vec;
    }

    pub fn is_elite(&self) -> bool {
        return self.is_elite;
    }

    pub fn set_elite(&mut self) {
        self.is_elite = true;
    }

    pub fn unset_elite(&mut self) {
        self.is_elite = false;
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
    let mut kb: Keyboard = keyboard.clone();
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
        let mut climb_kb: Keyboard = kb.clone();
        climb_kb.shuffle(rng, 1);
        climb_kb.eval(corpus);
        let climb_kb_score: f64 = climb_kb.get_score();

        let this_change = climb_kb_score - kb_score;
        let this_improvement: f64 = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta: f64 = this_improvement - last_improvement;
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

// TODO: How do make the division work with f64. Do we try to fix the truncating behavior?
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

// impl Keyboard {
//     fn get_efficiency(&mut self, input: u8) -> f64 {
//         const DEFAULT_EFFICIENCY: f64 = 1.0;
//
//         let slot_idx: usize = if let Some(&Some(slot)) = self.slot_ascii.get(input as usize) {
//             slot
//         } else {
//             self.last_slot_idx = None;
//             return 0.0;
//         };
//
//         let this_slot: &KeySlot = &self.keyslots[slot_idx];
//         let mut efficiency: f64 = DEFAULT_EFFICIENCY;
//
//         let this_row: Row = this_slot.get_row();
//         if this_row == Row::Above {
//             efficiency *= 0.75;
//         } else if this_row == Row::Below {
//             efficiency *= 0.50;
//         } else if this_row == Row::Num {
//             efficiency *= 0.25;
//         }
//
//         let this_hand: Hand = this_slot.get_hand();
//         // The down/right slope of each column does not agree with the left hand
//         if this_hand == Hand::Left {
//             efficiency *= 0.75;
//         }
//
//         let this_col: Col = this_slot.get_col();
//         if this_col == Col::Five {
//             efficiency *= 0.75;
//
//             if this_row == Row::Below {
//                 efficiency *= 0.50;
//             }
//         }
//
//         if this_col == Col::Six {
//             efficiency *= 0.75;
//
//             if this_row == Row::Above {
//                 efficiency *= 0.50;
//             }
//         }
//
//         let this_finger: Finger = this_slot.get_finger();
//         if this_finger == Finger::Ring {
//             efficiency *= 0.75;
//         } else if this_finger == Finger::Pinky {
//             efficiency *= 0.25;
//         }
//
//         if this_finger == Finger::Middle && this_row == Row::Above {
//             efficiency *= 1.25;
//         }
//
//         if this_col == Col::Eleven || this_col == Col::Twelve {
//             efficiency *= 0.75;
//
//             if this_row == Row::Above || this_row == Row::Num {
//                 efficiency *= 0.25;
//             }
//         }
//
//         let last_slot: &KeySlot = if let Some(slot) = self
//             .last_slot_idx
//             .and_then(|slot| return self.keyslots.get(slot))
//         {
//             slot
//         } else {
//             self.last_slot_idx = Some(slot_idx);
//             return efficiency;
//         };
//
//         let last_row = last_slot.get_row();
//         let last_col = last_slot.get_col();
//         let last_hand = last_slot.get_hand();
//         let last_finger = last_slot.get_finger();
//         let row_distance = this_row.get_num().abs_diff(last_row.get_num());
//
//         if last_hand == this_hand {
//             efficiency *= 0.25;
//
//             if this_finger == last_finger {
//                 efficiency *= 0.50;
//
//                 if row_distance == 1 {
//                     efficiency *= 0.75;
//                 } else if row_distance == 2 {
//                     efficiency *= 0.50;
//                 } else if row_distance == 3 {
//                     efficiency *= 0.25;
//                 }
//             }
//
//             if this_finger != last_finger {
//                 let finger_distance = this_finger.get_num().abs_diff(last_finger.get_num());
//
//                 if finger_distance == 1 {
//                     if row_distance == 2 {
//                         efficiency *= 0.5;
//                     } else if row_distance == 3 {
//                         efficiency *= 0.75;
//                     }
//                 }
//
//                 if this_row != last_row {
//                     efficiency *= 0.50;
//                 }
//             }
//         }
//
//         if last_hand == Hand::Right
//             && this_hand == Hand::Right
//             && (last_col == Col::Eleven || last_col == Col::Twelve)
//         {
//             efficiency *= 0.75;
//
//             if last_row == Row::Above || last_row == Row::Num {
//                 efficiency *= 0.25;
//             }
//         }
//
//         self.last_slot_idx = Some(slot_idx);
//
//         return efficiency;
//     }
//
//     // TODO: Is there a better way to handle this? There should be some result return you can
//     // use to say "I did a new evaluation" or "I have already been evaluated"
//     // You could maybe make this return a corpus err and include already evaluated
//     pub fn eval(&mut self, corpus: &[String]) {
//         if self.evaluated {
//             return;
//         }
//
//         self.score = 0.0;
//         self.clear_last_slot();
//
//         for entry in corpus {
//             for b in entry.as_bytes() {
//                 self.score += self.get_efficiency(*b);
//             }
//         }
//
//         self.evaluated = true;
//     }
//
//     pub fn get_score(&self) -> f64 {
//         return self.score;
//     }
//
//     // TODO: This is fine for drafting but needs a rework for more serious use
//     pub fn display_keyboard(&self) {
//         let mut number_vec: Vec<char> = Vec::new();
//         let mut above_vec: Vec<char> = Vec::new();
//         let mut home_vec: Vec<char> = Vec::new();
//         let mut below_vec: Vec<char> = Vec::new();
//
//         for slot in &self.keyslots {
//             let this_row = slot.get_row();
//             let this_key = slot.get_key().get_base() as char;
//
//             if this_row == Row::Num {
//                 number_vec.push(this_key);
//             } else if this_row == Row::Above {
//                 above_vec.push(this_key);
//             } else if this_row == Row::Home {
//                 home_vec.push(this_key);
//             } else if this_row == Row::Below {
//                 below_vec.push(this_key);
//             }
//         }
//
//         println!("{:?}", number_vec);
//         println!("{:?}", above_vec);
//         println!("{:?}", home_vec);
//         println!("{:?}", below_vec);
//     }
// }
