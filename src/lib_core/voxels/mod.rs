/// Sparse Voxel Octree. Based on https://research.nvidia.com/sites/default/files/pubs/2010-02_Efficient-Sparse-Voxel/laine2010i3d_paper.pdf
/// Does not include contours
/// Uses Little Endian

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Voxel {
    Empty,
    Skin,
    Bone,
    Cloth,
    Metal,
}

pub struct VoxelChunk {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    voxels: Vec<Voxel>,
}

impl VoxelChunk {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize) -> Self {
        let capacity = x_depth * y_depth * z_depth;
        let mut voxels = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            voxels.push(Voxel::Empty);
        }

        Self {
            x_depth,
            y_depth,
            z_depth,
            voxels,
        }
    }

    fn index_1d(&self, x: usize, y: usize, z: usize) -> usize {
        // Wrap so it's not out of bounds
        let x = x % self.x_depth;
        let y = y % self.y_depth;
        let z = z % self.z_depth;

        x + y * self.x_depth + z * self.x_depth * self.y_depth
    }

    fn index_3d(&self, i: usize) -> (usize, usize, usize) {
        let z = i / (self.x_depth * self.y_depth);
        let i = i - (z * self.x_depth * self.y_depth);
        let y = i / self.x_depth;
        let x = i % self.x_depth;

        (x, y, z)
    }

    fn voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        return self.voxels[self.index_1d(x, y, z)];
    }

    fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let i = self.index_1d(x, y, z);
        self.voxels[i] = voxel;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn VoxelChunk_New_DefaultsToEmpty() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = VoxelChunk::new(x_depth, y_depth, z_depth);

        assert_eq!(x_depth, chunk.x_depth);
        assert_eq!(y_depth, chunk.y_depth);
        assert_eq!(z_depth, chunk.z_depth);

        assert_eq!(x_depth * y_depth * z_depth, chunk.voxels.len());

        for voxel in chunk.voxels {
            assert_eq!(voxel, Voxel::Empty);
        }
    }

    #[test]
    fn VoxelChunk_index_1d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = VoxelChunk::new(x_depth, y_depth, z_depth);

        let (x, y, z) = (0, 0, 0);
        let expected = 0;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);

        let (x, y, z) = (1, 2, 3);
        let expected = x + y * x_depth + z * x_depth * y_depth;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);

        // Boundary check
        let (x, y, z) = (x_depth, y_depth, z_depth);
        let expected = 0;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);
    }

    #[test]
    fn VoxelChunk_index_3d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = VoxelChunk::new(x_depth, y_depth, z_depth);

        let (x, y, z) = (1, 2, 3);
        let expected = (x, y, z);
        let i = chunk.index_1d(x, y, z);
        let actual = chunk.index_3d(i);
        assert_eq!(expected, actual);

        let (x, y, z) = (3, 3, 2);
        let expected = (0, y, z);
        let i = chunk.index_1d(x, y, z);
        let actual = chunk.index_3d(i);
        assert_eq!(expected, actual);
    }

    #[test]
    fn VoxelChunk_set_voxels_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let mut chunk = VoxelChunk::new(x_depth, y_depth, z_depth);

        let voxel = Voxel::Cloth;
        let (x, y, z) = (0, 0, 0);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (0, 0, 0);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (2, 3, 1);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (3, 2, 4);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));
    }
}
