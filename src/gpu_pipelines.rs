use crate::gpu::GpuContext;
use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, Rgb, RgbImage};
use wgpu::util::DeviceExt;

#[cfg(feature = "compute-gpu")]
pub struct GpuBuffer {
    pub buffer: wgpu::Buffer,
    pub width: u32,
    pub height: u32,
    pub size: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LinearizeUniforms {
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct HalationUniforms {
    width: u32,
    height: u32,
    threshold: f32,
    _pad1: f32,
    sigma: f32,
    strength: f32,
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LightLeakUniforms {
    width: u32,
    height: u32,
    leak_count: u32,
    _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GpuLightLeak {
    position: [f32; 2],
    radius: f32,
    intensity: f32,
    color: [f32; 3],
    shape: u32,
    rotation: f32,
    roughness: f32,
    _pad: [f32; 2],
}

#[cfg(feature = "compute-gpu")]
pub struct LinearizePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "compute-gpu")]
pub struct HalationPipeline {
    pipeline_x: wgpu::ComputePipeline,
    pipeline_y: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "compute-gpu")]
pub struct LightLeakPipeline {
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
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/linearize.wgsl").into()),
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

    pub fn process_image(
        &self,
        context: &GpuContext,
        input: &RgbImage,
    ) -> Option<ImageBuffer<Rgb<f32>, Vec<f32>>> {
        crate::gpu::block_on(self.process_image_async(context, input))
    }

    pub fn process_to_gpu_buffer(
        &self,
        context: &GpuContext,
        input: &RgbImage,
    ) -> Option<GpuBuffer> {
        let width = input.width();
        let height = input.height();

        // 1. Prepare Input Buffer
        let raw_bytes = input.as_raw();
        let input_buffer = if raw_bytes.len().is_multiple_of(4) {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Input Buffer"),
                    contents: raw_bytes,
                    usage: wgpu::BufferUsages::STORAGE,
                })
        } else {
            let mut padded = raw_bytes.clone();
            while !padded.len().is_multiple_of(4) {
                padded.push(0);
            }
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Input Buffer"),
                    contents: &padded,
                    usage: wgpu::BufferUsages::STORAGE,
                })
        };

        // 2. Prepare Output Buffer (Storage Only)
        // Ensure COPY_SRC so we can read it back if needed, and STORAGE for writing.
        // Also allow it to be used as STORAGE input for next stage (read_write or read_only).
        let output_size = (width * height * 3 * 4) as u64; // f32 * 3 channels
        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // 3. Prepare Uniforms
        let uniforms = LinearizeUniforms { width, height };
        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        // 4. Bind Group
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

        // 5. Dispatch
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
        input: &RgbImage,
    ) -> Option<ImageBuffer<Rgb<f32>, Vec<f32>>> {
        let gpu_buffer = self.process_to_gpu_buffer(context, input)?;
        read_gpu_buffer(context, &gpu_buffer).await
    }
}

#[cfg(feature = "compute-gpu")]
pub async fn read_gpu_buffer(
    context: &GpuContext,
    gpu_buffer: &GpuBuffer,
) -> Option<ImageBuffer<Rgb<f32>, Vec<f32>>> {
    let width = gpu_buffer.width;
    let height = gpu_buffer.height;
    let size = gpu_buffer.size;

    // Prepare Staging Buffer (Map Read)
    let staging_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy Encoder"),
        });

    encoder.copy_buffer_to_buffer(&gpu_buffer.buffer, 0, &staging_buffer, 0, size);
    context.queue.submit(Some(encoder.finish()));

    // Map and Read
    let buffer_slice = staging_buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
        let _ = tx.send(v);
    });

    #[cfg(not(target_arch = "wasm32"))]
    context
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .unwrap();

    if let Ok(Ok(())) = rx.await {
        let data = buffer_slice.get_mapped_range();
        let result_vec: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        return ImageBuffer::from_raw(width, height, result_vec);
    }

    None
}

#[cfg(feature = "compute-gpu")]
pub fn create_gpu_buffer_from_f32(
    context: &GpuContext,
    input: &ImageBuffer<Rgb<f32>, Vec<f32>>,
) -> GpuBuffer {
    let width = input.width();
    let height = input.height();
    let raw_bytes = bytemuck::cast_slice(input.as_raw());

    let buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GpuBuffer from f32"),
            contents: raw_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

    GpuBuffer {
        buffer,
        width,
        height,
        size: (raw_bytes.len() as u64),
    }
}

#[cfg(feature = "compute-gpu")]
impl HalationPipeline {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Halation Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/halation.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Halation Bind Group Layout"),
                    entries: &[
                        // Input
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
                        // Output
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
                        // Uniforms
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
                        // Original Input (Used in Pass 2)
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
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
                    label: Some("Halation Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    immediate_size: 0,
                });

        let pipeline_x = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Halation Pipeline X"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main_x"),
                compilation_options: Default::default(),
                cache: None,
            });

        let pipeline_y = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Halation Pipeline Y"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main_y"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            pipeline_x,
            pipeline_y,
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

        if film.halation_strength <= 0.0 {
            // Copy input to output to pass through
            let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Halation Output Buffer (Copy)"),
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
                        label: Some("Halation Copy Encoder"),
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

        // Temp Buffer for X pass result
        let temp_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Halation Temp Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE, // ReadWrite in Pass 1, Read in Pass 2
            mapped_at_creation: false,
        });

        // Output Buffer
        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Halation Output Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let uniforms = HalationUniforms {
            width,
            height,
            threshold: film.halation_threshold,
            _pad1: 0.0,
            sigma: width as f32 * film.halation_sigma,
            strength: film.halation_strength,
            tint_r: film.halation_tint[0],
            tint_g: film.halation_tint[1],
            tint_b: film.halation_tint[2],
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        };

        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Halation Uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        // Bind Group 1: Pass X (Input -> Temp)
        let bind_group_x = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Halation Bind Group X"),
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
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: input.buffer.as_entire_binding(), // Unused
                    },
                ],
            });

        // Bind Group 2: Pass Y (Temp -> Output, using Input as Original)
        let bind_group_y = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Halation Bind Group Y"),
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
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: input.buffer.as_entire_binding(),
                    },
                ],
            });

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Halation Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Halation Pass"),
                timestamp_writes: None,
            });

            // Pass X
            compute_pass.set_pipeline(&self.pipeline_x);
            compute_pass.set_bind_group(0, &bind_group_x, &[]);
            let workgroup_size = 16;
            let x_groups = width.div_ceil(workgroup_size);
            let y_groups = height.div_ceil(workgroup_size);
            compute_pass.dispatch_workgroups(x_groups, y_groups, 1);

            // Pass Y
            compute_pass.set_pipeline(&self.pipeline_y);
            compute_pass.set_bind_group(0, &bind_group_y, &[]);
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

#[cfg(feature = "compute-gpu")]
impl LightLeakPipeline {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("LightLeak Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light_leak.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("LightLeak Bind Group Layout"),
                    entries: &[
                        // Image Buffer (ReadWrite)
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
                        // Leaks Buffer (ReadOnly)
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
                        // Uniforms
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
                    label: Some("LightLeak Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    immediate_size: 0,
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("LightLeak Pipeline"),
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
        image_buffer: &GpuBuffer, // In-place modification
        config: &crate::light_leak::LightLeakConfig,
    ) {
        if !config.enabled || config.leaks.is_empty() {
            return;
        }

        let width = image_buffer.width;
        let height = image_buffer.height;

        // Convert leaks to GPU struct
        let gpu_leaks: Vec<GpuLightLeak> = config
            .leaks
            .iter()
            .map(|l| GpuLightLeak {
                position: [l.position.0, l.position.1],
                radius: l.radius,
                intensity: l.intensity,
                color: l.color,
                shape: match l.shape {
                    crate::light_leak::LightLeakShape::Circle => 0,
                    crate::light_leak::LightLeakShape::Linear => 1,
                    crate::light_leak::LightLeakShape::Organic => 2,
                    crate::light_leak::LightLeakShape::Plasma => 3,
                },
                rotation: l.rotation,
                roughness: l.roughness,
                _pad: [0.0; 2],
            })
            .collect();

        let leaks_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("LightLeak Leaks Buffer"),
                contents: bytemuck::cast_slice(&gpu_leaks),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let uniforms = LightLeakUniforms {
            width,
            height,
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
                label: Some("LightLeak Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: image_buffer.buffer.as_entire_binding(),
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
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("LightLeak Pass"),
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
    }
}
