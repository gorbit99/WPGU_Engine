#[repr(C)]
#[derive(Copy, Clone)]
pub struct Mat4 {
    values: [f32; 16]
}

impl Mat4 {
    pub fn new(m00: f32, m01: f32, m02: f32, m03: f32,
               m10: f32, m11: f32, m12: f32, m13: f32,
               m20: f32, m21: f32, m22: f32, m23: f32,
               m30: f32, m31: f32, m32: f32, m33: f32,) -> Mat4 {
        Mat4 {
            values: [
                m00, m01, m02, m03,
                m10, m11, m12, m13,
                m20, m21, m22, m23,
                m30, m31, m32, m33
            ]
        }
    }

    pub fn identity() -> Mat4 {
        Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        )
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Mat4 {
        Mat4::new(
            1.0, 0.0, 0.0, x,
            0.0, 1.0, 0.0, y,
            0.0, 0.0, 1.0, z,
            0.0, 0.0, 0.0, 1.0
        )
    }

    pub fn rotate(x: f32, y: f32, z: f32) -> Mat4 {
        let sx = x.sin();
        let sy = y.sin();
        let sz = z.sin();
        let cx = x.cos();
        let cy = y.cos();
        let cz = z.cos();
        Mat4::new(
            cy * cz, -cy * sz, sy, 0.0,
            sx * sy * cz + cx * sz, -sx * sy * sz + cx * cz, -sx * cy, 0.0,
            -cx * sy * cz + sx * sz, cx * sy * sz + sx * cz, cx * cy, 0.0,
            0.0, 0.0, 0.0, 1.0
        )
    }

    pub fn perspective(fov: f32, aspect: f32, far: f32, near: f32) -> Mat4 {
        let tfov = 1.0 / (fov / 2.0).tan();
        let farnear = -1.0 / (far - near);
        Mat4::new(
            tfov / aspect, 0.0, 0.0, 0.0,
            0.0, tfov, 0.0, 0.0,
            0.0, 0.0, (far + near) * farnear, 2.0 * far * near * farnear,
            0.0, 0.0, -1.0, 0.0
        )
    }
}