pub type PaletteIndex = u8;

/// A single RGBA color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Struct containing all Palette colors
pub struct Palette {
    max_colors: usize,
    colors: Vec<Color>,
}

fn add_col(r: u8, g: u8, b: u8, a: u8, colors: &mut Vec<Color>) {
    colors.push(Color { r, g, b, a });
}

fn add_col_indexed(r: u8, g: u8, b: u8, a: u8, index: usize, colors: &mut Vec<Color>) {
    let index = index % colors.len();
    colors[index] = Color { r, g, b, a };
}

impl Palette {
    pub fn new() -> Self {
        let max_colors = u8::MAX as usize;
        let mut colors = Vec::with_capacity(max_colors);

        // Add in voxel colors

        // Voxel::Empty
        add_col(0, 0, 0, 0, &mut colors);
        //     Voxel::Skin
        add_col(252, 215, 172, 255, &mut colors);

        //     Voxel::Bone => (),
        add_col(255, 244, 232, 255, &mut colors);

        //     Voxel::Cloth => (),
        add_col(41, 216, 255, 255, &mut colors);

        //     Voxel::Metal => (83, 94, 97),
        add_col(83, 94, 97, 255, &mut colors);

        // Debug stuff
        {
            use crate::lib_core::voxels::Voxel;
            let (r, g, b) = Voxel::DebugCollisionShape.to_color();
            println!("{:?}", Voxel::DebugCollisionShape.to_color());
            add_col(r, g, b, 255, &mut colors);
        }

        // Ensure we populate the full texture
        for _ in 0..max_colors - colors.len() {
            colors.push(Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            });
        }

        Self { max_colors, colors }
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn colors(&self) -> &Vec<Color> {
        &self.colors
    }
}
