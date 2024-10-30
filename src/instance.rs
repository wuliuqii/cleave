use glam::{DVec4, Vec2, Vec4};
use wgpu::{Color, VertexBufferLayout};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    // matrix: Mat4,
    color: Vec4,
    pos: Vec2,
    size: Vec2,
    rotation: f32,
    _padding: [f32; 3],
}

impl InstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
      1=>Float32x4,
      2=>Float32x2,
      3=>Float32x2,
      4=>Float32,
    ];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Instance {
    pos: Vec2,
    // Shape Id?
    size: Vec2,
    // Rotation?
    color: Color,
}

impl Instance {
    fn to_raw(self) -> InstanceRaw {
        // let matrix = Mat4::from_scale_rotation_translation(
        //     self.size.extend(0.),
        //     Quat::IDENTITY,
        //     self.pos.extend(0.),
        // );
        let color = DVec4::new(self.color.r, self.color.g, self.color.b, self.color.a);
        InstanceRaw {
            // matrix,
            pos: self.pos,
            size: self.size,
            rotation: 0., // TODO: Rotation?
            color: color.as_vec4(),
            _padding: [0.; 3],
        }
    }
}