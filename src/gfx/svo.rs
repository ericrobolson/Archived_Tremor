// sparse voxel octree

/*
Children layout, represented by a u8:
    Top layer:
    ---------
    | 0 | 1 |
    ---------
    | 4 | 3 |
    ---------
    Bottom layer:
    ---------
    | 5 | 6 |
    ---------
    | 8 | 7 |
    ---------
*/

pub enum MaterialType {
    Empty,
    Metal,
    Wood,
}

pub struct SparseVoxelOctree {
    // Children mask. Clockwise, starting at the top.
    children: u8,
    data: Vec<Option<SparseVoxelOctree>>,
}

impl SparseVoxelOctree {
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(8);
        for _ in 0..8 {
            data.push(None);
        }
        Self { children: 0, data }
    }
    pub fn average(&self) -> MaterialType {
        // If the children bitmask is empty, bounce out
        if self.children == 0 {
            return MaterialType::Empty;
        }

        unimplemented!();
    }
}
