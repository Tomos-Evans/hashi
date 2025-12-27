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
pub enum BridgeType {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum BridgeDirection {
    Down,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BridgeLine {
    pub start: Position,
    pub end: Position,
    pub direction: BridgeDirection,
}

impl BridgeLine {
    pub fn new(start: Position, end: Position) -> Result<Self, HashiError> {
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
    pub bridges: BTreeMap<BridgeLine, BridgeType>,
}

impl HashiGrid {
    pub fn placeholder() -> Self {
        Self {
            width: 0,
            height: 0,
            islands: BTreeMap::new(),
            bridges: BTreeMap::new(),
        }
    }
    pub fn new(width: u8, height: u8) -> Result<Self, HashiError> {
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
        let num_islands = ((width as u16 * height as u16) / 5).max(8) as u8;

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
                _ => Direction::Right, // This should be unreachable, but in the event its not, better to favor Right than to panic
                                       // I am not using `choose` with a custom impl of sample as you cannot pass rng, would not be deterministic
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

        let chance_of_loop = 0.6; // todo - change based on difficulty
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
                    if rng.random::<f64>() > chance_of_loop {
                        continue;
                    }
                    // At this point if this fails it does not matter, it just means it would have crossed another bridge.
                    // Safe to ignore the error
                    let _ = grid.add_bridge(BridgeLine::new(island_pos, target_pos)?);
                }
            }
        }

        // double some bridges randomly
        let bridge_lines_to_double: Vec<BridgeLine> = grid
            .bridges
            .iter()
            .filter_map(|(bridge_line, bridge_type)| {
                if *bridge_type == BridgeType::Single && rng.random::<f64>() < 0.3 {
                    Some(*bridge_line)
                } else {
                    None
                }
            })
            .collect();

        for bridge_line in bridge_lines_to_double {
            let _ = grid.add_bridge(bridge_line);
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

    fn can_add_island(&self, position: Position) -> Result<(), HashiError> {
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

        Ok(())
    }

    fn add_island(&mut self, position: Position) -> Result<(), HashiError> {
        self.can_add_island(position)?;

        self.islands.insert(
            position,
            Island {
                required_bridges: 0,
            },
        );

        Ok(())
    }

    fn count_brdges_ending_at(&self, position: Position) -> u8 {
        let mut count = 0;

        for (bridge_line, bridge_type) in self.bridges_ending_at(position) {
            if bridge_line.start == position || bridge_line.end == position {
                match bridge_type {
                    BridgeType::Single => count += 1,
                    BridgeType::Double => count += 2,
                }
            }
        }

        count
    }

    fn bridges_ending_at(&self, position: Position) -> Vec<(&BridgeLine, &BridgeType)> {
        let mut result = Vec::new();

        for (bridge_line, bridge_type) in &self.bridges {
            if bridge_line.start == position || bridge_line.end == position {
                result.push((bridge_line, bridge_type));
            }
        }

        result
    }

    fn can_bridge(&self, bridge: BridgeLine) -> Result<BridgeType, HashiError> {
        match self.bridges.get(&bridge) {
            Some(BridgeType::Double) => {
                // already a double, cannot add more
                return Err(HashiError::Overwrite {
                    position: bridge.start,
                });
            }
            Some(BridgeType::Single) => {
                // Check if the islands have capacity for another bridge
                for end in [bridge.start, bridge.end] {
                    let island = self.islands.get(&end).unwrap(); // safe unwrap, validated when bridge was first added
                    let existing_bridges = self.count_brdges_ending_at(end);
                    if island.required_bridges != 0
                        && island.required_bridges < existing_bridges + 1
                    {
                        return Err(HashiError::Overwrite { position: end });
                    }
                }

                // already a single, can upgrade to double. No need to validate it again
                return Ok(BridgeType::Double);
            }
            None => {
                // does not exist yet, proceed with validation

                // Check that both ends of the bridgeline are connected to islands
                for end in [bridge.start, bridge.end] {
                    if !self.islands.contains_key(&end) {
                        return Err(HashiError::UnconnectedBridge {
                            line: bridge,
                            position: end,
                        });
                    }

                    // Check that the islands have capacity for another bridge
                    let island = self.islands.get(&end).unwrap(); // safe unwrap, validated when bridge was first added
                    let existing_bridges = self.count_brdges_ending_at(end);
                    if island.required_bridges != 0
                        && island.required_bridges < existing_bridges + 1
                    {
                        return Err(HashiError::Overwrite { position: end });
                    }
                }

                // check that the bridge does not cross any existing islands other than the two endpoints it is between
                for &island_pos in self.islands.keys() {
                    if island_pos != bridge.start
                        && island_pos != bridge.end
                        && bridge.crosses(island_pos)
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
            }
        };

        // All checks passed, can add single bridge
        Ok(BridgeType::Single)
    }

    pub fn add_bridge(&mut self, bridge: BridgeLine) -> Result<BridgeType, HashiError> {
        let suitable_bridge_type = self.can_bridge(bridge)?;
        self.bridges.insert(bridge, suitable_bridge_type);
        Ok(suitable_bridge_type)
    }

    pub fn wipe_bridges(mut self) -> Self {
        self.bridges.clear();
        self
    }

    pub fn is_complete(&self) -> bool {
        // No island should have no bridges, at this point. If it does, something is wrong and we should errror.
        for island in self.islands.values() {
            if island.required_bridges == 0 {
                return false;
            }
        }

        // Run through each bridge and check that it has the correct number of connections
        for (island_pos, island) in &self.islands {
            let mut bridge_count = 0;
            for (bridge_line, bridge_type) in &self.bridges {
                if bridge_line.start == *island_pos || bridge_line.end == *island_pos {
                    match bridge_type {
                        BridgeType::Single => bridge_count += 1,
                        BridgeType::Double => bridge_count += 2,
                    }
                }
            }

            if bridge_count != island.required_bridges {
                return false;
            }
        }

        true
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

    // ============================================================================
    // GRID CREATION AND INITIALIZATION TESTS
    // ============================================================================

    #[test]
    fn test_grid_creation_valid() {
        // Test: Creating a grid with valid dimensions should succeed
        let grid = HashiGrid::new(5, 5);
        assert!(grid.is_ok());
        let grid = grid.unwrap();
        assert_eq!(grid.width, 5);
        assert_eq!(grid.height, 5);
        assert!(grid.islands.is_empty());
        assert!(grid.bridges.is_empty());
    }

    #[test]
    fn test_grid_creation_zero_width() {
        // Test: A grid with zero width should fail
        let result = HashiGrid::new(0, 5);
        assert_eq!(result.unwrap_err(), HashiError::Size);
    }

    #[test]
    fn test_grid_creation_zero_height() {
        // Test: A grid with zero height should fail
        let result = HashiGrid::new(5, 0);
        assert_eq!(result.unwrap_err(), HashiError::Size);
    }

    #[test]
    fn test_grid_creation_zero_both() {
        // Test: A grid with both zero dimensions should fail
        let result = HashiGrid::new(0, 0);
        assert_eq!(result.unwrap_err(), HashiError::Size);
    }

    #[test]
    fn test_placeholder_grid() {
        // Test: Placeholder grids should have zero dimensions and no elements
        let grid = HashiGrid::placeholder();
        assert_eq!(grid.width, 0);
        assert_eq!(grid.height, 0);
        assert!(grid.islands.is_empty());
        assert!(grid.bridges.is_empty());
    }

    // ============================================================================
    // POSITION AND BRIDGE LINE CONSTRUCTION TESTS
    // ============================================================================

    #[test]
    fn test_bridge_line_vertical_forward() {
        // Test: Creating a vertical bridge from top to bottom should normalize correctly
        let bridge = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        assert_eq!(bridge.start, Position { x: 2, y: 2 });
        assert_eq!(bridge.end, Position { x: 2, y: 5 });
        assert_eq!(bridge.direction, BridgeDirection::Down);
    }

    #[test]
    fn test_bridge_line_vertical_backward() {
        // Test: Creating a vertical bridge from bottom to top should normalize to down direction
        // with start.y < end.y (internally normalized)
        let bridge = BridgeLine::new(Position { x: 2, y: 5 }, Position { x: 2, y: 2 }).unwrap();
        assert_eq!(bridge.start, Position { x: 2, y: 2 });
        assert_eq!(bridge.end, Position { x: 2, y: 5 });
        assert_eq!(bridge.direction, BridgeDirection::Down);
    }

    #[test]
    fn test_bridge_line_horizontal_forward() {
        // Test: Creating a horizontal bridge from left to right should normalize correctly
        let bridge = BridgeLine::new(Position { x: 2, y: 3 }, Position { x: 5, y: 3 }).unwrap();
        assert_eq!(bridge.start, Position { x: 2, y: 3 });
        assert_eq!(bridge.end, Position { x: 5, y: 3 });
        assert_eq!(bridge.direction, BridgeDirection::Right);
    }

    #[test]
    fn test_bridge_line_horizontal_backward() {
        // Test: Creating a horizontal bridge from right to left should normalize to right direction
        // with start.x < end.x (internally normalized)
        let bridge = BridgeLine::new(Position { x: 5, y: 3 }, Position { x: 2, y: 3 }).unwrap();
        assert_eq!(bridge.start, Position { x: 2, y: 3 });
        assert_eq!(bridge.end, Position { x: 5, y: 3 });
        assert_eq!(bridge.direction, BridgeDirection::Right);
    }

    #[test]
    fn test_bridge_diagonal_rejected() {
        // Test: Bridges cannot be diagonal - both x and y differ
        let result = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 2, y: 2 });
        assert_eq!(result.unwrap_err(), HashiError::DiagonalBridge);
    }

    #[test]
    fn test_bridge_diagonal_rejected_other_direction() {
        // Test: Diagonal bridges rejected in any direction
        let result = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 3, y: 0 });
        assert_eq!(result.unwrap_err(), HashiError::DiagonalBridge);
    }

    #[test]
    fn test_bridge_zero_length_rejected() {
        // Test: A bridge cannot start and end at the same position
        let result = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 1, y: 1 });
        assert_eq!(result.unwrap_err(), HashiError::BridgeLengthZero);
    }

    // ============================================================================
    // BRIDGE CROSSING LOGIC TESTS
    // ============================================================================

    #[test]
    fn test_bridge_crosses_vertical_line_within_range() {
        // Test: A position on a vertical bridge within its y-range should be detected as crossed
        let bridge = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        assert!(bridge.crosses(Position { x: 2, y: 3 }));
        assert!(bridge.crosses(Position { x: 2, y: 4 }));
    }

    #[test]
    fn test_bridge_crosses_vertical_line_at_endpoints() {
        // Test: The endpoints of a bridge should report as crossed by that bridge
        let bridge = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        assert!(bridge.crosses(Position { x: 2, y: 2 }));
        assert!(bridge.crosses(Position { x: 2, y: 5 }));
    }

    #[test]
    fn test_bridge_crosses_vertical_line_outside_range() {
        // Test: A position outside the bridge's y-range should not be crossed, even if on same x
        let bridge = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        assert!(!bridge.crosses(Position { x: 2, y: 1 }));
        assert!(!bridge.crosses(Position { x: 2, y: 6 }));
    }

    #[test]
    fn test_bridge_crosses_vertical_line_different_x() {
        // Test: A position with different x should not be crossed by a vertical bridge
        let bridge = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        assert!(!bridge.crosses(Position { x: 1, y: 3 }));
        assert!(!bridge.crosses(Position { x: 3, y: 3 }));
    }

    #[test]
    fn test_bridge_crosses_horizontal_line_within_range() {
        // Test: A position on a horizontal bridge within its x-range should be detected as crossed
        let bridge = BridgeLine::new(Position { x: 2, y: 3 }, Position { x: 5, y: 3 }).unwrap();
        assert!(bridge.crosses(Position { x: 3, y: 3 }));
        assert!(bridge.crosses(Position { x: 4, y: 3 }));
    }

    #[test]
    fn test_bridge_crosses_horizontal_line_at_endpoints() {
        // Test: The endpoints of a horizontal bridge should report as crossed
        let bridge = BridgeLine::new(Position { x: 2, y: 3 }, Position { x: 5, y: 3 }).unwrap();
        assert!(bridge.crosses(Position { x: 2, y: 3 }));
        assert!(bridge.crosses(Position { x: 5, y: 3 }));
    }

    #[test]
    fn test_bridge_crosses_horizontal_line_outside_range() {
        // Test: A position outside the bridge's x-range should not be crossed, even if on same y
        let bridge = BridgeLine::new(Position { x: 2, y: 3 }, Position { x: 5, y: 3 }).unwrap();
        assert!(!bridge.crosses(Position { x: 1, y: 3 }));
        assert!(!bridge.crosses(Position { x: 6, y: 3 }));
    }

    #[test]
    fn test_bridge_crosses_horizontal_line_different_y() {
        // Test: A position with different y should not be crossed by a horizontal bridge
        let bridge = BridgeLine::new(Position { x: 2, y: 3 }, Position { x: 5, y: 3 }).unwrap();
        assert!(!bridge.crosses(Position { x: 3, y: 2 }));
        assert!(!bridge.crosses(Position { x: 3, y: 4 }));
    }

    // ============================================================================
    // BRIDGE INTERSECTION LOGIC TESTS
    // ============================================================================

    #[test]
    fn test_bridge_intersects_perpendicular_lines() {
        // Test: A vertical and horizontal line crossing should detect intersection
        let vertical = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        let horizontal = BridgeLine::new(Position { x: 1, y: 3 }, Position { x: 4, y: 3 }).unwrap();

        // Both directions should detect the same intersection point
        assert_eq!(
            vertical.intersects(&horizontal),
            Some(Position { x: 2, y: 3 })
        );
        assert_eq!(
            horizontal.intersects(&vertical),
            Some(Position { x: 2, y: 3 })
        );
    }

    #[test]
    fn test_bridge_intersects_parallel_vertical_lines() {
        // Test: Two vertical (parallel) lines should not intersect
        let line1 = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        let line2 = BridgeLine::new(Position { x: 3, y: 1 }, Position { x: 3, y: 6 }).unwrap();

        assert_eq!(line1.intersects(&line2), None);
        assert_eq!(line2.intersects(&line1), None);
    }

    #[test]
    fn test_bridge_intersects_parallel_horizontal_lines() {
        // Test: Two horizontal (parallel) lines should not intersect
        let line1 = BridgeLine::new(Position { x: 1, y: 3 }, Position { x: 4, y: 3 }).unwrap();
        let line2 = BridgeLine::new(Position { x: 0, y: 4 }, Position { x: 5, y: 4 }).unwrap();

        assert_eq!(line1.intersects(&line2), None);
        assert_eq!(line2.intersects(&line1), None);
    }

    #[test]
    fn test_bridge_intersects_endpoint_meeting() {
        // Test: When lines meet at endpoints, it's not considered an intersection (island meeting point)
        let vertical = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        let horizontal = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 4, y: 2 }).unwrap();

        // Lines meet at (2,2), which is an endpoint for both, so no intersection reported
        assert_eq!(vertical.intersects(&horizontal), None);
    }

    #[test]
    fn test_bridge_intersects_separated_perpendicular_lines() {
        // Test: Two perpendicular lines that don't actually cross should return None
        let vertical = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        let horizontal = BridgeLine::new(Position { x: 3, y: 6 }, Position { x: 6, y: 6 }).unwrap();

        assert_eq!(vertical.intersects(&horizontal), None);
    }

    #[test]
    fn test_bridge_line_equality_is_direction_independent() {
        // Test: Two bridges with same endpoints are equal regardless of construction direction
        let bridge1 = BridgeLine::new(Position { x: 1, y: 1 }, Position { x: 1, y: 3 }).unwrap();
        let bridge2 = BridgeLine::new(Position { x: 1, y: 3 }, Position { x: 1, y: 1 }).unwrap();
        assert_eq!(bridge1, bridge2);

        let bridge3 = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 5, y: 2 }).unwrap();
        let bridge4 = BridgeLine::new(Position { x: 5, y: 2 }, Position { x: 2, y: 2 }).unwrap();
        assert_eq!(bridge3, bridge4);
    }

    // ============================================================================
    // ISLAND PLACEMENT TESTS
    // ============================================================================

    #[test]
    fn test_add_island_success() {
        // Test: Adding an island at a valid position should succeed
        let mut grid = HashiGrid::new(5, 5).unwrap();
        let result = grid.add_island(Position { x: 2, y: 2 });
        assert!(result.is_ok());
        assert!(grid.islands.contains_key(&Position { x: 2, y: 2 }));
    }

    #[test]
    fn test_add_island_out_of_bounds_x() {
        // Test: Adding an island at x >= width should fail
        let mut grid = HashiGrid::new(5, 5).unwrap();
        let result = grid.add_island(Position { x: 5, y: 2 });
        assert_eq!(
            result.unwrap_err(),
            HashiError::OutOfBounds {
                position: Position { x: 5, y: 2 }
            }
        );
    }

    #[test]
    fn test_add_island_out_of_bounds_y() {
        // Test: Adding an island at y >= height should fail
        let mut grid = HashiGrid::new(5, 5).unwrap();
        let result = grid.add_island(Position { x: 2, y: 5 });
        assert_eq!(
            result.unwrap_err(),
            HashiError::OutOfBounds {
                position: Position { x: 2, y: 5 }
            }
        );
    }

    #[test]
    fn test_add_island_duplicate() {
        // Test: Adding an island where one already exists should fail
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        let result = grid.add_island(Position { x: 2, y: 2 });
        assert_eq!(
            result.unwrap_err(),
            HashiError::Overwrite {
                position: Position { x: 2, y: 2 }
            }
        );
    }

    #[test]
    fn test_add_island_on_existing_bridge_path() {
        // Test: Cannot place an island where a bridge already crosses through
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 2, y: 1 }).unwrap();
        grid.add_island(Position { x: 2, y: 4 }).unwrap();

        // Add a vertical bridge between the islands
        let bridge = BridgeLine::new(Position { x: 2, y: 1 }, Position { x: 2, y: 4 }).unwrap();
        grid.add_bridge(bridge).unwrap();

        // Try to place an island on the bridge path (should fail)
        let result = grid.add_island(Position { x: 2, y: 2 });
        assert_eq!(
            result.unwrap_err(),
            HashiError::Overwrite {
                position: Position { x: 2, y: 2 }
            }
        );
    }

    // ============================================================================
    // BRIDGE ADDITION LOGIC TESTS
    // ============================================================================

    #[test]
    fn test_add_bridge_between_connected_islands() {
        // Test: Successfully adding a bridge between two islands
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        let result = grid.add_bridge(bridge);
        assert_eq!(result.unwrap(), BridgeType::Single);
        assert!(grid.bridges.contains_key(&bridge));
    }

    #[test]
    fn test_add_bridge_to_unconnected_island_start() {
        // Test: Bridge cannot be added if start position has no island
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        let result = grid.add_bridge(bridge);
        assert_eq!(
            result.unwrap_err(),
            HashiError::UnconnectedBridge {
                line: bridge,
                position: Position { x: 1, y: 2 }
            }
        );
    }

    #[test]
    fn test_add_bridge_to_unconnected_island_end() {
        // Test: Bridge cannot be added if end position has no island
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        let result = grid.add_bridge(bridge);
        assert_eq!(
            result.unwrap_err(),
            HashiError::UnconnectedBridge {
                line: bridge,
                position: Position { x: 4, y: 2 }
            }
        );
    }

    #[test]
    fn test_add_bridge_crossing_island() {
        // Test: Bridge cannot cross through an island (except at endpoints)
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap(); // Island in the middle

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        let result = grid.add_bridge(bridge);
        assert_eq!(
            result.unwrap_err(),
            HashiError::Overwrite {
                position: Position { x: 2, y: 2 }
            }
        );
    }

    #[test]
    fn test_add_bridge_crossing_another_bridge() {
        // Test: Bridge cannot cross another bridge (perpendicular intersection)
        let mut grid = HashiGrid::new(5, 10).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        grid.add_island(Position { x: 2, y: 5 }).unwrap();
        grid.add_island(Position { x: 1, y: 3 }).unwrap();
        grid.add_island(Position { x: 4, y: 3 }).unwrap();

        // Add vertical bridge first
        let vertical = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 5 }).unwrap();
        grid.add_bridge(vertical).unwrap();

        // Try to add horizontal bridge that crosses the vertical one
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
    fn test_add_bridge_single_then_double() {
        // Test: Adding the same bridge twice converts single -> double
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();

        // First add should create a single bridge
        let result1 = grid.add_bridge(bridge);
        assert_eq!(result1.unwrap(), BridgeType::Single);

        // Second add should upgrade to double bridge
        let result2 = grid.add_bridge(bridge);
        assert_eq!(result2.unwrap(), BridgeType::Double);

        // Verify the bridge is now double
        assert_eq!(*grid.bridges.get(&bridge).unwrap(), BridgeType::Double);
    }

    #[test]
    fn test_add_bridge_double_cannot_add_third() {
        // Test: Cannot add a third bridge between same two islands
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();

        // Add single
        grid.add_bridge(bridge).unwrap();
        // Upgrade to double
        grid.add_bridge(bridge).unwrap();

        // Try to add a third - should fail
        let result = grid.add_bridge(bridge);
        assert_eq!(
            result.unwrap_err(),
            HashiError::Overwrite {
                position: Position { x: 1, y: 2 }
            }
        );
    }

    // ============================================================================
    // COMPLETE PUZZLE VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_is_complete_with_no_bridges() {
        // Test: A puzzle with islands but no bridges is incomplete
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        grid.islands
            .get_mut(&Position { x: 2, y: 2 })
            .unwrap()
            .required_bridges = 1;

        assert!(!grid.is_complete());
    }

    #[test]
    fn test_is_complete_incomplete_connection() {
        // Test: Puzzle with islands that need more bridges to complete
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        // Set required bridges higher than actual
        grid.islands
            .get_mut(&Position { x: 1, y: 2 })
            .unwrap()
            .required_bridges = 2;
        grid.islands
            .get_mut(&Position { x: 4, y: 2 })
            .unwrap()
            .required_bridges = 2;

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        grid.add_bridge(bridge).unwrap(); // Only single bridge (1 connection each)

        assert!(!grid.is_complete());
    }

    #[test]
    fn test_is_complete_satisfied_single_bridges() {
        // Test: Puzzle is complete when all islands have exact required bridge count (single bridges)
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        grid.islands
            .get_mut(&Position { x: 1, y: 2 })
            .unwrap()
            .required_bridges = 1;
        grid.islands
            .get_mut(&Position { x: 4, y: 2 })
            .unwrap()
            .required_bridges = 1;

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        grid.add_bridge(bridge).unwrap();

        assert!(grid.is_complete());
    }

    #[test]
    fn test_is_complete_satisfied_double_bridges() {
        // Test: Puzzle is complete with double bridges (counts as 2 connections per endpoint)
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        grid.islands
            .get_mut(&Position { x: 1, y: 2 })
            .unwrap()
            .required_bridges = 2;
        grid.islands
            .get_mut(&Position { x: 4, y: 2 })
            .unwrap()
            .required_bridges = 2;

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        grid.add_bridge(bridge).unwrap(); // Single
        grid.add_bridge(bridge).unwrap(); // Upgrade to double

        assert!(grid.is_complete());
    }

    #[test]
    fn test_is_complete_multiple_bridges_per_island() {
        // Test: Island with bridges in multiple directions properly counted
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        grid.add_island(Position { x: 2, y: 0 }).unwrap();
        grid.add_island(Position { x: 2, y: 4 }).unwrap();
        grid.add_island(Position { x: 0, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        // Central island should have 4 connections (one per direction)
        grid.islands
            .get_mut(&Position { x: 2, y: 2 })
            .unwrap()
            .required_bridges = 4;
        // All surrounding islands need 1 connection
        grid.islands
            .get_mut(&Position { x: 2, y: 0 })
            .unwrap()
            .required_bridges = 1;
        grid.islands
            .get_mut(&Position { x: 2, y: 4 })
            .unwrap()
            .required_bridges = 1;
        grid.islands
            .get_mut(&Position { x: 0, y: 2 })
            .unwrap()
            .required_bridges = 1;
        grid.islands
            .get_mut(&Position { x: 4, y: 2 })
            .unwrap()
            .required_bridges = 1;

        // Add all four bridges
        let up = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 0 }).unwrap();
        let down = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 4 }).unwrap();
        let left = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 0, y: 2 }).unwrap();
        let right = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 4, y: 2 }).unwrap();

        grid.add_bridge(up).unwrap();
        grid.add_bridge(down).unwrap();
        grid.add_bridge(left).unwrap();
        grid.add_bridge(right).unwrap();

        assert!(grid.is_complete());
    }

    #[test]
    fn test_is_complete_has_zero_required_bridges_island() {
        // Test: If any island has 0 required bridges, puzzle is incomplete
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        // Don't set required_bridges (defaults to 0)

        assert!(!grid.is_complete());
    }

    // ============================================================================
    // BRIDGE WIPING TESTS
    // ============================================================================

    #[test]
    fn test_wipe_bridges_clears_all_bridges() {
        // Test: wipe_bridges removes all bridges but keeps islands
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        let bridge = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        grid.add_bridge(bridge).unwrap();

        assert!(!grid.bridges.is_empty());
        let wiped_grid = grid.wipe_bridges();
        assert!(wiped_grid.bridges.is_empty());
        assert!(!wiped_grid.islands.is_empty());
    }

    // ============================================================================
    // GRID GENERATION TESTS (Deterministic Seeding)
    // ============================================================================

    #[test]
    fn test_generate_with_seed_produces_same_grid() {
        // Test: Same seed always produces identical grid
        let seed = 12345;
        let grid1 = HashiGrid::generate_with_seed(10, 10, seed).unwrap();
        let grid2 = HashiGrid::generate_with_seed(10, 10, seed).unwrap();

        assert_eq!(grid1.islands, grid2.islands);
        assert_eq!(grid1.bridges, grid2.bridges);
    }

    #[test]
    fn test_generate_with_seed_different_seeds_differ() {
        // Test: Different seeds produce different grids (with very high probability)
        let grid1 = HashiGrid::generate_with_seed(10, 10, 12345).unwrap();
        let grid2 = HashiGrid::generate_with_seed(10, 10, 54321).unwrap();

        // Grids should differ (checking islands as main content)
        assert_ne!(grid1.islands, grid2.islands);
    }

    #[test]
    fn test_generate_with_seed_creates_islands() {
        // Test: Generated grid should contain islands
        let grid = HashiGrid::generate_with_seed(10, 10, 42).unwrap();
        assert!(!grid.islands.is_empty());
        assert!(grid.islands.len() >= 8); // Minimum from code: (width*height)/5 or 8
    }

    #[test]
    fn test_generate_with_seed_creates_bridges() {
        // Test: Generated grid should contain bridges
        let grid = HashiGrid::generate_with_seed(10, 10, 42).unwrap();
        assert!(!grid.bridges.is_empty());
    }

    #[test]
    fn test_generate_with_seed_respects_grid_bounds() {
        // Test: All islands and bridges should be within grid bounds
        let grid = HashiGrid::generate_with_seed(10, 10, 42).unwrap();

        for pos in grid.islands.keys() {
            assert!(pos.x < grid.width);
            assert!(pos.y < grid.height);
        }

        for bridge in grid.bridges.keys() {
            assert!(bridge.start.x < grid.width);
            assert!(bridge.start.y < grid.height);
            assert!(bridge.end.x < grid.width);
            assert!(bridge.end.y < grid.height);
        }
    }

    #[test]
    fn test_generate_with_seed_small_grid() {
        // Test: Generation works on minimum viable grid size
        let grid = HashiGrid::generate_with_seed(3, 3, 42).unwrap();
        assert!(!grid.islands.is_empty());
    }

    // ============================================================================
    // COMPLEX SCENARIO TESTS
    // ============================================================================

    #[test]
    fn test_four_way_island_with_double_bridges() {
        // Test: Central island with 4 double bridges (8 total connections)
        let mut grid = HashiGrid::new(5, 5).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();
        grid.add_island(Position { x: 2, y: 0 }).unwrap();
        grid.add_island(Position { x: 2, y: 4 }).unwrap();
        grid.add_island(Position { x: 0, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();

        // Set requirements
        grid.islands
            .get_mut(&Position { x: 2, y: 2 })
            .unwrap()
            .required_bridges = 8;
        grid.islands
            .get_mut(&Position { x: 2, y: 0 })
            .unwrap()
            .required_bridges = 2;
        grid.islands
            .get_mut(&Position { x: 2, y: 4 })
            .unwrap()
            .required_bridges = 2;
        grid.islands
            .get_mut(&Position { x: 0, y: 2 })
            .unwrap()
            .required_bridges = 2;
        grid.islands
            .get_mut(&Position { x: 4, y: 2 })
            .unwrap()
            .required_bridges = 2;

        // Add all bridges as doubles
        let up = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 0 }).unwrap();
        let down = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 2, y: 4 }).unwrap();
        let left = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 0, y: 2 }).unwrap();
        let right = BridgeLine::new(Position { x: 2, y: 2 }, Position { x: 4, y: 2 }).unwrap();

        for bridge in [up, down, left, right] {
            grid.add_bridge(bridge).unwrap(); // Single
            grid.add_bridge(bridge).unwrap(); // Double
        }

        assert!(grid.is_complete());
    }

    #[test]
    fn test_complex_bridge_network_two_rows() {
        // Test: Create a 2-row puzzle with multiple islands and bridges
        // Layout: (1,0)-(2,0)-(3,0)
        //           |
        //         (2,2)
        let mut grid = HashiGrid::new(4, 3).unwrap();
        grid.add_island(Position { x: 1, y: 0 }).unwrap();
        grid.add_island(Position { x: 2, y: 0 }).unwrap();
        grid.add_island(Position { x: 3, y: 0 }).unwrap();
        grid.add_island(Position { x: 2, y: 2 }).unwrap();

        // Set bridge requirements (each island needs proper connections)
        grid.islands
            .get_mut(&Position { x: 1, y: 0 })
            .unwrap()
            .required_bridges = 1;
        grid.islands
            .get_mut(&Position { x: 2, y: 0 })
            .unwrap()
            .required_bridges = 3; // Right + Left + Down
        grid.islands
            .get_mut(&Position { x: 3, y: 0 })
            .unwrap()
            .required_bridges = 1;
        grid.islands
            .get_mut(&Position { x: 2, y: 2 })
            .unwrap()
            .required_bridges = 1;

        // Add horizontal bridges on top row
        let h1 = BridgeLine::new(Position { x: 1, y: 0 }, Position { x: 2, y: 0 }).unwrap();
        let h2 = BridgeLine::new(Position { x: 2, y: 0 }, Position { x: 3, y: 0 }).unwrap();
        let v1 = BridgeLine::new(Position { x: 2, y: 0 }, Position { x: 2, y: 2 }).unwrap();

        grid.add_bridge(h1).unwrap();
        grid.add_bridge(h2).unwrap();
        grid.add_bridge(v1).unwrap();

        assert!(grid.is_complete());
    }

    #[test]
    fn test_can_place_multiple_disconnected_islands() {
        // Test: Multiple islands that don't form a connected network
        let mut grid = HashiGrid::new(10, 10).unwrap();
        grid.add_island(Position { x: 1, y: 1 }).unwrap();
        grid.add_island(Position { x: 3, y: 1 }).unwrap();
        grid.add_island(Position { x: 7, y: 7 }).unwrap();
        grid.add_island(Position { x: 9, y: 9 }).unwrap();

        assert_eq!(grid.islands.len(), 4);
        // All should be present even though they're not connected
        assert!(grid.islands.contains_key(&Position { x: 1, y: 1 }));
        assert!(grid.islands.contains_key(&Position { x: 3, y: 1 }));
        assert!(grid.islands.contains_key(&Position { x: 7, y: 7 }));
        assert!(grid.islands.contains_key(&Position { x: 9, y: 9 }));
    }

    #[test]
    fn test_horizontal_and_vertical_bridge_on_same_row_col() {
        // Test: A horizontal and vertical bridge can coexist on the same row/col without crossing
        let mut grid = HashiGrid::new(6, 6).unwrap();
        grid.add_island(Position { x: 1, y: 2 }).unwrap();
        grid.add_island(Position { x: 4, y: 2 }).unwrap();
        grid.add_island(Position { x: 2, y: 1 }).unwrap();
        grid.add_island(Position { x: 2, y: 3 }).unwrap();

        // Horizontal bridge along y=2, x: 1..4
        let horiz = BridgeLine::new(Position { x: 1, y: 2 }, Position { x: 4, y: 2 }).unwrap();
        // Vertical bridge along x=2, y: 1..3
        let vert = BridgeLine::new(Position { x: 2, y: 1 }, Position { x: 2, y: 3 }).unwrap();

        // The horizontal bridge will pass through (2, 2)
        // The vertical bridge will pass through (2, 2)
        // This is an intersection!
        let result = grid.add_bridge(horiz);
        assert!(result.is_ok());

        let result = grid.add_bridge(vert);
        // Should fail because they intersect at (2, 2)
        assert!(result.is_err());
    }
}
