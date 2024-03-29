use super::*;
use colored::*;
use fnv::FnvHashMap;
use rand::seq::SliceRandom;
use std::cell::{RefCell, RefMut};

/// Handles all the global object state except for Component lifetimes.
pub struct World {
    pub verbose: u8,
    width: i32,
    height: i32,
    rng: RefCell<Box<dyn RngCore>>,
    actors: FnvHashMap<Point, Vec<ComponentId>>,
    pending: Vec<(Point, ComponentId)>,
    dummy: Vec<ComponentId>,
    ticks: i32, // incremented each time components get a chance to act
}

impl World {
    pub fn new(width: i32, height: i32, rng: Box<dyn RngCore>, verbose: u8) -> World {
        World {
            width,
            height,
            verbose,
            rng: RefCell::new(rng),
            actors: FnvHashMap::default(),
            pending: Vec::new(),
            dummy: Vec::new(),
            ticks: 0,
        }
    }

    pub fn rng(&self) -> RefMut<Box<dyn RngCore>> {
        self.rng.borrow_mut()
    }

    /// Note that the world is a toroid so locations wrap around.
    pub fn cell(&self, loc: Point) -> &Vec<ComponentId> {
        let loc = self.wrap(loc);
        &self.actors.get(&loc).unwrap_or(&self.dummy)
    }

    /// Use this for components that should always be rendered.
    pub fn add_back(&mut self, store: &Store, loc: Point, component: Component) {
        assert!(has_trait!(component, Action)); // required traits, objects may make use of others
        assert!(has_trait!(component, Render));

        let loc = self.wrap(loc);
        let actors = self.actors.entry(loc).or_default();
        actors.push(component.id);
        store.add(component)
    }

    /// Use this for components that are rendered when they are the only component.
    pub fn add_front(&mut self, store: &Store, loc: Point, component: Component) {
        assert!(has_trait!(component, Action)); // required traits, objects may make use of others
        assert!(has_trait!(component, Render));

        let loc = self.wrap(loc);
        let actors = self.actors.entry(loc).or_default();
        actors.insert(0, component.id);
        store.add(component)
    }

    pub fn move_to(&mut self, id: ComponentId, old_loc: Point, new_loc: Point) {
        let old_loc = self.wrap(old_loc);
        let new_loc = self.wrap(new_loc);
        let old_ids = self.actors.get_mut(&old_loc).unwrap();
        let index = old_ids.iter().position(|e| *e == id).unwrap();
        old_ids.remove(index);

        let new_ids = self.actors.entry(new_loc).or_default();
        new_ids.push(id);
    }

    pub fn remove(&mut self, store: &Store, id: ComponentId, loc: Point) {
        let loc = self.wrap(loc);
        let old_ids = self.actors.get_mut(&loc).unwrap();
        let index = old_ids.iter().position(|e| *e == id).unwrap();
        old_ids.remove(index);
        store.remove(id);

        if let Some(index) = self
            .pending
            .iter()
            .position(|(pt, i)| *pt == loc && *i == id)
        {
            // Don't act on components scheduled to be deleted.
            self.pending.remove(index);
        }
    }

    /// Return all cells within radius of loc that satisfy the predicate.
    pub fn all<P>(&self, loc: Point, radius: i32, predicate: P) -> Vec<Point>
    where
        P: Fn(Point) -> bool,
    {
        let mut cells = Vec::new();
        let loc = self.wrap(loc);
        for dy in -radius..=radius {
            let y = loc.y + dy;
            for dx in -radius..=radius {
                let x = loc.x + dx;
                let candidate = Point::new(x, y);
                if predicate(candidate) {
                    cells.push(candidate);
                }
            }
        }
        cells
    }

    /// Allow all components a chance to act.
    pub fn step(&mut self, store: &mut Store) {
        // 1) This is tricky code because we're interating over components that may modify
        // themselves and the world (e.g. by removing another component). We address this
        // by updating pending when a component is removed via an act call and by handling
        // component lifetimes in a separate Store object which uses interior mutability
        // to defer mutations until a sync call.
        // actor before calling act.
        // 2) Because act may add new actors we take care to not call act on them until
        // the next go around.
        // 3) To avoid bias as to execution order we randomize the order in which they are
        // acted upon.
        assert!(self.pending.is_empty());
        for (loc, ids) in self.actors.iter() {
            for id in ids {
                self.pending.push((*loc, *id));
            }
        }
        self.pending[..].shuffle(self.rng.borrow_mut().as_mut());

        while !self.pending.is_empty() {
            let (loc, id) = self.pending.pop().unwrap();
            {
                let context = Context {
                    world: self,
                    store: &store,
                    loc,
                    id,
                };

                let component = store.get(id);
                let mut action = find_trait_mut!(component, Action).unwrap();
                if action.act(context) == LifeCycle::Dead {
                    let ids = self.actors.get_mut(&loc).unwrap();
                    let index = ids.iter().position(|e| *e == id).unwrap();
                    ids.remove(index);
                    store.remove(id);
                }
            }
            store.sync();
        }
        self.ticks += 1;
    }

    /// Render all cells to the terminal.
    pub fn render(&self, store: &Store) -> LifeCycle {
        let mut cycle = LifeCycle::Dead;

        println!("ticks: {}", self.ticks);
        if self.verbose >= 1 {
            print!("  ");
            for x in 0..self.width {
                print!("{}", x % 10);
            }
            println!();
        }
        for y in 0..self.height {
            if self.verbose >= 1 {
                print!("{} ", y % 10);
            }
            for x in 0..self.width {
                let loc = Point::new(x, y);
                if let Some(id) = self.actors.get(&loc).map(|v| v.last()).flatten() {
                    let component = store.get(*id);
                    let render = find_trait!(component, Render).unwrap();
                    let ch = render.render();
                    if ch != "|".normal() && ch != " ".normal() {
                        cycle = LifeCycle::Alive;
                    }
                    print!("{}", ch);
                } else {
                    print!(" ");
                }
            }
            println!();
        }
        println!();
        println!("{}", "-".repeat(self.width as usize));
        cycle
    }

    /// Because the world is a toroid the distance between two locations has to take into
    /// account the edges of the world.
    pub fn distance2(&self, loc1: Point, loc2: Point) -> i32 {
        let mut dx = i32::abs(loc1.x - loc2.x);
        let mut dy = i32::abs(loc1.y - loc2.y);

        if dx > self.width / 2 {
            dx = self.width / 2 - dx;
        }

        if dy > self.height / 2 {
            dy = self.height - dy;
        }

        dx * dx + dy * dy
    }

    fn wrap(&self, loc: Point) -> Point {
        let mut x = loc.x;
        let mut y = loc.y;

        x = if x >= 0 {
            x % self.width
        } else {
            x + self.width
        };

        y = if y >= 0 {
            y % self.height
        } else {
            y + self.height
        };

        Point::new(x, y)
    }
}
