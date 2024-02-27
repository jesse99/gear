use super::*;

pub struct Hungering {
    hunger: i32, // [0, max_hunger)
    max_hunger: i32,
}
register_type!(Hungering);

impl Hungering {
    pub fn new(initial: i32, max: i32) -> Hungering {
        Hungering {
            hunger: initial,
            max_hunger: max,
        }
    }
}

impl Hunger for Hungering {
    fn get(&self) -> i32 {
        self.hunger
    }

    fn set(&mut self, value: i32) {
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
    }
}
