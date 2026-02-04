#[cfg(feature = "compute-gpu")]
use crate::gpu::{GpuBuffer, GpuContext};
#[cfg(feature = "compute-gpu")]
use futures::channel::oneshot;
#[cfg(feature = "compute-gpu")]
use std::sync::OnceLock;
#[cfg(feature = "compute-gpu")]
use wgpu::util::DeviceExt;

// Wrapper to make WGPU types Send + Sync on WASM
#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
struct SendSyncWrapper<T>(T);

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
unsafe impl<T> Send for SendSyncWrapper<T> {}
#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
unsafe impl<T> Sync for SendSyncWrapper<T> {}

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
impl<T> std::ops::Deref for SendSyncWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
type PipelineWrapper<T> = T;

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
type PipelineWrapper<T> = SendSyncWrapper<T>;

#[cfg(feature = "compute-gpu")]
static LINEARIZE_PIPELINE: OnceLock<PipelineWrapper<LinearizePipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static LIGHT_LEAK_PIPELINE: OnceLock<PipelineWrapper<LightLeakPipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static HALATION_PIPELINE: OnceLock<PipelineWrapper<HalationPipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static GAUSSIAN_PIPELINE: OnceLock<PipelineWrapper<GaussianPipeline>> = OnceLock::new();

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
fn wrap_pipeline<T>(pipeline: T) -> T {
    pipeline
}

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
fn wrap_pipeline<T>(pipeline: T) -> SendSyncWrapper<T> {
    SendSyncWrapper(pipeline)
}

#[cfg(feature = "compute-gpu")]
pub fn get_linearize_pipeline(context: &GpuContext) -> &'static LinearizePipeline {
    LINEARIZE_PIPELINE.get_or_init(|| wrap_pipeline(LinearizePipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_light_leak_pipeline(context: &GpuContext) -> &'static LightLeakPipeline {
    LIGHT_LEAK_PIPELINE.get_or_init(|| wrap_pipeline(LightLeakPipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_halation_pipeline(context: &GpuContext) -> &'static HalationPipeline {
    HALATION_PIPELINE.get_or_init(|| wrap_pipeline(HalationPipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_gaussian_pipeline(context: &GpuContext) -> &'static GaussianPipeline {
    GAUSSIAN_PIPELINE.get_or_init(|| wrap_pipeline(GaussianPipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub async fn read_gpu_buffer(
    context: &GpuContext,
    gpu_buffer: &GpuBuffer,
) -> Option<image::ImageBuffer<image::Rgb<f32>, Vec<f32>>> {
    let size = gpu_buffer.size;

    // Create a staging buffer for reading back data
    let staging_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Copy from GPU buffer to staging buffer
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Readback Copy Encoder"),
        });
    encoder.copy_buffer_to_buffer(&gpu_buffer.buffer, 0, &staging_buffer, 0, size);
    let _submission_index = context.queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);

    let (sender, mut receiver) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
        sender.send(v).unwrap();
    });

    // Wait for the copy to finish and the map callback to fire
    // We use PollType::Wait to block until the submission is done.
    // Note: We need to loop because sometimes the callback fires on the *next* poll after completion?
    // But let's try strict wait first.
    loop {
        let _ = context.device.poll(wgpu::PollType::Poll);
        if let Ok(Some(result)) = receiver.try_recv() {
            if result.is_ok() {
                let data = buffer_slice.get_mapped_range();
                let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

                drop(data);
                staging_buffer.unmap();

                return image::ImageBuffer::from_raw(gpu_buffer.width, gpu_buffer.height, result);
            } else {
                return None;
            }
        }
        // If we have the submission index, we could use Wait, but Poll works in a loop too.
        // Using Poll allows us to check the receiver.
        // If we use Wait, we block.
        // Let's stick to the loop with Poll for robustness as verified before,
        // but now we are polling for the STAGING buffer which is correct.
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

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
        read_gpu_buffer(context, &gpu_buffer).await
    }
}

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
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light_leak.wgsl").into()),
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

#[cfg(feature = "compute-gpu")]
pub struct HalationPipeline {
    pipeline_x: wgpu::ComputePipeline,
    pipeline_y: wgpu::ComputePipeline,
    bind_group_layout_x: wgpu::BindGroupLayout,
    bind_group_layout_y: wgpu::BindGroupLayout,
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

        let layout_x = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Halation X Layout"),
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

        let layout_y = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Halation Y Layout"),
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

        let pipeline_layout_x =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Halation X Pipeline Layout"),
                    bind_group_layouts: &[&layout_x],
                    immediate_size: 0,
                });

        let pipeline_layout_y =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Halation Y Pipeline Layout"),
                    bind_group_layouts: &[&layout_y],
                    immediate_size: 0,
                });

        let pipeline_x = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Halation Pipeline X"),
                layout: Some(&pipeline_layout_x),
                module: &shader,
                entry_point: Some("main_x"),
                compilation_options: Default::default(),
                cache: None,
            });

        let pipeline_y = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Halation Pipeline Y"),
                layout: Some(&pipeline_layout_y),
                module: &shader,
                entry_point: Some("main_y"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            pipeline_x,
            pipeline_y,
            bind_group_layout_x: layout_x,
            bind_group_layout_y: layout_y,
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

        let temp_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Halation Temp Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Halation Output Buffer"),
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
        unsafe impl bytemuck::Zeroable for Uniforms {}
        unsafe impl bytemuck::Pod for Uniforms {}

        let sigma = width as f32 * film.halation_sigma;
        let uniforms = Uniforms {
            width,
            height,
            threshold: film.halation_threshold,
            _pad1: 0.0,
            sigma,
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

        let bg_x = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Halation BG X"),
                layout: &self.bind_group_layout_x,
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
                label: Some("Halation BG Y"),
                layout: &self.bind_group_layout_y,
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

        let x_groups = width.div_ceil(16);
        let y_groups = height.div_ceil(16);

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Halation Pass X"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline_x);
            pass.set_bind_group(0, &bg_x, &[]);
            pass.dispatch_workgroups(x_groups, y_groups, 1);
        }

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Halation Pass Y"),
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
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blur.wgsl").into()),
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GrainUniforms {
    width: u32,
    height: u32,
    seed: f32,
    alpha: f32,
    sigma_read: f32,
    roughness: f32,
    monochrome: u32,
    _pad: f32,
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
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/grain.wgsl").into()),
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
            // Copy input to output
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

        let seed = 1234.5678; // TODO: Pass random seed

        // Scale grain parameters based on resolution
        // Reference: 2048px width (approx 2K scan)
        const REFERENCE_WIDTH: f32 = 2048.0;
        let scale_factor = width as f32 / REFERENCE_WIDTH;

        // Scale noise amplitude to maintain perceived granularity density
        // Var = alpha * D^1.5 + sigma^2
        // We want std_dev to scale linearly with resolution scale (to counter averaging)
        // So Variance scales with square of resolution scale
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
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/develop.wgsl").into()),
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
                }
            }
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        struct Uniforms {
            matrix_r: [f32; 4],
            matrix_g: [f32; 4],
            matrix_b: [f32; 4],
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

        let m = film.color_matrix;
        let uniforms = Uniforms {
            matrix_r: [m[0][0], m[0][1], m[0][2], 0.0],
            matrix_g: [m[1][0], m[1][1], m[1][2], 0.0],
            matrix_b: [m[2][0], m[2][1], m[2][2], 0.0],
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
