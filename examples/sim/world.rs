use super::*;
use std::collections::HashMap;

// Top-level sim state.
pub struct World {
    pub width: i32,
    pub height: i32,
    terrain: HashMap<Point, Component>, // grass or potentially stuff like water
    animals: HashMap<Point, Component>, // sheep or wolf
    ticks: i32,                         // incremented each time actors get a chance to act
}

impl World {
    pub fn new(width: i32, height: i32) -> World {
        World {
            width,
            height,
            terrain: HashMap::new(),
            animals: HashMap::new(),
            ticks: 0,
        }
    }

    pub fn add(&mut self, loc: Point, actor: Component) {
        assert!(loc.x >= 0);
        assert!(loc.y >= 0);
        assert!(loc.x < self.width);
        assert!(loc.y < self.height);
        assert!(has_trait!(actor, Action)); // required traits, objects may make use of others
        assert!(has_trait!(actor, Render));

        let old = if has_trait!(actor, Terrain) {
            self.terrain.insert(loc, actor)
        } else {
            self.animals.insert(loc, actor)
        };
        assert!(old.is_none());
    }

    pub fn get_terrain(&self, loc: Point) -> Option<&Component> {
        self.terrain.get(&loc)
    }

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

    pub fn step(&mut self) {
        // This is delicate because we want to mutate actors which may also cause the world
        // to mutate (e.g. they can add new actors to the world).
        let locs: Vec<Point> = self.terrain.keys().copied().collect(); // TODO: randomize these
        for loc in locs {
            // Earlier actor may have deleted this one.
            if let Some(mut actor) = self.terrain.remove(&loc) {
                let action = find_trait_mut!(actor, Action).unwrap();
                let alive = action.act(self, loc);
                if alive {
                    // This is safe because while an actor may add a new actor to a
                    // neighboring cell it won't add one to its own cell.
                    self.add(loc, actor);
                }
            }
        }

        // Difficult to avoid this repetition because of the borrow checker.
        let locs: Vec<Point> = self.animals.keys().copied().collect(); // TODO: randomize these
        for loc in locs {
            if let Some(mut actor) = self.animals.remove(&loc) {
                let action = find_trait_mut!(actor, Action).unwrap();
                let alive = action.act(self, loc);
                if alive {
                    self.add(loc, actor);
                }
            }
        }

        self.ticks += 1;
    }

    pub fn render(&self) {
        println!("{}  ticks: {}", "-".repeat(self.width as usize), self.ticks);
        for y in 0..self.height {
            for x in 0..self.width {
                let loc = Point::new(x, y);
                if let Some(actor) = self.animals.get(&loc) {
                    let render = find_trait!(actor, Render).unwrap();
                    print!("{}", render.render());
                } else {
                    if let Some(actor) = self.terrain.get(&loc) {
                        let render = find_trait!(actor, Render).unwrap();
                        print!("{}", render.render());
                    } else {
                        print!(" ");
                    }
                }
            }
            println!();
        }
        println!();
    }
}
