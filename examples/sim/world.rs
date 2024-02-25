use super::*;
use std::collections::HashMap;

// Top-level sim state.
pub struct World {
    pub width: i32,
    pub height: i32,
    actors: HashMap<Point, Vec<Component>>,
    ticks: i32, // incremented each time actors get a chance to act
    dummy: Vec<Component>,
}

impl World {
    pub fn new(width: i32, height: i32) -> World {
        World {
            width,
            height,
            actors: HashMap::new(),
            ticks: 0,
            dummy: Vec::new(),
        }
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

    pub fn get(&self, loc: Point) -> &Vec<Component> {
        &self.actors.get(&loc).unwrap_or(&self.dummy)
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

    /// Allow all components a chance to act.
    pub fn step(&mut self) {
        // This is delicate because we want to mutate actors which may also cause the world
        // to mutate (e.g. they can add new actors to the world).
        let locs: Vec<Point> = self.actors.keys().copied().collect(); // TODO: randomize these
        for loc in locs {
            let len = self.len_at(loc);
            let mut i = 0;
            while i < len {
                let mut actor = self.remove_at(loc, i);
                let action = find_trait_mut!(actor, Action).unwrap();
                let alive = action.act(self, loc);
                if alive {
                    self.insert_at(loc, i, actor);
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
