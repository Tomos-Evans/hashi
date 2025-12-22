use rand::SeedableRng;
use rand::prelude::*;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HashiError {
    #[error("Invalid grid size")]
    Size,

    #[error("Position out of bounds ({position:?})")]
    OutOfBounds { position: Position },

    #[error("Cannot overwrite existing element at {position:?}")]
    Overwrite { position: Position },

    #[error("Bridges cannot be diagonal")]
    DiagonalBridge,

    #[error("Bridge length cannot be zero")]
    BridgeLengthZero,

    #[error("BridgeLine ({line:?}) is not connected to island at {position:?}")]
    UnconnectedBridge {
        line: BridgeLine,
        position: Position,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum BridgeType {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
enum BridgeDirection {
    Down,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BridgeLine {
    start: Position,
    end: Position,
    direction: BridgeDirection,
}

impl BridgeLine {
    fn new(start: Position, end: Position) -> Result<Self, HashiError> {
        if start.x != end.x && start.y != end.y {
            return Err(HashiError::DiagonalBridge);
        }

        if start == end {
            return Err(HashiError::BridgeLengthZero);
        }

        // order the positions so start is always less than end
        if start.x == end.x {
            // vertical line, order by y
            if start.y > end.y {
                Ok(Self {
                    start: end,
                    end: start,
                    direction: BridgeDirection::Down,
                })
            } else {
                Ok(Self {
                    start,
                    end,
                    direction: BridgeDirection::Down,
                })
            }
        } else {
            // horizontal line, order by x
            if start.x > end.x {
                Ok(Self {
                    start: end,
                    end: start,
                    direction: BridgeDirection::Right,
                })
            } else {
                Ok(Self {
                    start,
                    end,
                    direction: BridgeDirection::Right,
                })
            }
        }
    }

    fn intersects(&self, other: &BridgeLine) -> Option<Position> {
        if self.direction == other.direction {
            // both vertical or both horizontal, cannot intersect
            // If they are overlapping on the same plane, then they would have to cross an island which is handled elsewhere
            return None;
        }

        let (vert, horiz) = if self.direction == BridgeDirection::Down {
            (self, other)
        } else {
            (other, self)
        };

        // if the lines are tip to tip, they do not intersect. This is allowed, they are meeting at an island
        if horiz.start == vert.start
            || horiz.start == vert.end
            || horiz.end == vert.start
            || horiz.end == vert.end
        {
            return None;
        }

        // if the y of the horizontal line is within the vertical line's y range
        if horiz.start.y >= vert.start.y && horiz.start.y <= vert.end.y {
            // if the x of the vertical line is within the horizontal line's x range
            if vert.start.x >= horiz.start.x && vert.start.x <= horiz.end.x {
                return Some(Position {
                    x: vert.start.x,
                    y: horiz.start.y,
                });
            }
        }

        None
    }

    fn crosses(&self, position: Position) -> bool {
        // if the bridge is vertical and the position is on the same x
        if self.start.x == self.end.x && position.x == self.start.x {
            let miny = self.start.y.min(self.end.y);
            let maxy = self.start.y.max(self.end.y);

            if position.y >= miny && position.y <= maxy {
                return true;
            }
        }

        // if the bridge is horizontal and the position is on the same y
        if self.start.y == self.end.y && position.y == self.start.y {
            let minx = self.start.x.min(self.end.x);
            let maxx = self.start.x.max(self.end.x);

            if position.x >= minx && position.x <= maxx {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Island {
    pub required_bridges: u8,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashiGrid {
    pub width: u8,
    pub height: u8,
    pub islands: BTreeMap<Position, Island>,
    bridges: BTreeMap<BridgeLine, BridgeType>,
}

impl HashiGrid {
    fn new(width: u8, height: u8) -> Result<Self, HashiError> {
        if width == 0 || height == 0 {
            return Err(HashiError::Size);
        }
        Ok(Self {
            width,
            height,
            islands: BTreeMap::new(),
            bridges: BTreeMap::new(),
        })
    }

    #[allow(dead_code)]
    pub fn generate(width: u8, height: u8) -> Result<Self, HashiError> {
        // make random number generator. Make a StdRng from the default random source
        let mut rng = rand::rng();
        let rng = rand::rngs::StdRng::from_rng(&mut rng);

        Self::_generate(width, height, rng)
    }

    pub fn generate_with_seed(width: u8, height: u8, seed: u64) -> Result<Self, HashiError> {
        // seed the random number generator
        let rng = rand::rngs::StdRng::seed_from_u64(seed);

        Self::_generate(width, height, rng)
    }

    fn _generate(width: u8, height: u8, mut rng: rand::rngs::StdRng) -> Result<Self, HashiError> {
        // Empty grid
        let mut grid = HashiGrid::new(width, height)?;

        // How many islands?
        // TODO - change based on difficulty
        let num_islands = ((width as u16 * height as u16) / 5).max(5) as u8;

        // place the first island randomly
        let x = rng.random_range(0..width);
        let y = rng.random_range(0..height);
        let position = Position { x, y };
        grid.add_island(position)?;

        let mut max_remaining_iterations = num_islands as usize * 100;

        // place the remaining islands
        while grid.islands.len() < num_islands as usize && max_remaining_iterations > 0 {
            max_remaining_iterations -= 1;

            // pick a random existing island - use index-based selection for determinism
            let island_keys: Vec<Position> = grid.islands.keys().copied().collect();
            let index = rng.random_range(0..island_keys.len());
            let existing_island_pos = island_keys[index];

            // pick a random direction
            let direction = match rng.random_range(0..4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                3 => Direction::Right,
                _ => unreachable!(),
            };

            let proposed_position = match direction {
                Direction::Up => {
                    if existing_island_pos.y == 0 {
                        None
                    } else {
                        Some(Position {
                            x: existing_island_pos.x,
                            y: rng.random_range(0..existing_island_pos.y),
                        })
                    }
                }
                Direction::Down => {
                    if existing_island_pos.y >= height - 1 {
                        None
                    } else {
                        Some(Position {
                            x: existing_island_pos.x,
                            y: rng.random_range((existing_island_pos.y + 1)..height),
                        })
                    }
                }
                Direction::Left => {
                    if existing_island_pos.x == 0 {
                        None
                    } else {
                        Some(Position {
                            x: rng.random_range(0..existing_island_pos.x),
                            y: existing_island_pos.y,
                        })
                    }
                }
                Direction::Right => {
                    if existing_island_pos.x >= width - 1 {
                        None
                    } else {
                        Some(Position {
                            x: rng.random_range((existing_island_pos.x + 1)..width),
                            y: existing_island_pos.y,
                        })
                    }
                }
            };

            let proposed_position = match proposed_position {
                Some(pos) => pos,
                None => continue,
            };

            // speculatively add the island
            match grid.add_island(proposed_position) {
                Ok(()) => {
                    // successfully added island
                    // can we add a bridge?
                    let bridge_line = BridgeLine::new(existing_island_pos, proposed_position)?;
                    match grid.add_bridge(bridge_line) {
                        Ok(_) => {}
                        Err(_) => {
                            // remove the island we just added
                            grid.islands.remove(&proposed_position);
                        }
                    }
                }
                Err(_e) => {
                    // try again
                    continue;
                }
            }
        }

        // todo - create loops

        let chance_of_loop = 0.3; // todo - change based on difficulty
        let island_positions: Vec<Position> = grid.islands.keys().copied().collect();

        for island_pos in island_positions {
            for direction in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                // find nearest island in that direction
                let mut target_island: Option<Position> = None;
                match direction {
                    Direction::Up => {
                        let mut y = island_pos.y;
                        while y > 0 {
                            y -= 1;
                            let pos = Position { x: island_pos.x, y };

                            if grid.islands.contains_key(&pos) {
                                target_island = Some(pos);
                                break;
                            }
                        }
                    }
                    Direction::Down => {
                        let mut y = island_pos.y;
                        while y < height - 1 {
                            y += 1;
                            let pos = Position { x: island_pos.x, y };
                            if grid.islands.contains_key(&pos) {
                                target_island = Some(pos);
                                break;
                            }
                        }
                    }
                    Direction::Left => {
                        let mut x = island_pos.x;
                        while x > 0 {
                            x -= 1;
                            let pos = Position { x, y: island_pos.y };
                            if grid.islands.contains_key(&pos) {
                                target_island = Some(pos);
                                break;
                            }
                        }
                    }
                    Direction::Right => {
                        let mut x = island_pos.x;
                        while x < width - 1 {
                            x += 1;
                            let pos = Position { x, y: island_pos.y };
                            if grid.islands.contains_key(&pos) {
                                target_island = Some(pos);
                                break;
                            }
                        }
                    }
                }

                if let Some(target_pos) = target_island {
                    let bridge_line = BridgeLine::new(island_pos, target_pos)?;
                    if rng.random::<f64>() > chance_of_loop {
                        continue;
                    }
                    match grid.add_bridge(bridge_line) {
                        Ok(_) => {}
                        Err(_) => {
                            // failed to add bridge, ignore
                        }
                    }
                }
            }
        }

        // double some bridges randomly
        let mut bridge_lines: Vec<BridgeLine> = grid.bridges.keys().cloned().collect();
        bridge_lines.shuffle(&mut rng);
        for bridge_line in bridge_lines.iter().take((bridge_lines.len() / 4).max(1)) {
            if let Ok(BridgeType::Double) = grid.add_bridge(*bridge_line) {}
        }

        // count bridges per island
        let island_positions: Vec<Position> = grid.islands.keys().copied().collect();
        for island_pos in island_positions {
            let mut bridge_count = 0;
            for (bridge_line, bridge_type) in &grid.bridges {
                if bridge_line.start == island_pos || bridge_line.end == island_pos {
                    match bridge_type {
                        BridgeType::Single => bridge_count += 1,
                        BridgeType::Double => bridge_count += 2,
                    }
                }
            }
            if let Some(island) = grid.islands.get_mut(&island_pos) {
                island.required_bridges = bridge_count;
            }
        }

        Ok(grid)
    }
    fn add_island(&mut self, position: Position) -> Result<(), HashiError> {
        if position.x >= self.width || position.y >= self.height {
            return Err(HashiError::OutOfBounds { position });
        }

        if self.islands.contains_key(&position) {
            return Err(HashiError::Overwrite { position });
        }

        for &bridge_line in self.bridges.keys() {
            if bridge_line.crosses(position) {
                return Err(HashiError::Overwrite { position });
            }
        }

        self.islands.insert(
            position,
            Island {
                required_bridges: 0,
            },
        );
        Ok(())
    }

    fn add_bridge(&mut self, bridge: BridgeLine) -> Result<BridgeType, HashiError> {
        // if the bridge already exists and its a single, then its already been validated, just upgrade it
        if let Some(bridge_type) = self.bridges.get(&bridge) {
            if *bridge_type == BridgeType::Double {
                // already a double, cannot add more
                return Err(HashiError::Overwrite {
                    position: bridge.start,
                });
            }
            self.bridges.insert(bridge, BridgeType::Double);
            return Ok(BridgeType::Double);
        }

        // The bridge is new, so validate it

        // Check if both ends are islands
        if !self.islands.contains_key(&bridge.start) {
            return Err(HashiError::UnconnectedBridge {
                line: bridge,
                position: bridge.start,
            });
        }
        if !self.islands.contains_key(&bridge.end) {
            return Err(HashiError::UnconnectedBridge {
                line: bridge,
                position: bridge.end,
            });
        }

        // check that the bridge does not cross any existing islands other than the two endpoints it is between
        for &island_pos in self.islands.keys() {
            if island_pos != bridge.start && island_pos != bridge.end && bridge.crosses(island_pos)
            {
                return Err(HashiError::Overwrite {
                    position: island_pos,
                });
            }
        }

        // check that the bridge does not cross any existing bridges
        for &existing_bridge in self.bridges.keys() {
            if let Some(collision) = bridge.intersects(&existing_bridge) {
                return Err(HashiError::Overwrite {
                    position: collision,
                });
            }
        }
        // all checks passed, add the bridge as a single
        self.bridges.insert(bridge, BridgeType::Single);

        Ok(BridgeType::Single)
    }
}

impl std::fmt::Display for HashiGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Top header
        write!(f, "   |",)?;
        for x in 0..self.width {
            write!(f, "{:3}  ", x)?;
        }
        writeln!(f)?;

        // Separator
        write!(f, "___|",)?;
        for _ in 0..self.width {
            write!(f, "_____")?;
        }
        writeln!(f)?;

        // each row
        for y in 0..self.height {
            // Start with the row index and then separator
            write!(f, "{:3}|", y)?;

            // each column
            for x in 0..self.width {
                // Is there an island here?
                if let Some(island) = self.islands.get(&Position { x, y }) {
                    write!(f, " ({}) ", island.required_bridges)?;
                    continue;
                }
                // Is there a bridge  here?
                match self
                    .bridges
                    .iter()
                    .find(|(line, _)| line.crosses(Position { x, y }))
                {
                    Some((line, bridge_type)) => match (line.direction, bridge_type) {
                        (BridgeDirection::Down, BridgeType::Single) => write!(f, "  |  ")?,
                        (BridgeDirection::Down, BridgeType::Double) => write!(f, " ||  ")?,
                        (BridgeDirection::Right, BridgeType::Single) => write!(f, "-----")?,
                        (BridgeDirection::Right, BridgeType::Double) => write!(f, "=====")?,
                    },
                    None => write!(f, "     ")?,
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
    fn test_line_crosses() {
        // vertical bridgeline (check both directions)
        let forward = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        let backwards = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();

        for bridge in [forward, backwards] {
            // same x within y range
            assert!(bridge.crosses(Position { x: 2, y: 3 }));
            assert!(bridge.crosses(Position { x: 2, y: 2 }));
            assert!(bridge.crosses(Position { x: 2, y: 5 }));

            // different x for vert bridge
            assert!(!bridge.crosses(Position { x: 1, y: 3 }));
            assert!(!bridge.crosses(Position { x: 3, y: 3 }));

            // same x but outside y range
            assert!(!bridge.crosses(Position { x: 2, y: 1 }));
            assert!(!bridge.crosses(Position { x: 2, y: 6 }));
        }

        let forwards = BridgeLine::new(Position { x: 2, y: 5 }, Position { x: 2, y: 2 }).unwrap();
        let backwards = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();

        for bridge in [forwards, backwards] {
            // same x within y range
            assert!(bridge.crosses(Position { x: 2, y: 3 }));
            assert!(bridge.crosses(Position { x: 2, y: 2 }));
            assert!(bridge.crosses(Position { x: 2, y: 5 }));

            // different x for vert bridge
            assert!(!bridge.crosses(Position { x: 1, y: 3 }));
            assert!(!bridge.crosses(Position { x: 3, y: 3 }));

            // same x but outside y range
            assert!(!bridge.crosses(Position { x: 2, y: 1 }));
            assert!(!bridge.crosses(Position { x: 2, y: 6 }));
        }
    }

    #[test]
    fn test_diagonal_bridge_error() {
        let result = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 2, y: 2 });
        assert_eq!(result.unwrap_err(), HashiError::DiagonalBridge);
        let result = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 3, y: 0 });
        assert_eq!(result.unwrap_err(), HashiError::DiagonalBridge);
    }

    #[test]
    fn test_bridge_direction_independence() {
        let bridge1 = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 1, y: 3 }).unwrap();
        let bridge2 = BridgeLine::new(Position { x: 1, y: 3 }, Position { x: 1, y: 1 }).unwrap();
        assert_eq!(bridge1, bridge2);

        let bridge3 = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 5, y: 2 }).unwrap();
        let bridge4 = BridgeLine::new(Position { x: 5, y: 2 }, Position { x: 2, y: 2 }).unwrap();
        assert_eq!(bridge3, bridge4);
    }

    #[test]
    fn test_zero_length_bridge_error() {
        let result = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 1, y: 1 });
        assert_eq!(result.unwrap_err(), HashiError::BridgeLengthZero);
    }

    #[test]
    fn test_unconnected_bridge() {
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 1 }).unwrap();

        let bridge = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 1, y: 3 }).unwrap();
        let result = grid.add_bridge(bridge);
        assert_eq!(
            result.unwrap_err(),
            HashiError::UnconnectedBridge {
                line: bridge,
                position: Position { x: 1, y: 3 }
            }
        );

        let bridge = BridgeLine::new(Position { x: 1, y: 3 }, Position { x: 1, y: 1 }).unwrap();
        let result = grid.add_bridge(bridge);
        assert_eq!(
            result.unwrap_err(),
            HashiError::UnconnectedBridge {
                line: bridge,
                position: Position { x: 1, y: 3 }
            }
        );
    }

    #[test]
    fn test_overwrite_island_error() {
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 1 }).unwrap();
        grid.add_island(Position { x: 1, y: 3 }).unwrap();
        let bridge = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 1, y: 3 }).unwrap();

        assert!(grid.add_bridge(bridge).unwrap() == BridgeType::Single);
    }
    #[test]
    fn test_bridges_cant_intersect() {
        let mut grid = HashiGrid::new(5, 10).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        grid.add_island(Position { x: 2, y: 5 }).unwrap();
        grid.add_island(Position { x: 1, y: 3 }).unwrap();
        grid.add_island(Position { x: 4, y: 3 }).unwrap();

        let vertical = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        assert!(grid.add_bridge(vertical).unwrap() == BridgeType::Single);

        let horizontal = BridgeLine::new(Position { x: 1, y: 3 }, Position { x: 4, y: 3 }).unwrap();
        let result = grid.add_bridge(horizontal);
        assert_eq!(
            result.unwrap_err(),
            HashiError::Overwrite {
                position: Position { x: 2, y: 3 }
            }
        );
    }

    #[test]
    fn test_lines_intersect() {
        let vertical = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        let horizontal = BridgeLine::new(Position { x: 1, y: 3 }, Position { x: 4, y: 3 }).unwrap();

        assert_eq!(
            vertical.intersects(&horizontal),
            Some(Position { x: 2, y: 3 })
        );
        assert_eq!(
            horizontal.intersects(&vertical),
            Some(Position { x: 2, y: 3 })
        );

        let no_intersection =
            BridgeLine::new(Position { x: 3, y: 6 }, Position { x: 6, y: 6 }).unwrap();
        assert_eq!(vertical.intersects(&no_intersection), None);
        assert_eq!(no_intersection.intersects(&vertical), None);
    }

    #[test]
    fn can_have_four_bridges_to_same_island() {
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        grid.add_island(Position { x: 2, y: 0 }).unwrap();
        grid.add_island(Position { x: 2, y: 4 }).unwrap();
        grid.add_island(Position { x: 0, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        let up = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 0 }).unwrap();
        let down = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 4 }).unwrap();
        let left = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 0, y: 2 }).unwrap();
        let right = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 4, y: 2 }).unwrap();

        assert!(grid.add_bridge(up).unwrap() == BridgeType::Single);
        assert!(grid.add_bridge(down).unwrap() == BridgeType::Single);
        assert!(grid.add_bridge(left).unwrap() == BridgeType::Single);
        assert!(grid.add_bridge(right).unwrap() == BridgeType::Single);
        assert!(grid.add_bridge(up).unwrap() == BridgeType::Double);
        assert!(grid.add_bridge(down).unwrap() == BridgeType::Double);
        assert!(grid.add_bridge(left).unwrap() == BridgeType::Double);
        assert!(grid.add_bridge(right).unwrap() == BridgeType::Double);
    }

    #[test]
    fn test_same_seed_produces_same_grid() {
        let seed = 12345;
        let grid1 = HashiGrid::generate_with_seed(10, 10, seed).unwrap();
        let grid2 = HashiGrid::generate_with_seed(10, 10, seed).unwrap();
        let grid3 = HashiGrid::generate_with_seed(10, 10, seed + 1).unwrap();

        assert_eq!(grid1, grid2);
        assert_ne!(grid1, grid3);
    }
}
