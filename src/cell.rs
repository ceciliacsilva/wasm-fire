#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Dead,
    Burning,
    Alive,
}

impl Cell {
    pub fn live_to_burn(&mut self) {
        if let Cell::Alive = *self {
            *self = Cell::Burning
        };
    }

    pub fn burn_to_dead(&mut self) {
        if let Cell::Burning = *self {
            *self = Cell::Dead
        };
    }

    pub fn is_burning(&self) -> u8 {
        match *self {
            Cell::Dead => 0,
            Cell::Burning => 1,
            Cell::Alive => 0,
        }
    }
}
