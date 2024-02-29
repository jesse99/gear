use super::*;
use fnv::FnvHashMap;
use std::cell::RefCell;

/// Manages [`Component`]` lifetimes. This is broken out from [`World`] to avoid borrow
/// checker issues.
pub struct Store {
    components: FnvHashMap<ComponentId, Component>,
    liverow: RefCell<Vec<Component>>,
    deathrow: RefCell<Vec<ComponentId>>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            components: FnvHashMap::default(),
            liverow: RefCell::new(Vec::new()),
            deathrow: RefCell::new(Vec::new()),
        }
    }

    pub fn get(&self, id: ComponentId) -> &Component {
        self.components.get(&id).unwrap()
    }

    pub fn add(&self, actor: Component) {
        self.liverow.borrow_mut().push(actor);
    }

    pub fn remove(&self, id: ComponentId) {
        self.deathrow.borrow_mut().push(id);
    }

    pub fn sync(&mut self) {
        for component in self.liverow.take() {
            let old = self.components.insert(component.id, component);
            assert!(old.is_none());
        }

        for id in self.deathrow.take() {
            let old = self.components.remove(&id);
            assert!(old.is_some());
        }
    }
}
