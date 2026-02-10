#[cfg(feature = "compute-gpu")]
use crate::gpu::{GpuBuffer, GpuContext};
#[cfg(feature = "compute-gpu")]
use wgpu::util::DeviceExt;

#[cfg(feature = "compute-gpu")]
pub struct LightLeakPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "compute-gpu")]
impl LightLeakPipeline {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("LightLeak Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/light_leak.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("LightLeak BindGroupLayout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
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

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("LightLeak Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("LightLeak Pipeline"),
                layout: Some(&layout),
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
        buffer: &mut GpuBuffer,
        config: &crate::light_leak::LightLeakConfig,
    ) {
        if !config.enabled || config.leaks.is_empty() {
            return;
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        struct GpuLightLeak {
            position: [f32; 2],
            radius: f32,
            intensity: f32,
            color: [f32; 3],
            shape: u32,
            rotation: f32,
            roughness: f32,
            padding: [f32; 2],
        }
        unsafe impl bytemuck::Zeroable for GpuLightLeak {}
        unsafe impl bytemuck::Pod for GpuLightLeak {}

        let gpu_leaks: Vec<GpuLightLeak> = config
            .leaks
            .iter()
            .map(|l| GpuLightLeak {
                position: [l.position.0, l.position.1],
                radius: l.radius,
                intensity: l.intensity,
                color: l.color,
                shape: l.shape as u32,
                rotation: l.rotation,
                roughness: l.roughness,
                padding: [0.0; 2],
            })
            .collect();

        let leaks_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Leaks Buffer"),
                contents: bytemuck::cast_slice(&gpu_leaks),
                usage: wgpu::BufferUsages::STORAGE,
            });

        #[repr(C)]
        #[derive(Copy, Clone)]
        struct Uniforms {
            width: u32,
            height: u32,
            leak_count: u32,
            _pad: u32,
        }
        unsafe impl bytemuck::Zeroable for Uniforms {}
        unsafe impl bytemuck::Pod for Uniforms {}

        let uniforms = Uniforms {
            width: buffer.width,
            height: buffer.height,
            leak_count: gpu_leaks.len() as u32,
            _pad: 0,
        };

        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("LightLeak Uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("LightLeak BindGroup"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: leaks_buffer.as_entire_binding(),
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
                label: Some("LightLeak Encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("LightLeak Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let x_groups = buffer.width.div_ceil(16);
            let y_groups = buffer.height.div_ceil(16);
            pass.dispatch_workgroups(x_groups, y_groups, 1);
        }

        context.queue.submit(Some(encoder.finish()));
    }
}
