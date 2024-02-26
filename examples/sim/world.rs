use super::*;
use rand::seq::SliceRandom;
use std::borrow::Cow;
use std::collections::HashMap;

// Top-level sim state.
pub struct World {
    pub width: i32,
    pub height: i32,
    pub rng: Box<dyn RngCore>,
    actors: HashMap<Point, Vec<ComponentId>>,
    components: HashMap<ComponentId, Component>,
    pending: Vec<(Point, ComponentId)>,
    executing: (Point, ComponentId),
    dummy: Vec<ComponentId>,
    ticks: i32, // incremented each time actors get a chance to act
}

impl World {
    pub fn new(width: i32, height: i32, rng: Box<dyn RngCore>) -> World {
        World {
            width,
            height,
            rng,
            actors: HashMap::new(),
            components: HashMap::new(),
            pending: Vec::new(),
            executing: (Point::new(-1, -1), next_component_id()),
            dummy: Vec::new(),
            ticks: 0,
        }
    }

    pub fn get(&self, id: ComponentId) -> &Component {
        self.components.get(&id).unwrap()
    }

    pub fn cell(&self, loc: Point) -> Cow<Vec<ComponentId>> {
        if loc == self.executing.0 {
            // If we're currently executing an id at loc then we cannot return the id
            // here because callers cannot use it (step temporarily removes it from self.
            // components). So, when that happens, we return a new vector without that id.
            // This is potentially a problem if that object wants access to another object
            // on its component. TODO: can probably fix this by passing the component into
            // act.
            //
            // I experimented with avoiding allocating a new vector by using a RefCell
            // and returning `impl Iterator<Item=ComponentId> + '_ ` or
            // `Box<dyn Iterator<Item = &ComponentId> + '_>`or &Vec<ComponentId> but
            // those didn't work because the iterator types are different and/or the
            // temporary reference from the RefCell borrow didn't live long enough.
            let temp = self
                .actors
                .get(&loc)
                .unwrap()
                .iter()
                .copied()
                .take_while(|id| *id != self.executing.1)
                .collect();
            Cow::Owned(temp)
        } else {
            Cow::Borrowed(self.actors.get(&loc).unwrap_or(&self.dummy))
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
        actors.push(actor.id);

        let old = self.components.insert(actor.id, actor);
        assert!(old.is_none());
    }

    pub fn move_to(&mut self, id: ComponentId, old_loc: Point, new_loc: Point) {
        let old_ids = self.actors.get_mut(&old_loc).unwrap();
        let index = old_ids.iter().position(|e| *e == id).unwrap();
        old_ids.remove(index);

        let new_ids = self.actors.entry(new_loc).or_default();
        new_ids.push(id);
    }

    // TODO: remove has to update pending (might want to make this a hashset)

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
        // 1) This is tricky code because we're iterating over actors which may mutate
        // themselves or the world or other actors. That's why we temporarily remove the
        // actor before calling act.
        // 2) Because act may add new actors we take care to not call act on them until
        // the next go around.
        // 3) To avoid bias as to execution order we randomize the order in which they
        // are acted upon.
        assert!(self.pending.is_empty());
        for (loc, ids) in self.actors.iter() {
            for id in ids {
                self.pending.push((*loc, *id));
            }
        }
        self.pending[..].shuffle(&mut self.rng);

        while !self.pending.is_empty() {
            let (loc, id) = self.pending.pop().unwrap();
            let actor = self.components.remove(&id).unwrap();
            let id = actor.id;

            self.executing = (loc, id);
            let action = find_trait_mut!(actor, Action).unwrap();
            let alive = action.act(self, id, loc);
            if alive {
                self.components.insert(id, actor);
            } else {
                let ids = self.actors.get_mut(&loc).unwrap();
                let index = ids.iter().position(|e| *e == id).unwrap();
                ids.remove(index);
            }
        }

        self.executing = (Point::new(-1, -1), self.executing.1);
        self.ticks += 1;
    }

    /// Render all cells to the terminal.
    pub fn render(&self) {
        println!("{}  ticks: {}", "-".repeat(self.width as usize), self.ticks);
        for y in 0..self.height {
            for x in 0..self.width {
                let loc = Point::new(x, y);
                if let Some(id) = self.actors.get(&loc).map(|v| v.last()).flatten() {
                    let actor = self.get(*id);
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
