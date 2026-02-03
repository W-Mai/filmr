use crate::gpu::GpuContext;
use image::{ImageBuffer, Rgb, RgbImage};

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
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba32Float,
                                view_dimension: wgpu::TextureViewDimension::D2,
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
        let width = input.width();
        let height = input.height();

        // 1. Convert to RGBA
        let mut rgba_input = Vec::with_capacity((width * height * 4) as usize);
        for pixel in input.pixels() {
            rgba_input.push(pixel[0]);
            rgba_input.push(pixel[1]);
            rgba_input.push(pixel[2]);
            rgba_input.push(255);
        }

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // 2. Create and Upload Input Texture
        let input_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Input Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &input_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba_input,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        // 3. Create Output Texture
        let output_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let input_view = input_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 4. Bind Group
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Linearize Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&output_view),
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

        // 6. Readback Preparation
        let unpadded_bytes_per_row = width * 16; // 4 * f32 = 16 bytes per pixel
        let align = 256;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

        let output_buffer_size = (padded_bytes_per_row * height) as u64;
        let output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            texture_size,
        );

        context.queue.submit(Some(encoder.finish()));

        // 7. Map and Read
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        context
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .unwrap();

        if let Ok(Ok(())) = receiver.recv() {
            let data = buffer_slice.get_mapped_range();
            // Convert padded data to linear image
            let mut result_image: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

            for y in 0..height {
                let row_offset = (y * padded_bytes_per_row) as usize;
                for x in 0..width {
                    let pixel_offset = row_offset + (x * 16) as usize;
                    let r = f32::from_le_bytes(
                        data[pixel_offset..pixel_offset + 4].try_into().unwrap(),
                    );
                    let g = f32::from_le_bytes(
                        data[pixel_offset + 4..pixel_offset + 8].try_into().unwrap(),
                    );
                    let b = f32::from_le_bytes(
                        data[pixel_offset + 8..pixel_offset + 12]
                            .try_into()
                            .unwrap(),
                    );
                    // Alpha is ignored

                    result_image.put_pixel(x, y, Rgb([r, g, b]));
                }
            }

            drop(data);
            output_buffer.unmap();
            return Some(result_image);
        }

        None
    }
}
