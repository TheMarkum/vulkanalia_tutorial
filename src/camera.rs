use std::default::Default;

use anyhow::{Ok, Result};
use cgmath::{vec3, vec4, Deg, Matrix4, Transform, Vector3};

type Vec4 = cgmath::Vector4<f32>;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Mat4(pub Matrix4<f32>);

impl Mat4 {
    pub fn new(
        c0r0: f32,
        c0r1: f32,
        c0r2: f32,
        c0r3: f32,
        c1r0: f32,
        c1r1: f32,
        c1r2: f32,
        c1r3: f32,
        c2r0: f32,
        c2r1: f32,
        c2r2: f32,
        c2r3: f32,
        c3r0: f32,
        c3r1: f32,
        c3r2: f32,
        c3r3: f32,
    ) -> Self {
        Self(Matrix4::new(
            c0r0, c0r1, c0r2, c0r3, c1r0, c1r1, c1r2, c1r3, c2r0, c2r1, c2r2, c2r3, c3r0, c3r1,
            c3r2, c3r3,
        ))
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self(
            Mat4::new(
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            )
            .0,
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec3(pub Vector3<f32>);

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(vec3(x, y, z))
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self(Vec3::new(0.0, 0.0, 0.0).0)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Camera {
    pub velocity: Vec3,
    pub position: Vec3,
    pub rotation: Mat4,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn new(velocity: Vec3, position: Vec3, pitch: f32, yaw: f32) -> Self {
        Self {
            velocity,
            position,
            rotation: Mat4::default(),
            pitch,
            yaw,
        }
    }

    pub fn update(self, position: Vector3<f32>) -> Result<Vector3<f32>> {
        let foo = self.velocity.0 * 0.5;
        let bar = self.rotation.0 * vec4(foo.x, foo.y, foo.z, 0.0);

        let pos = position + vec3(bar.x, bar.y, bar.z);

        Ok(pos)
    }

    pub fn get_rotation_matrix(self) -> Result<Matrix4<f32>> {
        let pitch_rotation = Matrix4::from_axis_angle(vec3(1.0, 0.0, 0.0), Deg(self.pitch));
        let yaw_rotation = Matrix4::from_axis_angle(vec3(0.0, -1.0, 0.0), Deg(self.yaw));

        let rotation = pitch_rotation * yaw_rotation;

        Ok(rotation)
    }

    pub fn get_view_matrix(self) -> Result<Matrix4<f32>> {
        let cam_trans = Matrix4::from_translation(self.position.0);

        let view_matrix =
            Matrix4::inverse_transform(&(cam_trans * self.get_rotation_matrix()?)).unwrap();

        Ok(view_matrix)
    }
}
