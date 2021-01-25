use crate::gfx::{
    uniforms::{MaterialUbo, MaterialUniformContainer},
    DeviceQueue,
};

pub struct Material {
    /// The metallic-roughness texture.
    ///
    /// The metalness values are sampled from the B channel.
    /// The roughness values are sampled from the G channel.
    /// These values are linear. If other channels are present (R or A),
    /// they are ignored for metallic-roughness calculations.
    pub metallic_roughness_texture: Option<()>,

    /// Returns the base color texture. The texture contains RGB(A) components
    /// in sRGB color space.
    pub base_color_texture: Option<()>,

    pub normal_texture: Option<()>,

    pub occlusion_texture: Option<()>,
    pub emissive_texture: Option<()>,

    pub material_ubo: MaterialUniformContainer,
}
