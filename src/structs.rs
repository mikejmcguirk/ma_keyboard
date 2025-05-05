use std::collections::HashMap;

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, rngs::SmallRng},
    strum::IntoEnumIterator as _,
};

use crate::{
    constants::KEY_TUPLES,
    enums::{Col, Finger, Hand, Row},
    kb_components::{Key, KeySlot},
};

// TODO: This should just be keyboard.rs
#[derive(Clone)]
// NOTE: Don't store RNG in the keyboard. Otherwise, if a keyboard is cloned then discarded, the
// original will not have an updated rng state
pub struct Keyboard {
    keyslots: Vec<KeySlot>,
    // Between the base and shift layer, there are enough possible keypresses to justify this
    slot_ref: HashMap<char, usize>,
    last_slot: Option<KeySlot>,
    generation: usize,
    id: usize,
    lineage: String,
    evaluated: bool,
    score: f64,
    is_elite: bool,
}

impl Keyboard {
    // TODO: I'm not actually sure if new() needs to be run more than once. Even for creating the
    // initial keyboards, it's probably faster to just run clone(). And then for subsequent
    // keyboards, we know we are cloning pre-existing ones. So I think any performance optimization
    // here at the expense of safety and clarity might be an unnecessary flex
    // PERF: This can be optimized by pre-allocating keyslots and unsafely writing to it
    // TODO: At some point this logic will need to handle keys that are not totally randomized. As
    // much of this logic as possible should be tied to the enums. The key though is it needs to
    // flow intuitively. Right now, col.get_finger() intuitively makes sense because we know each
    // keyboard column has a finger mapped to it. You don't really need to jump to definition to
    // understand it
    // TODO: Too early right now, but the slot logic in new should be common with set_slot. The
    // problem at the moment is that we are pushing into the slot Vec rather than setting indexes,
    // which makes the logic incompatible. You could allocate the Vec and write unsafely, but is
    // that less contrived than just repeating some code? Maybe when you're getting super deep into
    // polishing, but not at this stage
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
    pub fn make_origin(id: usize) -> Result<Self> {
        let mut keyslots: Vec<KeySlot> = Vec::new();
        let mut kt_idx: usize = 0;

        // TODO: Do some kind of checked index access for key_tuple_idx
        // TODO: Add a check or debug_assert that key_tuple_idx matches the len of
        // KEY_TUPLES. We need the number of keys to match exactly the amount of slots to fill
        // Add documentation for this behavior as well, since it corrolates a couple different
        // pieces of code
        for row in Row::iter() {
            for col in Col::iter() {
                let Some(key_tuple): Option<&(char, char)> = KEY_TUPLES.get(kt_idx) else {
                    return Err(anyhow!("Out of bounds read from KEY_TUPLES"));
                };

                kt_idx += 1;
                let key: Key = Key::new(*key_tuple)?;
                let hand: Hand = col.get_hand();
                let finger: Finger = col.get_finger();

                let slot: KeySlot = KeySlot::new(key, row, col, hand, finger)?;
                keyslots.push(slot);
            }
        }

        let mut slot_ref: HashMap<char, usize> = HashMap::new();
        for i in 0..keyslots.len() {
            let key: Key = keyslots[i].get_key();

            slot_ref.insert(key.get_base(), i);
            slot_ref.insert(key.get_shift(), i);
        }

        let lineage: String = format!("0.{}", id);

        return Ok(Keyboard {
            keyslots,
            slot_ref,
            last_slot: None,
            generation: 0,
            id,
            lineage,
            evaluated: false,
            score: 0.0,
            is_elite: false,
        });
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
    pub fn mutate_kb(kb: &Keyboard, gen_input: usize, id: usize) -> Self {
        let keyslots: Vec<KeySlot> = kb.get_keyslots().to_vec();
        let slot_ref: HashMap<char, usize> = kb.get_slot_ref().clone();
        let last_slot: Option<KeySlot> = None;
        let generation: usize = gen_input;
        let lineage: String = format!("{}-{}.{}", kb.get_lineage(), gen_input, id);

        let evaluated: bool = kb.get_eval_status();
        let score: f64 = kb.get_score();

        return Self {
            keyslots,
            slot_ref,
            last_slot,
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
        let slot_ref: HashMap<char, usize> = kb.get_slot_ref().clone();
        let last_slot: Option<KeySlot> = None;
        let generation: usize = kb.get_generation();
        let id: usize = kb.get_id();
        let lineage: String = kb.get_lineage();

        let evaluated: bool = kb.get_eval_status();
        let score: f64 = kb.get_score();
        let is_elite: bool = kb.is_elite();

        return Self {
            keyslots,
            slot_ref,
            last_slot,
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

    pub fn is_elite(&self) -> bool {
        return self.is_elite;
    }

    pub fn set_elite(&mut self) {
        self.is_elite = true;
    }

    pub fn unset_elite(&mut self) {
        self.is_elite = false;
    }

    // TODO: Clone bad
    pub fn get_lineage(&self) -> String {
        return self.lineage.clone();
    }

    pub fn get_keyslots(&self) -> &[KeySlot] {
        return &self.keyslots;
    }

    pub fn get_slot_ref(&self) -> &HashMap<char, usize> {
        return &self.slot_ref;
    }

    pub fn get_eval_status(&self) -> bool {
        return self.evaluated;
    }

    fn set_slot(&mut self, idx: usize, key: Key) -> Result<()> {
        if idx > self.keyslots.len() {
            return Err(anyhow!("Invalid keyslot index"));
        }

        self.keyslots[idx].set_key(key)?;
        self.slot_ref.insert(key.get_base(), idx);
        self.slot_ref.insert(key.get_shift(), idx);

        return Ok(());
    }

    // TODO: The fact this can error is really unintuitive
    pub fn shuffle_all(&mut self, rng: &mut SmallRng) -> Result<()> {
        self.evaluated = false;

        for i in 0..self.keyslots.len() {
            let j: usize = rng.random_range(i..self.keyslots.len());

            let key_i: Key = self.keyslots[i].get_key();
            self.set_slot(i, self.keyslots[j].get_key())?;
            self.set_slot(j, key_i)?;
        }

        return Ok(());
    }

    pub fn shuffle_some(&mut self, rng: &mut SmallRng, amt: usize) -> Result<()> {
        self.evaluated = false;

        if amt > self.keyslots.len() {
            return Err(anyhow!("Amount is greater than valid keys"));
        }

        if amt == 0 {
            return Err(anyhow!("Shuffle amount at shuffle_some is zero"));
        }

        // TODO: I removed any checks from this code because it's going to be re-written anyway.
        // The biggest issue is any swapping rules that are put in. The other issue is, how much do
        // we want to allow for "wrong" swaps. Swapping a > b, b > c, c > a is fine. But do we want
        // to allow non-swaps? Is that acceptable randomness or does that miss the point of a
        // particular swapping step?
        for _ in 0..amt {
            let i: usize = rng.random_range(0..self.keyslots.len());
            let j: usize = rng.random_range(0..self.keyslots.len());

            let key_i: Key = self.keyslots[i].get_key();
            self.set_slot(i, self.keyslots[j].get_key())?;
            self.set_slot(j, key_i)?;
        }

        return Ok(());
    }

    // TODO: The evaluation should be setup in such a manner that this is a private function. This
    // kind of internal state management should not be publicly exposed
    fn clear_last_slot(&mut self) {
        self.last_slot = None;
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
    fn get_efficiency(&mut self, input: char) -> f64 {
        if input == ' ' {
            self.last_slot = None;
            // TODO: Space is an interesting key because it gets to if you score for efficiency,
            // speed, hand pain, or some combination of the three. Space is a slow and cumbersome
            // key, but it causes no thumb pain
            // To some extent though, it might not matter, because I'm not sure if there's any
            // particular key combination that's harder to hit space from
            return 1.0;
        }

        if input == '\n' {
            if let Some(last_slot) = self.last_slot {
                let last_hand = last_slot.get_hand();
                // TODO: This is not correct because it's easier to hit another key with your left
                // hand after return than your left
                self.last_slot = None;
                if last_hand == Hand::Left {
                    return 0.9;
                }
            }

            self.last_slot = None;
            return 0.8;
        }

        let slot_idx: &usize = match self.slot_ref.get(&input) {
            Some(slot) => slot,
            None => return 0.0, // Not a valid key, don't affect score
        };

        let slot = self.keyslots[*slot_idx];
        const DEFAULT_EFFICIENCY: f64 = 1.0;
        let mut efficiency: f64 = DEFAULT_EFFICIENCY;

        let row: Row = slot.get_row();
        // I agree with Dvorak. The top row is easier to hit than the bottom
        if row == Row::Above {
            efficiency *= 0.92;
        }
        if row == Row::Below {
            efficiency *= 0.84;
        }

        // Penalize index finger extensions
        let col: Col = slot.get_col();
        if col == Col::Five || col == Col::Six {
            efficiency *= 0.9;
        }

        // These extensions are especially bad
        if (col == Col::Five && row == Row::Below) || (col == Col::Six && row == Row::Above) {
            efficiency *= 0.9;
        }

        let hand: Hand = slot.get_hand();
        // Because the keyboard columns slope down-right, this goes against the grain of the left
        // hand, so we penalize it here. But, only slightly because left-handed typists are out
        // there and people have different preferences
        // TODO: This is a bit of a cheat
        if hand == Hand::Left {
            efficiency *= 0.95;
        }

        // The ring and pinky fingers are penalized evenly due to variance in personal preference.
        // Neither the index nor middle finger are preferenced for the same reason
        let finger: Finger = slot.get_finger();
        if finger == Finger::Ring || finger == Finger::Pinky {
            efficiency *= 0.9;
        }

        if let Some(last_slot) = self.last_slot {
            let last_row: Row = last_slot.get_row();
            let last_hand: Hand = last_slot.get_hand();

            // TODO: This is sloppy, but don't want to over-dial in because at some point the eval
            // function is going to be more fundamentally re-written I'm sure
            // TODO: Need a way to handle these types of conditions
            // Two row jumps are not good
            let last_finger: Finger = last_slot.get_finger();
            if hand == last_hand
                && ((last_row == Row::Above && row == Row::Below)
                    || (last_row == Row::Below && row == Row::Above))
            {
                efficiency *= 0.75;

                //Scissor
                let distance_i8 = last_finger.get_num() as i8 - finger.get_num() as i8;
                if distance_i8.abs() == 1 {
                    efficiency *= 0.75;
                }
            }

            // Any row jump is not preferred
            if hand == last_hand && last_row != row {
                efficiency *= 0.9;
            }

            // Left handed row jumps are especially bad
            if hand == last_hand && hand == Hand::Left && row != last_row {
                efficiency *= 0.85;
            }

            if hand == last_hand
                && ((finger == Finger::Pinky && last_finger == Finger::Ring)
                    || (finger == Finger::Ring && last_finger == Finger::Pinky))
            {
                efficiency *= 0.75;
            }

            // Slow, causes pain
            if finger == last_finger && hand == last_hand {
                efficiency *= 0.75;
            }

            // let last_col: Col = last_slot.get_col();

            // TODO: This in particular needs refining. In general, rolls are good, but not all
            // rolls are created equal. Need to find general principles that work without
            // overtuning to individual typing preferences
            // For now, we simply go with Dvorak's notion that the typing motion should go in
            if hand == last_hand && last_finger.get_num() < finger.get_num() {
                efficiency *= 0.85;
            }
        }

        self.last_slot = Some(slot);

        return efficiency;
    }

    pub fn evaluate(&mut self, corpus: &[String]) -> Result<()> {
        if corpus.len() < 1 {
            return Err(anyhow!("No entries in corpus"));
        }

        // TODO: Is there a better way to handle this? There should be some result return you can
        // use to say "I did a new evaluation" or "I have already been evaluated"
        if self.evaluated {
            return Ok(());
        }

        // TODO: This is fine for now, but as we add more corpus entries we might run into floating
        // point precision issues. Since we eventually want to weight the parts of the corpus
        // anyway, their efficiencies should be stored individually, weighted, then added
        self.score = 0.0;
        self.clear_last_slot();

        for entry in corpus {
            for c in entry.chars() {
                self.score += self.get_efficiency(c);
            }
        }

        self.evaluated = true;

        return Ok(());
    }

    pub fn get_score(&self) -> f64 {
        return self.score;
    }

    // TODO: This is fine for drafting but needs a rework for more serious use
    pub fn display_keyboard(&self) {
        let mut above_vec: Vec<char> = Vec::new();
        let mut home_vec: Vec<char> = Vec::new();
        let mut below_vec: Vec<char> = Vec::new();

        for slot in &self.keyslots {
            let this_row = slot.get_row();
            let this_key = slot.get_key().get_base();

            if this_row == Row::Above {
                above_vec.push(this_key);
            } else if this_row == Row::Home {
                home_vec.push(this_key);
            } else if this_row == Row::Below {
                below_vec.push(this_key);
            }
        }

        println!("{:?}", above_vec);
        println!("{:?}", home_vec);
        println!("{:?}", below_vec);
    }
}
