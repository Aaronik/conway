// pub struct Snapshot {
//     previous: String,
//     current: String,
// }

// impl Snapshot {
//     pub fn new() -> Snapshot {
//         Snapshot {
//             previous: String::from(""),
//             current: String::from(""),
//         }
//     }

//     pub fn add(&mut self, i: u32, j: u32) {
//         self.current.push_str(&format!("{i}-{j}|"));
//     }

//     /// Is the current build up snapshot the same as the previous one, the one that most recently
//     /// got cycled?
//     pub fn is_same(&self) -> bool {
//         self.current == self.previous
//     }

//     /// Move the current snapshot to the previous slot and reset current for another round
//     pub fn cycle(&mut self) {
//         self.previous.clear();
//         self.previous
//             .push_str(&self.current.drain(..self.current.len()).collect::<String>()[..]);
//         self.current.clear();
//     }
// }

// This needs to:
// * Keep a list of every state in the history of an evolution
// * Be able to compare those states and return equal for two equivalent states
// * However there's no guarantee from our Cells grid the order in which living cells
// will be returned, so we need to be order agnostic
// * Be fast
//
// TODO What're we doin here, it's actually pretty interesting and should be moved over
// to a readme showing off all the mathiness.
// Where String comes from the serialization of a BTreeSet. Yes, each
// cell in the grid can be represented by a number: SIZE*i + j. Thus we
// can have ordering, and a BTreeSet. That set can be serialized
// deterministcally into a string, which can be stored in a hashset.
// Along side this hash set can be a vector with the strings, for
// ordering. So at each iteration of the grid, we can check in O(1)
// if this has been a state that we've seen yet, and then if so we can
// see how far back it happened, and that's our fitness.

use std::collections::{BTreeSet, HashSet};

pub struct Snapshot {
    grids_vec: Vec<String>,
    grids_set: HashSet<String>,
    cells: BTreeSet<u32>,
    size: u32,
    has_repeat: bool,
}

impl Snapshot {
    pub fn new(size: u32) -> Snapshot {
        Snapshot {
            grids_vec: Vec::new(),
            grids_set: HashSet::new(),
            cells: BTreeSet::new(),
            size,
            has_repeat: false,
        }
    }

    pub fn add_cell(&mut self, i: u32, j: u32) {
        let cell_number = self.size * i + j;
        self.cells.insert(cell_number);
    }

    /// Is the current build up snapshot the same as the previous one, the one that most recently
    /// got cycled?
    pub fn has_repeat(&self) -> bool {
        self.has_repeat
    }

    // If has_repeat is true, this will return how long the repeat period is
    pub fn period(&self) -> usize {
        if !self.has_repeat {
            panic!("snapshot.period called with snapshot.has_repeat is false. Wait till true to call!");
        };

        let most_recent = self.grids_vec.first().unwrap();
        self.grids_vec.iter().enumerate().position(|(index, grid)| {
            grid == most_recent
        }).unwrap() + 2 // TODO Remove this + 2, bring back that index != 0
    }

    /// Commit the cells that were added to memory as a single grid state.
    pub fn commit_cells(&mut self) {
        let serialized = self.serialize_cells();
        self.grids_vec.push(serialized.clone()); // TODO It'd be way better if it lived in
                                                 // grids_set and was referenced in here
        // TODO Ok we're getting false values here. It's showing a repeat when there is no
        // repeat.
        let has_repeat = !self.grids_set.insert(serialized);

        if has_repeat {
            self.has_repeat = true;
        }
    }

    fn serialize_cells(&self) -> String {
        let mut serialized = String::from("");

        self.cells.iter().for_each(|cell| {
            serialized += &cell.to_string();
            serialized += "|";
        });

        serialized
    }
}
