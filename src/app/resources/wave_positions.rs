use parry3d::bounding_volume::AABB;

#[derive(Debug, Clone)]
pub struct WavePositions {
    pub positions: Vec<AABB>,
}

impl WavePositions {
    pub fn _new(positions: Vec<AABB>) -> Self {
        Self {
            positions,
        }
    }
}