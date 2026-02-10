#[cfg(feature = "compute-gpu")]
use crate::gpu::{GpuBuffer, GpuContext};
#[cfg(feature = "compute-gpu")]
use futures::channel::oneshot;

// Wrapper to make WGPU types Send + Sync on WASM
#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
pub struct SendSyncWrapper<T>(pub T);

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
pub fn wrap_pipeline<T>(pipeline: T) -> T {
    pipeline
}

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
pub fn wrap_pipeline<T>(pipeline: T) -> SendSyncWrapper<T> {
    SendSyncWrapper(pipeline)
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
            }
            return None;
        }
        // If we have the submission index, we could use Wait, but Poll works in a loop too.
        // Using Poll allows us to check the receiver.
        // If we use Wait, we block.
        // Let's stick to the loop with Poll for robustness as verified before,
        // but now we are polling for the STAGING buffer which is correct.
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
