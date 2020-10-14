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
    pub texture: Option<wgpu::Texture>,
    pub view: Option<wgpu::TextureView>,
}

impl VoxelStreamManager {
    pub fn new() -> Self {
        let mut stream_manager = Self {
            next_page_header: 0,
            data: vec![], // TODO: reserve data?
            texture: None,
            view: None,
        };

        // TODO: automate adding of headers when adding/removing voxels
        stream_manager.add_header();

        // Add a single voxel/octree node.
        let mut voxel = Octree::empty();
        voxel.add_child(0);
        voxel.add_child(7);
        stream_manager.data.push(VoxelStreamTypes::Voxel(voxel));

        stream_manager
    }

    fn add_header(&mut self) -> u32 {
        let page_header = self.next_page_header;
        self.data.push(VoxelStreamTypes::PageHeader(page_header));
        self.next_page_header += 1;

        page_header
    }

    fn texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
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

        let tex_size = wgpu::Extent3d {
            width: self.data.len() as u32,
            height: 1,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::COPY_DST, // TODO: wire up to shader for storage (https://docs.rs/wgpu/0.6.0/wgpu/enum.BindingType.html#variant.StorageTexture)
            label: Some("voxels"),
        });

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytes, // bytes?
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: bytes.len() as u32,
                rows_per_image: 1,
            },
            tex_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.texture = Some(texture);
        self.view = Some(texture_view);
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
