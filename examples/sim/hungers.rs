use super::*;

pub struct Hungers {
    hunger: i32, // [0, max_hunger]
    max_hunger: i32,
}
register_type!(Hungers);

impl Hungers {
    pub fn new(initial: i32, max: i32) -> Hungers {
        Hungers {
            hunger: initial,
            max_hunger: max,
        }
    }
}

impl Hunger for Hungers {
    fn get(&self) -> i32 {
        self.hunger
    }

    fn set(&mut self, value: i32) {
        assert!(value >= 0);
        assert!(value <= self.max_hunger);
        self.hunger = value;
    }

    fn adjust(&mut self, delta: i32) {
        if delta > 0 {
            if self.hunger <= self.max_hunger - delta {
                self.hunger += delta;
            } else {
                self.hunger = self.max_hunger;
            }
        } else {
            if self.hunger >= -delta {
                self.hunger += delta;
            } else {
                self.hunger = 0;
            }
        }

        assert!(self.hunger >= 0);
        assert!(self.hunger <= self.max_hunger);
    }
}
