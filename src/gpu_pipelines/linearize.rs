#[cfg(feature = "compute-gpu")]
use crate::gpu::{GpuBuffer, GpuContext};
#[cfg(feature = "compute-gpu")]
use wgpu::util::DeviceExt;

#[cfg(feature = "compute-gpu")]
pub struct LinearizePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "compute-gpu")]
impl LinearizePipeline {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Linearize Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/linearize.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Linearize Bind Group Layout"),
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
                    label: Some("Linearize Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    immediate_size: 0,
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Linearize Pipeline"),
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

    pub fn process_to_gpu_buffer(
        &self,
        context: &GpuContext,
        input: &image::RgbImage,
    ) -> Option<GpuBuffer> {
        let width = input.width();
        let height = input.height();
        let pixel_count = (width * height) as u64;
        let output_size = pixel_count * 3 * 4; // 3 channels, f32 (4 bytes)

        // Pad input data to multiple of 4 bytes for array<u32> compatibility
        let mut input_data = input.as_raw().clone();
        while !input_data.len().is_multiple_of(4) {
            input_data.push(0);
        }

        let input_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Linearize Input Buffer"),
                contents: &input_data,
                usage: wgpu::BufferUsages::STORAGE,
            });

        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Linearize Output Buffer"),
            size: output_size,
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
        }
        unsafe impl bytemuck::Zeroable for Uniforms {}
        unsafe impl bytemuck::Pod for Uniforms {}

        let uniforms = Uniforms { width, height };
        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Linearize Uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Linearize Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input_buffer.as_entire_binding(),
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
                label: Some("Linearize Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Linearize Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_size = 16;
            let x_groups = width.div_ceil(workgroup_size);
            let y_groups = height.div_ceil(workgroup_size);
            compute_pass.dispatch_workgroups(x_groups, y_groups, 1);
        }

        context.queue.submit(Some(encoder.finish()));

        Some(GpuBuffer {
            buffer: output_buffer,
            width,
            height,
            size: output_size,
        })
    }

    pub async fn process_image_async(
        &self,
        context: &GpuContext,
        input: &image::RgbImage,
    ) -> Option<image::ImageBuffer<image::Rgb<f32>, Vec<f32>>> {
        let gpu_buffer = self.process_to_gpu_buffer(context, input)?;
        super::read_gpu_buffer(context, &gpu_buffer).await
    }
}
