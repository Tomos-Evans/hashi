use thiserror::{Error};


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Island,
    BridgeH,
    BridgeV,
    DoubleBridgeH,
    DoubleBridgeV,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coordinate {
    pub x: u8,
    pub y: u8,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HashiError {
    #[error("Invalid grid size")]
    Size,
    #[error("Placing in out of bounds cell at ({}, {})", coordinate.x, coordinate.y)]
    OutOfBounds { coordinate: Coordinate },
    #[error("Cannot overwrite cell at ({}, {})", coordinate.x, coordinate.y)]
    Overwrite { coordinate: Coordinate },
    #[error("Bridges cannot be diagonal")]
    DiagonalBridge,
}


#[derive(Clone, Debug)]
pub struct HashiGrid {
    width: u8,
    height: u8,
    cells: Vec<Vec<Cell>>
}



impl HashiGrid {


    pub fn new(width: u8, height: u8) -> Result<Self, HashiError> {
        if width == 0 || height == 0 {
            return Err(HashiError::Size);
        }

        Ok(Self {
            width,
            height,
            cells: vec![vec![Cell::Empty; width as usize]; height as usize],
        })
    }


    pub fn add_island(&mut self, coordinate: Coordinate) -> Result<(), HashiError> {
        if coordinate.x >= self.width || coordinate.y >= self.height {
            return Err(HashiError::OutOfBounds { coordinate });
        }

        if let Some(Cell::Empty) = self.cells[coordinate.y as usize].get(coordinate.x as usize) {
            self.cells[coordinate.y as usize][coordinate.x as usize] = Cell::Island;
            Ok(())
        } else {
            return Err(HashiError::Overwrite { coordinate });
        }
    }

    pub fn get_cell(&self, coordinate: &Coordinate) -> Result<&Cell, HashiError> {
        if coordinate.x >= self.width || coordinate.y >= self.height {
            return Err(HashiError::OutOfBounds { coordinate: coordinate.clone() });
        }
        
        Ok(&self.cells[coordinate.y as usize][coordinate.x as usize])
    }

    pub fn bridge_islands(&mut self, from: Coordinate, to: Coordinate) -> Result<(), HashiError> {   
        if from.x != to.x && from.y != to.y {
            return Err(HashiError::DiagonalBridge);
        }

        if from.x == to.x && from.y == to.y {
            return Err(HashiError::Overwrite { coordinate: from });
        }

        // if let Some(Cell::Island)

        // check that both from and to are islands
        match self.get_cell(&from)? {
            Cell::Island => {},
            _ => return Err(HashiError::Overwrite { coordinate: from }),
        }
        match self.get_cell(&to)? {
            Cell::Island => {},
            _ => return Err(HashiError::Overwrite { coordinate: to }),
        }

        if from.y == to.y {
            // horizontal bridge
            let y = from.y as usize;
            let (start_x, end_x) = if from.x < to.x {
                (from.x as usize + 1, to.x as usize)
            } else {
                (to.x as usize + 1, from.x as usize)
            };
            let all_empty    = (start_x..end_x).all(|x| matches!(self.cells[y][x], Cell::Empty));
            let all_h_bridge = (start_x..end_x).all(|x| matches!(self.cells[y][x], Cell::BridgeH));
            if !all_empty && !all_h_bridge {
                return Err(HashiError::Overwrite { coordinate: Coordinate { x: start_x as u8, y: from.y } });
            }
            for x in start_x..end_x {
                if all_h_bridge {
                    self.cells[y][x] = Cell::DoubleBridgeH;
                } else {    
                    self.cells[y][x] = Cell::BridgeH;
                }
            }

            return Ok(());
        }

        if from.x == to.x {
            // vertical bridge
            let x = from.x as usize;
            let (start_y, end_y) = if from.y < to.y {
                (from.y as usize + 1, to.y as usize)
            } else {
                (to.y as usize + 1, from.y as usize)
            };
            let all_empty = (start_y..end_y).all(|y| matches!(self.cells[y][x], Cell::Empty));
            let all_v_bridge = (start_y..end_y).all(|y| matches!(self.cells[y][x], Cell::BridgeV));
            if !all_empty && !all_v_bridge {
                return Err(HashiError::Overwrite { coordinate: Coordinate { x: from.x, y: start_y as u8 } });
            }
            for y in start_y..end_y {
                if all_v_bridge {
                    self.cells[y][x] = Cell::DoubleBridgeV;
                } else {
                    self.cells[y][x] = Cell::BridgeV;
                }
            }

            return Ok(());
        }


        Ok(())


    }

    

    
}



impl std::fmt::Display for HashiGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        write!(f, "   |", )?;
        for x in 0..self.width {
            write!(f, "{:3}  ", x)?;
        }
        writeln!(f)?;


        write!(f, "___|", )?;
        for _ in 0..self.width {
            write!(f, "_____")?;
        }
        writeln!(f)?;


        for y in 0..self.height {
            write!(f, "{:3}|", y)?;
            for x in 0..self.width {
                let cell = &self.cells[y as usize][x as usize];
                match cell {
                    Cell::Empty => write!(f, "     ")?,
                    Cell::Island => write!(f, "  X  ")?,
                    Cell::BridgeH => write!(f, "-----")?,
                    Cell::BridgeV => write!(f, "  |  ")?,
                    Cell::DoubleBridgeH => write!(f, "=====")?,
                    Cell::DoubleBridgeV => write!(f, " ||  ")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())        
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_sizes() {
        assert_eq!(HashiGrid::new(0, 5).err(), Some(HashiError::Size));
        assert_eq!(HashiGrid::new(5, 0).err(), Some(HashiError::Size));
        assert!(HashiGrid::new(5, 5).is_ok());
    }

    #[test]
    fn test_add_island() {
        let mut grid = HashiGrid::new(5, 10).unwrap();
        // within bounds and empty
        assert!(grid.add_island(Coordinate { x: 2, y: 2 }).is_ok());

        // overwrite existing island
        assert_eq!(grid.add_island(Coordinate { x: 2, y: 2 }).err(), Some(HashiError::Overwrite { coordinate: Coordinate { x: 2, y: 2 } }));

        // out of bounds in y
        assert_eq!(grid.add_island(Coordinate { x: 2, y: 10 }).err(), Some(HashiError::OutOfBounds { coordinate: Coordinate { x: 2, y: 10 } }));

        // out of bounds in x
        assert_eq!(grid.add_island(Coordinate { x: 5, y: 2 }).err(), Some(HashiError::OutOfBounds { coordinate: Coordinate { x: 5, y: 2 } }));
    }


    #[test]
    fn test_bridge_islands() {
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Coordinate { x: 1, y: 1 }).unwrap();
        grid.add_island(Coordinate { x: 1, y: 4 }).unwrap();
        grid.add_island(Coordinate { x: 4, y: 1 }).unwrap();

        // valid vertical bridge
        assert!(grid.bridge_islands(Coordinate { x: 1, y: 1 }, Coordinate { x: 1, y: 4 }).is_ok());

        // valid horizontal bridge
        assert!(grid.bridge_islands(Coordinate { x: 1, y: 1 }, Coordinate { x: 4, y: 1 }).is_ok());

        // diagonal bridge
        assert_eq!(grid.bridge_islands(Coordinate { x: 1, y: 1 }, Coordinate { x: 4, y: 4 }).err(), Some(HashiError::DiagonalBridge));

        // bridge to non-island
        assert_eq!(grid.bridge_islands(Coordinate { x: 1, y: 1 }, Coordinate { x: 2, y: 1 }).err(), Some(HashiError::Overwrite { coordinate: Coordinate { x: 2, y: 1 } }));

        // double bridge
        assert!(grid.bridge_islands(Coordinate { x: 1, y: 1 }, Coordinate { x: 1, y: 4 }).is_ok());
        assert!(grid.bridge_islands(Coordinate { x: 1, y: 1 }, Coordinate { x: 4, y: 1 }).is_ok());

        // check cell types
        assert_eq!(grid.get_cell(&Coordinate { x: 1, y: 1 }).unwrap(), &Cell::Island);
        assert_eq!(grid.get_cell(&Coordinate { x: 1, y: 2 }).unwrap(), &Cell::DoubleBridgeV);

        // cannot tripple bridge
        assert_eq!(grid.bridge_islands(Coordinate { x: 1, y: 1 }, Coordinate { x: 1, y: 4 }).err(), Some(HashiError::Overwrite { coordinate: Coordinate { x: 1, y: 2 } }));
    }
}