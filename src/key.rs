use rand::{Rng, rngs::SmallRng};

use crate::key_template::KeyTemplate;

#[derive(Debug, Clone)]
pub struct Key {
    base: u8,
    shift: u8,
    valid_locations: Vec<(usize, usize)>,
    is_static: bool,
    template: KeyTemplate,
}

impl Key {
    pub fn from_template(template: KeyTemplate) -> Self {
        let base: u8 = template.get_base();
        let shift: u8 = template.get_shift();
        let valid_locations: Vec<(usize, usize)> = template.get_valid_locations();

        let is_static = valid_locations.len() == 1;

        if valid_locations.len() == 0 {
            panic!("Template provided no valid locations");
        }

        return Self {
            base,
            shift,
            valid_locations,
            is_static,
            template,
        };
    }

    pub fn get_base(&self) -> u8 {
        return self.base;
    }

    pub fn get_shift(&self) -> u8 {
        return self.shift;
    }

    pub fn is_static(&self) -> bool {
        return self.is_static;
    }

    pub fn get_cnt_valid_locations(&self) -> usize {
        return self.valid_locations.len();
    }

    pub fn get_valid_locations(&self) -> &[(usize, usize)] {
        return &self.valid_locations;
    }

    pub fn shuffle_valid_locations(&mut self, rng: &mut SmallRng) {
        for i in 0..self.valid_locations.len() - 1 {
            let j = rng.random_range((i + 1)..self.valid_locations.len());

            self.valid_locations.swap(i, j);
        }
    }

    pub fn get_valid_location_at_idx(&self, idx: usize) -> (usize, usize) {
        return self.valid_locations[idx];
    }

    pub fn get_template(&self) -> KeyTemplate {
        return self.template;
    }
}

#[derive(Debug, Clone)]
pub struct LightKey {
    base: u8,
    shift: u8,
    template: KeyTemplate,
}

impl LightKey {
    pub fn from_template(template: KeyTemplate) -> Self {
        let base: u8 = template.get_base();
        let shift: u8 = template.get_shift();

        return Self {
            base,
            shift,
            template,
        };
    }

    pub fn get_base(&self) -> u8 {
        return self.base;
    }

    pub fn get_shift(&self) -> u8 {
        return self.shift;
    }

    pub fn get_template(&self) -> KeyTemplate {
        return self.template;
    }
}
