#!./make.py --10
// from @raffimolero / redstoneboi https://www.reddit.com/r/rust/comments/183ex3i/comment/kaoxj74/?context=3

//INFO: algo with specialized types

use std::{
    fmt::{Display, Formatter},
    io,
    iter::FromIterator,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

struct Grid {
    data: [NumSet; 81],
    spaces: SpaceSet,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct NumSet(u16);

impl NumSet {
    const ALL: Self = Self(0b_111_111_111);
    const EMPTY: Self = Self(0);

    fn one_hot(val: u8) -> Self {
        Self(1 << val)
    }

    fn val(self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    fn len(self) -> u32 {
        self.0.count_ones()
    }
}

impl FromIterator<Self> for NumSet {
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|a, b| a + b)
            .expect("iterator must have more than 1 value")
    }
}

impl Add for NumSet {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        NumSet(self.0 | rhs.0)
    }
}

impl AddAssign for NumSet {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for NumSet {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        NumSet(self.0 & !rhs.0)
    }
}

impl SubAssign for NumSet {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

struct NumSetIter {
    set: NumSet,
    mask: u16,
}

impl Iterator for NumSetIter {
    type Item = NumSet;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.mask >>= 1;
            if self.mask == 0 {
                return None;
            }
            let masked = self.set.0 & self.mask;
            if masked != 0 {
                return Some(NumSet(masked));
            }
        }
    }
}

impl IntoIterator for NumSet {
    type Item = NumSet;
    type IntoIter = NumSetIter;
    fn into_iter(self) -> NumSetIter {
        NumSetIter {
            set: self,
            mask: 1 << 9,
        }
    }
}

#[derive(Clone, Copy)]
struct Space {
    i: usize,
    x: usize,
    y: usize,
    b: usize,
}

impl From<usize> for Space {
    fn from(value: usize) -> Self {
        let x = value % 9;
        let y = value / 9;
        let bx = x / 3 * 3;
        let by = y / 3 * 3;
        Self {
            i: value,
            x,
            y: y * 9,
            b: by * 9 + bx,
        }
    }
}

impl Space {
    const DEFAULT: Self = Self {
        i: 0,
        x: 0,
        y: 0,
        b: 0,
    };
}

struct SpaceSet {
    data: [Space; 81],
    len: usize,
}

impl SpaceSet {
    fn empty() -> Self {
        Self {
            data: [Space::DEFAULT; 81],
            len: 0,
        }
    }

    fn insert(&mut self, item: Space) {
        self.data[self.len] = item;
        self.len += 1;
    }

    fn remove(&mut self, index: usize) {
        self.len -= 1;
        self.data[index] = self.data[self.len];
    }

    fn iter(&self) -> impl '_ + Iterator<Item = Space> {
        self.data[..self.len].iter().copied()
    }
}

impl Grid {
    fn sqr(&self, b: usize) -> NumSet {
        NumSet::from_iter(
            self.data[b..b + 3]
                .iter()
                .chain(&self.data[b + 9..b + 12])
                .chain(&self.data[b + 18..b + 21])
                .copied(),
        )
    }

    fn col(&self, x: usize) -> NumSet {
        NumSet::from_iter((0..9).map(|y| self.data[y * 9 + x]))
    }

    fn row(&self, y: usize) -> NumSet {
        NumSet::from_iter(self.data[y..y + 9].iter().copied())
    }

    fn free(&self, space: Space) -> NumSet {
        let col = self.col(space.x);
        let row = self.row(space.y);
        let sqr = self.sqr(space.b);
        NumSet::ALL - (col + row + sqr)
    }

    fn resolv(&mut self) -> bool {
        let mut best_space_index = 0;
        let mut best_space = Space::DEFAULT;
        let mut best_set = NumSet::ALL;
        let mut best_set_len = 10;
        for (i, space) in self.spaces.iter().enumerate() {
            let free = self.free(space);
            let set_len = free.len();
            if set_len == 0 {
                // Unsolvable.
                return false;
            }
            if set_len < best_set_len {
                // Found better candidate set, update all.
                best_space_index = i;
                best_space = space;
                best_set = free;
                best_set_len = set_len;
            }
            if set_len == 1 {
                // Only one candidate here; we can't do better.
                break;
            }
        }

        if best_set_len == 10 {
            // Best set was never updated. Solved.
            return true;
        };

        self.spaces.remove(best_space_index);
        for c in best_set {
            self.data[best_space.i] = c;
            if self.resolv() {
                return true;
            }
        }
        self.data[best_space.i] = NumSet::EMPTY;
        self.spaces.insert(best_space);

        false
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParseGridError {
    pos: usize,
}

impl FromStr for Grid {
    type Err = ParseGridError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = [NumSet::EMPTY; 81];
        let mut spaces = SpaceSet::empty();
        for (i, (g, c)) in data.iter_mut().zip(s.chars()).enumerate() {
            match c {
                '1'..='9' => *g = NumSet::one_hot(c as u8 - b'1'),
                '.' => spaces.insert(i.into()),
                _ => return Err(ParseGridError { pos: i }),
            }
        }
        Ok(Grid { data, spaces })
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for g in self.data {
            let c = match g {
                NumSet::EMPTY => '.',
                _ => (b'1' + g.val()) as char,
            };
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let content = fs::read_to_string("grids.txt")?;
//     let gg: Vec<&str> = content.lines().take(1956).collect();

//     for line in gg {
//         let mut grid: Grid = line.trim().parse().unwrap();
//         grid.resolv();
//         println!("{} ", grid);
//     }
//     Ok(())
// }

fn main() {
    // Iterate over the lines in io::stdin()
    for line in io::stdin().lines() {
        let mut grid: Grid = line.unwrap().trim().parse().unwrap();
        grid.resolv();
        println!("{} ", grid);
    }
}
