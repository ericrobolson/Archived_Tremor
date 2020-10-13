/// Sparse Voxel Octree. Based on https://research.nvidia.com/sites/default/files/pubs/2010-02_Efficient-Sparse-Voxel/laine2010i3d_paper.pdf
/// Does not include contours
/// Uses Little Endian


// Constants for bit ops
#[cfg_attr(rustfmt, rustfmt_skip)]
const FAR_BIT:          u32 = 0b0000_0000_0000_0001_0000_0000_0000_0000;
#[cfg_attr(rustfmt, rustfmt_skip)]
const CHILD_POINTER:    u32 = 0b1111_1111_1111_1110_0000_0000_0000_0000;
#[cfg_attr(rustfmt, rustfmt_skip)]
const VALID_MASK:       u32 = 0b0000_0000_0000_0000_0000_0000_1111_1111;
#[cfg_attr(rustfmt, rustfmt_skip)]
const LEAF_MASK:        u32 = 0b0000_0000_0000_0000_1111_1111_0000_0000;

pub struct VoxelStreamManager {}

pub struct Voxel {
    mask: u32,
}

impl Voxel {
    pub fn serialize(&self) -> [u8; 4] {
        self.mask.to_le_bytes()
    }

    pub fn deserialize(bytes: [u8; 4]) -> Self {
        Self {
            mask: u32::from_le_bytes(bytes),
        }
    }

    pub fn child_pointer(&self) -> u16 {
        // Child pointer is first 15 bits from the left
        let mask = self.mask & CHILD_POINTER;
        let mask = mask.to_le_bytes();

        u16::from_le_bytes([mask[0], mask[1]])
    }

    pub fn far(&self) -> bool {
        // Far is the 16th bit from the left
        (self.mask & FAR_BIT) != 0
    }

    /// Mask that tells whether each child slot contains a voxel
    pub fn valid_mask(&self) -> u8 {
        // 17-24 bits
        let mask = (self.mask & VALID_MASK).to_le_bytes();

        u8::from_le_bytes([mask[2]])
    }

    /// Mask that specifies whether each child is a leaf
    pub fn leaf_mask(&self) -> u8 {
        // 25-32 bits
        let mask = (self.mask & LEAF_MASK).to_le_bytes();

        u8::from_le_bytes([mask[3]])
    }

    pub fn valid_child(&self, child: u8) -> bool {
        ((self.valid_mask() >> child) & 1) > 0
    }

    pub fn leaf_child(&self, child: u8) -> bool {
        ((self.leaf_mask() >> child) & 1) > 0
    }

    pub fn active_voxel(&self, child: u8) -> bool {
        // if valid mask || leaf mask == true, is active.
        self.valid_child(child) || self.leaf_child(child)
    }

    pub fn is_leaf_voxel(&self, child: u8) -> bool {
        // if both bits set, contains a leaf voxel
        self.valid_child(child) && self.leaf_child(child)
    }
}
