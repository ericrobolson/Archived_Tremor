use crate::gfx::{DeviceQueue, DoubleBuffer};
use cgmath::Rotation3;
use wgpu::util::DeviceExt;

/// The representation of an instance for a model.
#[derive(Copy, Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Instance {
    pub fn default() -> Self {
        Self::new(
            [0.0, 0.0, 0.0].into(),
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
            [1.0, 1.0, 1.0].into(),
        )
    }

    pub fn new(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        scale: cgmath::Vector3<f32>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        InstanceRaw::desc()
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model_transform: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model_transform: (cgmath::Matrix4::<f32>::from_translation(self.position)
                * cgmath::Matrix4::<f32>::from(self.rotation)
                * cgmath::Matrix4::<f32>::from_nonuniform_scale(
                    self.scale.x,
                    self.scale.y,
                    self.scale.z,
                ))
            .into(),
        }
    }
}

/// Double buffered container for managing instances for models.
pub struct InstanceContainer {
    container_a: InnerInstanceContainer,
    container_b: InnerInstanceContainer,
    double_buffer: DoubleBuffer,
}

impl InstanceContainer {
    pub fn new(max_instances: u32, dq: &DeviceQueue) -> Self {
        Self {
            container_a: InnerInstanceContainer::new(max_instances, dq),
            container_b: InnerInstanceContainer::new(max_instances, dq),
            double_buffer: DoubleBuffer::UpdateARenderB,
        }
    }

    pub fn add_instance(&mut self, instance: Instance) -> bool {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => {
                return self.container_a.add_instance(instance);
            }
            DoubleBuffer::UpdateBRenderA => {
                return self.container_b.add_instance(instance);
            }
        }
    }

    pub fn clear(&mut self) {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => {
                self.double_buffer = DoubleBuffer::UpdateBRenderA;
                return self.container_b.clear();
            }
            DoubleBuffer::UpdateBRenderA => {
                self.double_buffer = DoubleBuffer::UpdateARenderB;
                return self.container_a.clear();
            }
        }
    }

    pub fn update_buffer(&mut self, dq: &DeviceQueue) {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => {
                self.container_a.update_buffer(dq);
            }
            DoubleBuffer::UpdateBRenderA => {
                self.container_b.update_buffer(dq);
            }
        }
    }

    /// Returns the render buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => self.container_b.buffer(),
            DoubleBuffer::UpdateBRenderA => self.container_a.buffer(),
        }
    }

    /// Returns the len of the render buffer
    pub fn len(&self) -> usize {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => self.container_b.len(),
            DoubleBuffer::UpdateBRenderA => self.container_a.len(),
        }
    }

    /// Returns whether the instances are empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

struct InnerInstanceContainer {
    max_instances: usize,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    dirty: bool,
}

impl InnerInstanceContainer {
    fn new(max_instances: u32, dq: &DeviceQueue) -> Self {
        let mut data = Vec::with_capacity(max_instances as usize);
        for _ in 0..max_instances {
            data.push(Instance::default());
        }

        let mut ic = Self::from_data(data, dq);
        ic.clear();
        ic.update_buffer(dq);

        ic
    }

    fn add_instance(&mut self, instance: Instance) -> bool {
        if self.instances.len() < self.max_instances {
            self.instances.push(instance);
            self.dirty = true;
            return true;
        }

        false
    }

    fn clear(&mut self) {
        self.instances.clear();
        self.dirty = true;
    }

    fn update_buffer(&mut self, dq: &DeviceQueue) {
        if self.dirty && self.instances.is_empty() == false {
            let instance_data = map_instances_to_raw(&self.instances);

            dq.queue
                .write_buffer(self.buffer(), 0, bytemuck::cast_slice(&instance_data));
        }

        self.dirty = false;
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.instance_buffer
    }

    fn len(&self) -> usize {
        self.instances.len()
    }

    fn from_data(instances: Vec<Instance>, dq: &DeviceQueue) -> Self {
        let instance_data = map_instances_to_raw(&instances);
        let instance_buffer = dq
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });

        Self {
            max_instances: instances.len(),
            instances,
            instance_buffer,
            dirty: false,
        }
    }
}

fn map_instances_to_raw(instances: &Vec<Instance>) -> Vec<InstanceRaw> {
    instances.iter().map(Instance::to_raw).collect::<Vec<_>>()
}
