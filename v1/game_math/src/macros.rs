/// Macro to implement constant types for a specific number.
#[macro_export]
macro_rules! implement_types {
    ($n:ty) => {
        pub type Num = $n;
        pub use crate::RawConverter;
        pub type Mat4 = crate::mat4::Mat4<Num>;
        pub type Vec2 = crate::vec2::Vec2<Num>;
        pub type Vec3 = crate::vec3::Vec3<Num>;
        pub type Vec4 = crate::vec4::Vec4<Num>;
        pub type Quaternion = crate::quaternion::Quaternion<Num>;
    };
}
