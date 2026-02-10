#[cfg(feature = "compute-gpu")]
use crate::gpu::{GpuBuffer, GpuContext};
#[cfg(feature = "compute-gpu")]
use wgpu::util::DeviceExt;

#[cfg(feature = "compute-gpu")]
pub struct GaussianPipeline {
    pipeline_x: wgpu::ComputePipeline,
    pipeline_y: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "compute-gpu")]
impl GaussianPipeline {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Blur Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/blur.wgsl").into()),
            });

        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Blur Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Blur Pipeline Layout"),
                    bind_group_layouts: &[&layout],
                    immediate_size: 0,
                });

        let pipeline_x = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Blur Pipeline X"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main_x"),
                compilation_options: Default::default(),
                cache: None,
            });

        let pipeline_y = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Blur Pipeline Y"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main_y"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            pipeline_x,
            pipeline_y,
            bind_group_layout: layout,
        }
    }

    pub fn process(
        &self,
        context: &GpuContext,
        input: &GpuBuffer,
        sigma: f32,
    ) -> Option<GpuBuffer> {
        let width = input.width;
        let height = input.height;
        let size = input.size;

        let temp_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Blur Temp Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Blur Output Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        #[repr(C)]
        #[derive(Copy, Clone)]
        struct Uniforms {
            width: u32,
            height: u32,
            sigma: f32,
            _pad: f32,
        }
        unsafe impl bytemuck::Zeroable for Uniforms {}
        unsafe impl bytemuck::Pod for Uniforms {}

        let uniforms = Uniforms {
            width,
            height,
            sigma,
            _pad: 0.0,
        };

        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Blur Uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bg_x = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Blur BG X"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: temp_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let bg_y = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Blur BG Y"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: temp_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Blur Encoder"),
            });

        let x_groups = width.div_ceil(16);
        let y_groups = height.div_ceil(16);

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Blur Pass X"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline_x);
            pass.set_bind_group(0, &bg_x, &[]);
            pass.dispatch_workgroups(x_groups, y_groups, 1);
        }

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Blur Pass Y"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline_y);
            pass.set_bind_group(0, &bg_y, &[]);
            pass.dispatch_workgroups(x_groups, y_groups, 1);
        }

        context.queue.submit(Some(encoder.finish()));

        Some(GpuBuffer {
            buffer: output_buffer,
            width,
            height,
            size,
        })
    }
}
