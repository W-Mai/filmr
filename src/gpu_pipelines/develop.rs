#[cfg(feature = "compute-gpu")]
use crate::gpu::{GpuBuffer, GpuContext};
#[cfg(feature = "compute-gpu")]
use wgpu::util::DeviceExt;

#[cfg(feature = "compute-gpu")]
pub struct DevelopPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "compute-gpu")]
impl DevelopPipeline {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Develop Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/develop.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Develop Bind Group Layout"),
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
                    label: Some("Develop Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    immediate_size: 0,
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Develop Pipeline"),
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
        spectral_matrix: &[[f32; 3]; 3],
        wb_gains: [f32; 3],
        t_eff: f32,
    ) -> Option<GpuBuffer> {
        let width = input.width;
        let height = input.height;
        let size = input.size;

        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Develop Output Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        #[repr(C)]
        #[derive(Copy, Clone)]
        struct GpuCurve {
            d_min: f32,
            d_max: f32,
            gamma: f32,
            exposure_offset: f32,
            shoulder_point: f32,
            _pad0: f32,
            _pad1: f32,
            _pad2: f32,
        }
        unsafe impl bytemuck::Zeroable for GpuCurve {}
        unsafe impl bytemuck::Pod for GpuCurve {}

        impl From<&crate::film::SegmentedCurve> for GpuCurve {
            fn from(c: &crate::film::SegmentedCurve) -> Self {
                Self {
                    d_min: c.d_min,
                    d_max: c.d_max,
                    gamma: c.gamma,
                    exposure_offset: c.exposure_offset,
                    shoulder_point: c.shoulder_point,
                    _pad0: 0.0,
                    _pad1: 0.0,
                    _pad2: 0.0,
                }
            }
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        struct Uniforms {
            spectral_r: [f32; 4],
            spectral_g: [f32; 4],
            spectral_b: [f32; 4],
            color_r: [f32; 4],
            color_g: [f32; 4],
            color_b: [f32; 4],
            curve_r: GpuCurve,
            curve_g: GpuCurve,
            curve_b: GpuCurve,
            wb_r: f32,
            wb_g: f32,
            wb_b: f32,
            t_eff: f32,
            width: u32,
            height: u32,
            _pad: [u32; 2],
        }
        unsafe impl bytemuck::Zeroable for Uniforms {}
        unsafe impl bytemuck::Pod for Uniforms {}

        let sm = spectral_matrix;
        let cm = film.color_matrix;
        let uniforms = Uniforms {
            spectral_r: [sm[0][0], sm[0][1], sm[0][2], 0.0],
            spectral_g: [sm[1][0], sm[1][1], sm[1][2], 0.0],
            spectral_b: [sm[2][0], sm[2][1], sm[2][2], 0.0],
            color_r: [cm[0][0], cm[0][1], cm[0][2], 0.0],
            color_g: [cm[1][0], cm[1][1], cm[1][2], 0.0],
            color_b: [cm[2][0], cm[2][1], cm[2][2], 0.0],
            curve_r: GpuCurve::from(&film.r_curve),
            curve_g: GpuCurve::from(&film.g_curve),
            curve_b: GpuCurve::from(&film.b_curve),
            wb_r: wb_gains[0],
            wb_g: wb_gains[1],
            wb_b: wb_gains[2],
            t_eff,
            width,
            height,
            _pad: [0; 2],
        };

        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Develop Uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Develop Bind Group"),
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
                label: Some("Develop Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Develop Pass"),
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
