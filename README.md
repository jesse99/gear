# gear

Gear is a component object model for rust. So why does rust need a new object model? And
what the heck is a component object model?

## Four pillars of OOP

Grady Booch defined [four pillars](https://thegeekyasian.com/4-pillars-of-oop/) of OOP:

1. **Abstraction** Instead of dealing with objects directly, prefer to use an abstraction
that hides all unnecessary complexity. For example, use a base class instead of a concrete
class. This is a very good idea because managing complexity is hugely important in
software development and, if we can minimize that to client code, life becomes much easier.
2. **Encapsulation** State should be hidden away so that client code cannot directly
access it. In particular, objects often have [invariants](https://www.geeksforgeeks.org/what-is-class-invariant/)
that must be preserved so if state can only be changed via a method those invariants can
be maintainted (or at least checked).
3. **Polymorphism** This allows a type to be used as a more abstract type. For example,
in a GUI a Checkbox can be passed to a method that expects a Button or a ListView can
maintain a vector of View objects which might actually be a CheckBox and a Label.
4. **Inheritance** There are two forms of this: interface inheritance and implementation
inheritance. Interface inheritance is similar to rust trait implementations. Implementation
inheritance is classic OOP where a class inherits from another class and tweaks or adds
something. For example, you might have a FlashingButton that inherits from Button and
overrides the draw method so that it blinks on and off.

These are all useful and together work well in important domains like GUIs, sims, games,
plugin architectures, etc.

## Rust vs the pillars

1. **Abstraction** Traits are the principle way that rust handles abstraction. These work
fine in many cases but they don't compose well.
2. **Encapsulation** Rust handles this quite well with its visibility controls and module
system.
3. **Polymorphism** Traits again, either `impl Trait` or `Box<dyn Trait>`. But again, there
are composibility problems. For example, it quickly gets very awkward if you want a
Checkbox that can act Like a Button which can act like a View.
4. **Inheritance** Rust works well with interface inheritance but doesn't support
implementation inheritance at all. There are good reasons for this: while implementation
inheritance is useful it's also very
[problematic](https://www.tedinski.com/2018/02/13/inheritance-modularity.html) because it
introduces a tight coupling between the base and derived classes. Because of the issues
that causes the consensus now is to prefer
[composition over inheritance](https://en.wikipedia.org/wiki/Composition_over_inheritance).

## Gear basics

The gear object model consists of:

1. Components which contain one or more objects.
2. Objects which implement one or more traits.
3. Macros which allow access to traits implemented by the objects.

For example, you might have a component which exposes a Draw trait:

```rust
let component = get_from_somewhere();
let render = find_trait!(component, Render).unwrap();
render.render(&mut canvas);
```

## Gear vs the pillars

1. **Abstraction** Client code deals only with traits so abstraction is great. A new trait
can always be added to an object or a new object to a component so composition is no
problem (and can even by dunamic).
2. **Encapsulation** Also great because clients deal with traits not concrete types.
3. **Polymorphism** Components can expose as many traits as they want and that's all
hidden inside the Component so code can take a Component and query for the traits they
need. This is quite powerful though does have the downside that it is dynamic: there's
no compile time check that the trait actually exists.
4. **Inheritance** Behavior is not added by extending classes but by adding objects to a
component (or implementing a new trait on an existing object). This is not as powerful as
implementation inheritance but it's much simpler and way less brittle.

## Sample code

The [repro](https://github.com/jesse99/gear/tree/main/examples/sim) has an example that
consists of a wolf vs sheep simulation. Here is some annotated code from that to better illustrate how gear works:

```rust
// Gear currently requires nightly and you'll need this in your main.rs (or lib.rs).
#![feature(lazy_cell)]
#![feature(ptr_metadata)]
#![feature(unsize)]

// One of the traits the sim components expose. This is used by the World struct to render
// all of the components in the sim.
pub trait Render {
    fn render(&self) -> ColoredString;
}
register_type!(Render); // traits exposed by components have to be registered

// Function to add a wolf to the world and to the Store (the Store manages component
// lifetimes).
pub fn add_wolf(world: &mut World, store: &Store, loc: Point) -> ComponentId {
    let mut component = Component::new("wolf"); // the name is used with Component's Debug trait
    let id = component.id;  // each component is given a unique id allowing components to be compared and hashed
    add_object!(
        component,
        Wolf,           // object type
        Wolf::new(),    // object
        [Action, Animal, Predator, Render], // traits exposed by this object to the component
        [Debug]         // repeated traits (these can appear multiple times in a Component)
    );
    add_object!(component, Mover, Mover::new(), [Moveable]);    // code reuse: sheeps also move
    add_object!(        // more code reuse
        component,
        Hungers,
        Hungers::new(INITAL_HUNGER, MAX_HUNGER),
        [Hunger],
        [Debug]         // Debug is repeated for this Component
    );
    world.add_back(store, loc, component);  // animals are added to the back and grass to the front
    id
}

// Concrete object type, not directly used by client code.
struct Wolf {
    age: i32,       // wolves can die of old age
}
register_type!(Wolf);

// Traits are implemented normally.
impl Render for Wolf {
    fn render(&self) -> ColoredString {
        if self.age == 0 {
            "w".green()
        } else {
            "w".normal()
        }
    }
}

// Somewhat simplified version of the World::render method.
// Example of how components are used.
pub fn render(&self, store: &Store) {
    for y in 0..self.height {
        for x in 0..self.width {
            let loc = Point::new(x, y);
            if let Some(id) = self.actors.get(&loc).map(|v| v.last()).flatten() {   // render the last component at a loc
                let component = store.get(*id);
                let render = find_trait!(component, Render).unwrap();
                let ch = render.render();
                print!("{}", ch);
            } else {
                print!(" ");
            }
        }
        println!();
    }
    println!();
}
}```
