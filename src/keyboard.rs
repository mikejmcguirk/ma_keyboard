extern crate alloc;

use alloc::collections::BTreeMap;

use rand::{Rng as _, rngs::SmallRng, seq::SliceRandom as _};

use crate::{
    kb_helper_consts,
    kb_helpers::{
        check_key_no_hist, check_spaces, compare_keys, get_hand, get_key_locations,
        get_single_key_mult, place_keys,
    },
};

// TODO: Some of this stuff should be removed as we factor out the scoring
kb_helper_consts!();

pub enum KeyCompare {
    Mult(f64),
    Mismatch,
}

// FUTURE: The 2D Vec, even though it currently works, is brittle. I think, once we start adding
// the swap table, we'll see the precise limitations and what a change needs to accomplish
#[derive(Clone)]
pub struct Keyboard {
    kb_vec: Vec<Vec<(u8, u8)>>,
    valid: BTreeMap<(u8, u8), Vec<(usize, usize)>>,
    slot_ascii: Vec<Option<(usize, usize)>>,
    last_key_idx: Option<(usize, usize)>,
    prev_key_idx: Option<(usize, usize)>,
    generation: usize,
    id: usize,
    evaluated: bool,
    score: f64,
    left_uses: f64,
    right_uses: f64,
    is_elite: bool,
    pos_iter: usize,
}

impl Keyboard {
    /// # Panics
    /// Will panic if compile time data is incorrect
    // slot_ascii has a compile time length of 128. Every char in valid_locations is checked with
    // an assertion in get_key_locations that it is 127 or less
    #[expect(clippy::indexing_slicing)]
    pub fn create_origin(id_in: usize) -> Self {
        let mut kb_vec = vec![
            vec![SPACE; NUM_ROW_CNT],
            vec![SPACE; TOP_ROW_CNT],
            vec![SPACE; HOME_ROW_CNT],
            vec![SPACE; BOT_ROW_CNT],
        ];

        let valid_vec = get_key_locations();
        let kb_vec_cnt: usize = kb_vec.iter().map(|v| return v.len()).sum();
        assert_eq!(
            kb_vec_cnt,
            valid_vec.len(),
            "The amount of keys to assign ({}) is not equal to the number of keyboard slots ({})",
            kb_vec_cnt,
            valid_vec.len()
        );

        place_keys(&mut kb_vec, &valid_vec, 0);
        let space_keys = check_spaces(&kb_vec);
        assert!(
            space_keys.is_empty(),
            "The following kb_vec values contain spaces: {:?}",
            space_keys
        );

        let mut valid_bt: BTreeMap<(u8, u8), Vec<(usize, usize)>> = BTreeMap::new();
        for loc in &valid_vec {
            valid_bt.insert(loc.0, loc.1.clone());
        }

        let mut slot_ascii = vec![None; ASCII_CNT];
        kb_vec.iter().enumerate().for_each(|(i, row)| {
            row.iter().enumerate().for_each(|(j, &(base, shift))| {
                slot_ascii[usize::from(base)] = Some((i, j));
                slot_ascii[usize::from(shift)] = Some((i, j));
            });
        });

        return Self {
            kb_vec,
            valid: valid_bt,
            slot_ascii,
            last_key_idx: None,
            prev_key_idx: None,
            generation: 0,
            id: id_in,
            evaluated: false,
            score: 0.0,
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
            pos_iter: 0,
        };
    }

    pub fn mutate_from(kb: &Keyboard, gen_input: usize, id_in: usize) -> Self {
        return Self {
            kb_vec: kb.kb_vec.clone(),
            valid: kb.valid.clone(),
            slot_ascii: kb.slot_ascii.clone(),
            last_key_idx: None,
            prev_key_idx: None,
            generation: gen_input,
            id: id_in,
            evaluated: kb.evaluated,
            score: kb.get_score(),
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
            pos_iter: kb.pos_iter,
        };
    }

    // FUTURE: If we start giving more keys no shuffle options (like "<" and ">"), the value of the
    // iterative approach increases, since we can filter out keys where valid_locations == 1
    // FUTURE: It would be better if the shuffle amounts were read from a config
    // NOTE: The shuffle constants make sure that the number row and the symbol keys to the right
    // of the right pinky are ignored
    /// # Panics
    /// Panics if no valid locations are found for a selected key
    /// This function expects the data setup in `create_origin` to be correct
    /// Will also panic if the shuffle amount is too high, since these are compile time values
    #[expect(clippy::indexing_slicing)]
    pub fn shuffle(&mut self, rng: &mut SmallRng, amt: usize) {
        const MIN_ROW: usize = 1;
        const MAX_ROW: usize = 4;
        const MIN_COL: usize = 0;
        const MAX_COL: usize = 10;

        debug_assert!(amt <= 100, "{amt} is too many shuffles");
        self.evaluated = false;

        for _ in 0..amt {
            let this_row = rng.random_range(MIN_ROW..MAX_ROW);
            let this_col = rng.random_range(MIN_COL..MAX_COL);
            let this_key = self.kb_vec[this_row][this_col];
            if self.valid[&this_key].len() == 1 {
                continue;
            }

            if let Some(vec) = self.valid.get_mut(&this_key) {
                vec.shuffle(rng);
            } else {
                panic!("Valid locations not found for key {:?}", this_key);
            }

            for (i, _) in self.valid[&this_key].iter().enumerate() {
                let that_row = self.valid[&this_key][i].0;
                let that_col = self.valid[&this_key][i].1;
                let that_key = self.kb_vec[that_row][that_col];
                if self.valid[&that_key].len() == 1
                    || !self.valid[&that_key].contains(&(this_row, this_col))
                    || this_key == that_key
                {
                    continue;
                }

                self.kb_vec[this_row][this_col] = that_key;
                self.kb_vec[that_row][that_col] = this_key;

                self.slot_ascii[usize::from(that_key.0)] = Some((this_row, this_col));
                self.slot_ascii[usize::from(that_key.1)] = Some((this_row, this_col));
                self.slot_ascii[usize::from(this_key.0)] = Some((that_row, that_col));
                self.slot_ascii[usize::from(this_key.1)] = Some((that_row, that_col));

                break;
            }

            assert_ne!(
                self.kb_vec[this_row][this_col], this_key,
                "Key {:?} at {},{} not changed!",
                this_key, this_row, this_col
            );
        }
    }

    // NOTE: A single major efficiency penalty at any point in the algorithm can cause the entire
    // layout to change. Be careful over-indexing for any particular factor
    fn get_efficiency(&mut self, this_key: (usize, usize)) -> f64 {
        let mut eff = BASE_EFF;

        let this_col = this_key.1;
        let this_hand = get_hand(this_col);
        if this_hand == RIGHT {
            self.right_uses += 1.0_f64;
        } else if this_hand == LEFT {
            self.left_uses += 1.0_f64;
        }

        eff *= get_single_key_mult(this_key);

        let last_compare: Option<KeyCompare> = self
            .last_key_idx
            .map(|last_key| return compare_keys(this_key, last_key, true));
        if let Some(key_compare) = last_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::Mismatch => {}
            }
        }

        let prev_compare: Option<KeyCompare> = self
            .prev_key_idx
            .map(|prev_key| return compare_keys(this_key, prev_key, false));
        if let Some(key_compare) = prev_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::Mismatch => {}
            }
        }

        eff *= check_key_no_hist(this_key);

        return eff;
    }

    pub fn eval(&mut self, corpus: &[String]) {
        if self.evaluated {
            return;
        }

        self.score = 0.0_f64;
        self.last_key_idx = None;
        self.prev_key_idx = None;
        self.left_uses = 0.0_f64;
        self.right_uses = 0.0_f64;

        for entry in corpus {
            for b in entry.as_bytes() {
                let this_key: (usize, usize) =
                    if let Some(&Some(key)) = self.slot_ascii.get(usize::from(*b)) {
                        key
                    } else {
                        self.prev_key_idx = self.last_key_idx;
                        self.last_key_idx = None;
                        continue;
                    };

                self.score += self.get_efficiency(this_key);

                self.prev_key_idx = self.last_key_idx;
                self.last_key_idx = Some(this_key);
            }
        }

        if self.left_uses < self.right_uses {
            self.score *= self.left_uses / self.right_uses;
        } else {
            self.score *= self.right_uses / self.left_uses;
        }

        self.evaluated = true;
    }

    // TODO: Very inefficient
    pub fn get_display_chars(&self) -> Vec<Vec<char>> {
        let mut display_chars: Vec<Vec<char>> = Vec::new();

        for row in &self.kb_vec {
            let mut chars: Vec<char> = Vec::new();
            for element in row {
                let char = char::from(element.0);
                chars.push(char);
            }

            display_chars.push(chars);
        }

        return display_chars;
    }

    // Info display
    pub fn get_score(&self) -> f64 {
        return self.score;
    }

    pub fn get_generation(&self) -> usize {
        return self.generation;
    }

    pub fn get_id(&self) -> usize {
        return self.id;
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

    pub fn get_pos_iter(&self) -> usize {
        return self.pos_iter;
    }

    pub fn add_pos_iter(&mut self) {
        if let Some(iter) = self.pos_iter.checked_add(1) {
            self.pos_iter = iter;
        } else {
            self.pos_iter = 0;
        }
    }
}
