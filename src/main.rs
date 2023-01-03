extern crate drawille;

use drawille::Canvas;
use drawille::PixelColor;
use rand::Rng;
use std::collections::HashSet;
use std::{thread, time};
use termsize;

const RATE: u64 = 100;

// Conway's rules:
// Births: Each dead cell adjacent to exactly three live neighbors will become live in the next generation.
// Death by isolation: Each live cell with one or fewer live neighbors will die in the next generation.
// Death by overcrowding: Each live cell with four or more live neighbors will die in the next generation.
// Survival: Each live cell with either two or three live neighbors will remain alive for the next generation.

fn main() {
    let termsize::Size { rows, cols } = termsize::get().unwrap();
    // let size: u32 = std::cmp::min(rows, cols) as u32;
    let size: u32 = 100;

    let mut canvas = Canvas::new(size, size);

    let mut cells = Cells::new();

    let midpoint = size / 2;
    cells.birth(midpoint, midpoint);
    cells.birth(midpoint, midpoint + 1);
    cells.birth(midpoint + 1, midpoint);
    cells.birth(midpoint + 1, midpoint + 1);
    cells.birth(midpoint + 1, midpoint + 2);
    cells.birth(midpoint + 1, midpoint + 3);
    cells.birth(midpoint + 1, midpoint + 4);

    loop {
        thread::sleep(time::Duration::from_millis(RATE));
        canvas.clear();

        for i in 1..size {
            for j in 1..size {
                // Perform rules on cell
                let num_living_neighbors = cells.num_living_neighbors(i, j);

                // Every living cell with <= 1 neighbor dies
                if num_living_neighbors <= 1 {
                    cells.kill(i, j);
                }

                // Every living cell with >= 4 neighbors dies
                if num_living_neighbors >= 4 {
                    cells.kill(i, j);
                }

                // Each live cell with 2 or 3 neighbors lives -- This is a noop for us

                // Every dead cell with 3 neighbors is born
                if !cells.is_alive(i, j) && num_living_neighbors == 3 {
                    cells.birth(i, j);
                }

                // Draw the cell if it's alive
                if cells.is_alive(i, j) {
                    let color = random_color();
                    canvas.set_colored(j, i, color);
                }
            }
        }

        print!("{}[2J", 27 as char); // Clear the term
        print!("{}", canvas.frame());
    }
}

pub struct Cells {
    living_cells: HashSet<String>,
}

impl Cells {
    pub fn new() -> Cells {
        Cells { living_cells: HashSet::new() }
    }

    pub fn birth(&mut self, i: u32, j: u32) {
        self.living_cells.insert(format_cell_key(i, j));
    }

    pub fn kill(&mut self, i: u32, j: u32) {
        self.living_cells.remove(&format_cell_key(i, j));
    }

    pub fn is_alive(&self, i: u32, j: u32) -> bool {
        self.living_cells.contains(&format_cell_key(i, j))
    }

    pub fn num_living_neighbors(&self, i: u32, j: u32) -> u32 {
        let neighbors = vec![
            format_cell_key(i - 1, j - 1),
            format_cell_key(i - 1, j),
            format_cell_key(i - 1, j + 1),
            format_cell_key(i, j - 1),
            format_cell_key(i, j + 1),
            format_cell_key(i + 1, j - 1),
            format_cell_key(i + 1, j),
            format_cell_key(i + 1, j + 1),
        ];

        let mut num_living_neighbors = 0;

        neighbors.iter().for_each(|key| {
            if self.living_cells.contains(key) {
                num_living_neighbors += 1;
            }
        });

        num_living_neighbors
    }
}

fn format_cell_key(i: u32, j: u32) -> String {
    format!("{i}-{j}")
}

fn random_color() -> PixelColor {
    let rnum = rand::thread_rng().gen_range(1..=11);
    match rnum {
        1 => PixelColor::Blue,
        2 => PixelColor::Cyan,
        3 => PixelColor::BrightBlack,
        4 => PixelColor::Green,
        5 => PixelColor::BrightGreen,
        6 => PixelColor::Yellow,
        7 => PixelColor::Magenta,
        8 => PixelColor::BrightRed,
        9 => PixelColor::BrightYellow,
        10 => PixelColor::BrightBlue,
        11 => PixelColor::BrightCyan,
        _ => PixelColor::Cyan,
    }
}
