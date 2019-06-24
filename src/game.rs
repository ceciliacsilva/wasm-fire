use rand::prelude::*;

use crate::cell::*;

#[derive(Clone)]
pub struct Universe {
    width: u32,
    height: u32,
    pub cells: Vec<Cell>,
    pub time_cells: Vec<u8>,
    pub game_config: GameConfig,
}

#[derive(Clone)]
pub struct GameConfig {
    to_burn: f32,
    time_to_burn: u8,
    num_focus: u8,
}

pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB { r, g, b}
    }

    pub fn to_float(&self) -> (f32, f32, f32) {
        let new_r = self.r as f32 / 255.0;
        let new_g = self.g as f32 / 255.0;
        let new_b = self.b as f32 / 255.0;

        (new_r, new_g, new_b)
    }

    pub fn time_to_pos(time: u8, len: usize, game_config: &GameConfig) -> usize {
        let pos = time as usize * len / game_config.time_to_burn as usize;
        pos
    }
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    pub fn burning_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 {
            self.height - 1
        } else {
            row - 1
        };

        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw].is_burning() as u8;

        let n = self.get_index(north, column);
        count += self.cells[n].is_burning() as u8;

        let ne = self.get_index(north, east);
        count += self.cells[ne].is_burning() as u8;

        let w = self.get_index(row, west);
        count += self.cells[w].is_burning() as u8;

        let e = self.get_index(row, east);
        count += self.cells[e].is_burning() as u8;

        let sw = self.get_index(south, west);
        count += self.cells[sw].is_burning() as u8;

        let s = self.get_index(south, column);
        count += self.cells[s].is_burning() as u8;

        let se = self.get_index(south, east);
        count += self.cells[se].is_burning() as u8;

        count
    }
}

impl Universe {
    pub fn tick (&mut self) -> bool {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];

                if cell == Cell::Alive {
                    let burning_neighbor = self.burning_neighbor_count(row, col);

                    if burning_neighbor > 0 {
                        let mut rng = rand::thread_rng();
                        let prob_burn: f32 = rng.gen();

                        if prob_burn < self.game_config.to_burn {
                            next[idx].live_to_burn();
                        }
                    }
                }

                if cell == Cell::Burning {
                    if self.time_cells[idx] < self.game_config.time_to_burn {
                        self.time_cells[idx] += 1;
                    }
                    else {
                        next[idx].burn_to_dead();
                    }
                }
            }
        }

        if self.cells == next {
            return true;
        }

        self.cells = next;
        false
    }

    pub fn new(width: u32, height: u32, num_focus: u8) -> Universe {
        crate::utils::set_panic_hook();

        let mut rng = rand::thread_rng();
        let mut pos = Vec::new();

        for _i in 0..num_focus {
            let n: u32 = rng.gen_range(0, width * height);
            pos.push(n);
        }

        let cells = (0..width * height)
            .map(|i| {
                if pos.contains(&i) {
                    Cell::Burning
                } else {
                    Cell::Alive
                }
            })
            .collect();

        let time_cells = (0..width * height)
            .map(|_i| {
                0
            })
            .collect();

        let game_config = GameConfig {
            to_burn: 0.3,
            time_to_burn: 10,
            num_focus,
        };

        Universe {
            width,
            height,
            cells,
            time_cells,
            game_config,
        }
    }
}
