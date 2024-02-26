use super::*;
use rand::seq::SliceRandom;
use std::collections::HashMap;

// Top-level sim state.
pub struct World {
    pub width: i32,
    pub height: i32,
    pub rng: Box<dyn RngCore>,
    actors: HashMap<Point, Vec<Component>>,
    dummy: Vec<Component>,
    ticks: i32, // incremented each time actors get a chance to act
}

impl World {
    pub fn new(width: i32, height: i32, rng: Box<dyn RngCore>) -> World {
        World {
            width,
            height,
            rng,
            actors: HashMap::new(),
            dummy: Vec::new(),
            ticks: 0,
        }
    }

    pub fn get(&self, loc: Point) -> &Vec<Component> {
        &self.actors.get(&loc).unwrap_or(&self.dummy)
    }

    pub fn add(&mut self, loc: Point, actor: Component) {
        assert!(loc.x >= 0);
        assert!(loc.y >= 0);
        assert!(loc.x < self.width);
        assert!(loc.y < self.height);
        assert!(has_trait!(actor, Action)); // required traits, objects may make use of others
        assert!(has_trait!(actor, Render));

        let actors = self.actors.entry(loc).or_default();
        actors.push(actor);
    }

    // remove would take a &Component? might need to add some sort of id to Component so
    // that we can find the right one

    /// Return all cells within radius of loc that satisfy the predicate.
    pub fn all<P>(&self, loc: Point, radius: i32, predicate: P) -> Vec<Point>
    where
        P: Fn(Point) -> bool,
    {
        let mut cells = Vec::new();
        for dy in -radius..=radius {
            let y = loc.y + dy;
            if y >= 0 && y < self.height {
                for dx in -radius..=radius {
                    let x = loc.x + dx;
                    if x >= 0 && x < self.width {
                        let candidate = Point::new(x, y);
                        if candidate != loc && predicate(candidate) {
                            cells.push(candidate);
                        }
                    }
                }
            }
        }
        cells
    }

    /// Allow all components a chance to act.
    pub fn step(&mut self) {
        // TODO:
        // actors should be {Point => [ID]}
        //    then have {ID => Component}
        // here we could assemble a randomized list of IDs to act on
        //    save this list in a field
        // remove would erase that id from list to process

        // 1) This is tricky code because we're iterating over actors which may mutate
        // themselves or the world or other actors. That's why we temporarily remove an
        // actor before calling act.
        // 2) To avoid bias as to execution order we randomize the order in which they
        // are acted upon.
        let mut locs: Vec<Point> = self.actors.keys().copied().collect();
        locs[..].shuffle(&mut self.rng);
        for loc in locs {
            let len = self.len_at(loc);
            let start: usize = self.rng.gen_range(0..len);

            // TODO: This won't quite work if act removes an earlier actor. Could maybe:
            // 1) set an executing flag here
            // 2) remove and add would append onto a deferred action list
            //    think we only need to do this for removes
            //    and technically just removes at loc
            // 3) after re-inserting the original actor run deferred actions
            //    would have to update the local i variable on removes
            //    probably len local too
            let mut i = 0;
            while i < len {
                let index = (start + i) % len;
                let mut actor = self.remove_at(loc, index);
                let action = find_trait_mut!(actor, Action).unwrap();
                let alive = action.act(self, loc);
                if alive {
                    self.insert_at(loc, index, actor);
                    i += 1;
                }
            }
        }

        self.ticks += 1;
    }

    /// Render all cells to the terminal.
    pub fn render(&self) {
        println!("{}  ticks: {}", "-".repeat(self.width as usize), self.ticks);
        for y in 0..self.height {
            for x in 0..self.width {
                let loc = Point::new(x, y);
                if let Some(actor) = self.actors.get(&loc).map(|v| v.last()).flatten() {
                    let render = find_trait!(actor, Render).unwrap();
                    print!("{}", render.render());
                } else {
                    print!(" ");
                }
            }
            println!();
        }
        println!();
    }
}

// internals
impl World {
    fn len_at(&mut self, loc: Point) -> usize {
        self.actors.get_mut(&loc).map_or(0, |v| v.len())
    }

    fn remove_at(&mut self, loc: Point, i: usize) -> Component {
        self.actors.get_mut(&loc).unwrap().remove(i)
    }

    fn insert_at(&mut self, loc: Point, i: usize, actor: Component) {
        self.actors.get_mut(&loc).unwrap().insert(i, actor);
    }
}
