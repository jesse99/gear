use std::cmp::Ordering;
use std::fmt::{self, Formatter};
use std::hash::{Hash, Hasher};

/// Represents a point in cartesian space,.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

impl Ord for Point {
    fn cmp(&self, rhs: &Self) -> Ordering {
        if self.y < rhs.y {
            Ordering::Less
        } else if self.y > rhs.y {
            Ordering::Greater
        } else if self.x < rhs.y {
            Ordering::Less
        } else if self.x > rhs.y {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Hash for Point {
    // This should be quite a bit better than simply folding x onto y.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut s = self.x as i64;
        s <<= 32;
        s |= self.y as i64;
        s.hash(state);
    }
}
