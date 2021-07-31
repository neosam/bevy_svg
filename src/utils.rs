use bevy::{math::{Vec2, Vec3}, prelude::{Color, Transform}};


pub trait TransformExt {
    fn to_bevy(&self) -> Transform;
    fn is_identity(&self) -> bool;
}

impl TransformExt for usvg::Transform {
    fn to_bevy(&self) -> Transform {
        Transform::from_matrix(bevy::math::Mat4::from_cols(
            [self.a as f32, self.b as f32, 0.0, 0.0].into(),
            [self.c as f32, self.d as f32, 0.0, 0.0].into(),
            [self.e as f32, self.f as f32, 1.0, 0.0].into(),
            [0.0, 0.0, 0.0, 1.0].into()
        ))
    }

    fn is_identity(&self) -> bool {
        *self == usvg::Transform::default()
    }
}

pub trait ColorExt {
    fn to_bevy(&self) -> Color;
    fn to_bevy_with_alpha_u8(&self, alpha: u8) -> Color;
}

impl ColorExt for usvg::Paint {
    fn to_bevy(&self) -> Color {
        match self {
            &usvg::Paint::Color(c) =>
                Color::rgb_u8(c.red, c.green, c.blue),
            _ => Color::default(),
        }
    }

    fn to_bevy_with_alpha_u8(&self, alpha: u8) -> Color {
        match self {
            &usvg::Paint::Color(c) =>
                Color::rgba_u8(c.red, c.green, c.blue, alpha),
            _ => Color::default(),
        }
    }
}
