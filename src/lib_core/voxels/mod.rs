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
const VALID_MASK_INDEX: u32 = 0b0000_0000_0000_0000_0000_0000_1000_0000;

#[cfg_attr(rustfmt, rustfmt_skip)]
const LEAF_MASK:        u32 = 0b0000_0000_0000_0000_1111_1111_0000_0000;
#[cfg_attr(rustfmt, rustfmt_skip)]
const LEAF_MASK_INDEX:  u32 = 0b0000_0000_0000_0000_1000_0000_0000_0000;

#[derive(Copy, Clone, PartialEq)]
pub enum VoxelStreamTypes {
    PageHeader(u32),
    Voxel(Octree),
    InfoSection,
}

pub struct VoxelStreamManager {
    next_page_header: u32,
    data: Vec<VoxelStreamTypes>,
}

impl VoxelStreamManager {
    pub fn new() -> Self {
        let mut stream_manager = Self {
            next_page_header: 0,
            data: vec![], // TODO: reserve data?
        };

        // TODO: automate adding of headers when adding/removing voxels
        stream_manager.add_header();

        // Add a single voxel/octree node.
        let mut voxel = Octree::empty();
        voxel.add_child(0);
        voxel.add_child(6);
        stream_manager.data.push(VoxelStreamTypes::Voxel(voxel));

        //Simply using 128 elements in the buffer for now. Load with empty voxels.
        for _ in 0..127 {
            stream_manager.data.push(VoxelStreamTypes::Voxel(voxel));
        }

        stream_manager
    }

    fn add_header(&mut self) -> u32 {
        let page_header = self.next_page_header;
        self.data.push(VoxelStreamTypes::PageHeader(page_header));
        self.next_page_header += 1;

        page_header
    }

    pub fn init(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        //TODO: move this code to GFX

        // Could probably use some optimization
        let bytes: Vec<u8> = self
            .data
            .iter()
            .map(|d| match d {
                VoxelStreamTypes::PageHeader(header) => *header,
                VoxelStreamTypes::Voxel(octree) => octree.mask,
                VoxelStreamTypes::InfoSection => 0,
            })
            .map(|b| b.to_le_bytes())
            .collect::<Vec<[u8; 4]>>()
            .iter()
            .map(|b| b.iter())
            .flatten()
            .map(|b| *b)
            .collect();

        use wgpu::util::DeviceExt;
        let octree_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("voxel_buf"),
            contents: &bytes,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: true,
                    min_binding_size: wgpu::BufferSize::new((bytes.len() + 1) as u64), //TODO: fix up?
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(octree_buffer.slice(..)),
            }],
        });

        return (bind_group_layout, bind_group);
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Octree {
    mask: u32,
}

impl Octree {
    pub fn serialize(&self) -> [u8; 4] {
        self.mask.to_le_bytes()
    }

    pub fn deserialize(bytes: [u8; 4]) -> Self {
        Self {
            mask: u32::from_le_bytes(bytes),
        }
    }

    pub fn empty() -> Self {
        Self { mask: 0 }
    }

    pub fn from_u32(voxel: u32) -> Self {
        Self { mask: voxel }
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

    pub fn add_child(&mut self, child: u8) {
        // Simple dummy code to add leafs
        let child_index = child % 8;

        let leaf_mask = LEAF_MASK_INDEX >> child_index;
        let valid_mask = VALID_MASK_INDEX >> child_index;

        self.mask |= leaf_mask;
        self.mask |= valid_mask;
    }
}
