#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<BlockPosition> for Position {
    fn from(value: BlockPosition) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
            z: value.z as f64,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<Position> for BlockPosition {
    fn from(value: Position) -> Self {
        Self {
            x: value.x.floor() as i32,
            y: value.y.floor() as i32,
            z: value.z.floor() as i32,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl From<BlockPosition> for ChunkPosition {
    fn from(value: BlockPosition) -> Self {
        Self {
            x: value.x.div_euclid(16),
            z: value.z.div_euclid(16),
        }
    }
}

impl From<Position> for ChunkPosition {
    fn from(value: Position) -> Self {
        Self {
            x: (value.x.floor() as i32).div_euclid(16),
            z: (value.z.floor() as i32).div_euclid(16),
        }
    }
}
