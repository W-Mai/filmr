#[cfg(feature = "compute-gpu")]
use crate::gpu::{GpuBuffer, GpuContext};
#[cfg(feature = "compute-gpu")]
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GrainUniforms {
    pub width: u32,
    pub height: u32,
    pub seed: f32,
    pub alpha: f32,
    pub sigma_read: f32,
    pub roughness: f32,
    pub monochrome: u32,
    pub _pad: f32,
}
unsafe impl bytemuck::Zeroable for GrainUniforms {}
unsafe impl bytemuck::Pod for GrainUniforms {}

#[cfg(feature = "compute-gpu")]
pub struct GrainPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "compute-gpu")]
impl GrainPipeline {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Grain Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/grain.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Grain Bind Group Layout"),
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
                    label: Some("Grain Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    immediate_size: 0,
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Grain Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn process(
        &self,
        context: &GpuContext,
        input: &GpuBuffer,
        film: &crate::FilmStock,
    ) -> Option<GpuBuffer> {
        let width = input.width;
        let height = input.height;
        let size = input.size;

        if film.grain_model.alpha <= 0.0 && film.grain_model.sigma_read <= 0.0 {
            let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Grain Output Buffer (Copy)"),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Grain Copy Encoder"),
                    });
            encoder.copy_buffer_to_buffer(&input.buffer, 0, &output_buffer, 0, size);
            context.queue.submit(Some(encoder.finish()));
            return Some(GpuBuffer {
                buffer: output_buffer,
                width,
                height,
                size,
            });
        }

        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grain Output Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let seed = 1234.5678;

        const REFERENCE_WIDTH: f32 = 2048.0;
        let scale_factor = width as f32 / REFERENCE_WIDTH;

        let alpha_scaled = film.grain_model.alpha * scale_factor * scale_factor;
        let sigma_read_scaled = film.grain_model.sigma_read * scale_factor;

        let uniforms = GrainUniforms {
            width,
            height,
            seed,
            alpha: alpha_scaled,
            sigma_read: sigma_read_scaled,
            roughness: film.grain_model.roughness,
            monochrome: if film.grain_model.monochrome { 1 } else { 0 },
            _pad: 0.0,
        };

        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grain Uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Grain Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input.buffer.as_entire_binding(),
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
                label: Some("Grain Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Grain Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let x_groups = width.div_ceil(16);
            let y_groups = height.div_ceil(16);
            compute_pass.dispatch_workgroups(x_groups, y_groups, 1);
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
