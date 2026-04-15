use image::{Rgb, RgbImage};

/// Camera shake trajectory: a sequence of (x, y, dwell_weight) points.
/// Models physiological hand tremor (8-12Hz sinusoidal) + slow drift.
/// The trajectory represents the optical axis displacement on the film plane.
struct ShakeTrajectory {
    points: Vec<(f32, f32, f32)>, // (x_px, y_px, dwell_weight)
}

impl ShakeTrajectory {
    /// Generate a realistic camera shake trajectory.
    ///
    /// - `amplitude_px`: max displacement in pixels
    /// - `n_samples`: number of time samples during exposure
    fn generate(amplitude_px: f32, n_samples: usize) -> Self {
        let exposure_time = 1.0 / 60.0f32;
        let dt = exposure_time / n_samples as f32;
        let tau = std::f32::consts::TAU;

        // Main tremor: 8-12Hz
        let freq_x = 8.0 + rand::random::<f32>() * 4.0;
        let freq_y = 8.0 + rand::random::<f32>() * 4.0;
        let phase_x: f32 = rand::random::<f32>() * tau;
        let phase_y: f32 = rand::random::<f32>() * tau;

        // Drift: very slow, ~10% of amplitude over exposure
        let drift_vx = (rand::random::<f32>() - 0.5) * amplitude_px * 0.1 / exposure_time;
        let drift_vy = (rand::random::<f32>() - 0.5) * amplitude_px * 0.1 / exposure_time;

        // Amplitude per axis
        let ax = amplitude_px * (0.3 + rand::random::<f32>() * 0.7);
        let ay = amplitude_px * (0.3 + rand::random::<f32>() * 0.7);

        // Secondary frequencies (random phases)
        let phase_x2: f32 = rand::random::<f32>() * tau;
        let phase_y2: f32 = rand::random::<f32>() * tau;
        let phase_x3: f32 = rand::random::<f32>() * tau;
        let phase_y3: f32 = rand::random::<f32>() * tau;

        let mut points = Vec::with_capacity(n_samples);
        let mut prev_x = 0.0f32;
        let mut prev_y = 0.0f32;

        for i in 0..n_samples {
            let t = i as f32 * dt;

            // Multi-frequency tremor + drift
            // f1: 8-12Hz main (100%), f2: 2-4Hz slow sway (20%), f3: 18-25Hz micro (8%)
            let x = ax * (tau * freq_x * t + phase_x).sin()
                + ax * 0.2 * (tau * freq_x * 0.3 * t + phase_x2).sin()
                + ax * 0.08 * (tau * freq_x * 2.2 * t + phase_x3).sin()
                + drift_vx * t;
            let y = ay * (tau * freq_y * t + phase_y).sin()
                + ay * 0.2 * (tau * freq_y * 0.3 * t + phase_y2).sin()
                + ay * 0.08 * (tau * freq_y * 2.2 * t + phase_y3).sin()
                + drift_vy * t;

            // Speed (pixels per second)
            let speed = ((x - prev_x).powi(2) + (y - prev_y).powi(2)).sqrt() / dt;

            // Shutter curtain: 10% ramp at each end
            let frac = i as f32 / n_samples as f32;
            let ramp = 0.1;
            let curtain = if frac < ramp {
                frac / ramp
            } else if frac > 1.0 - ramp {
                (1.0 - frac) / ramp
            } else {
                1.0
            };

            // Dwell = curtain × (1/speed). Epsilon proportional to typical speed.
            let typical_speed = amplitude_px * freq_x * tau; // rough estimate
            let dwell = curtain / (speed + typical_speed * 0.01);

            prev_x = x;
            prev_y = y;
            points.push((x, y, dwell));
        }

        // Normalize dwell weights to sum to 1
        let total: f32 = points.iter().map(|p| p.2).sum();
        if total > 0.0 {
            for p in points.iter_mut() {
                p.2 /= total;
            }
        }

        Self { points }
    }

    /// Render trajectory as a visualization image.
    /// White background, trajectory drawn with brightness proportional to dwell weight.
    fn visualize(&self, size: u32) -> RgbImage {
        let mut img = RgbImage::from_pixel(size, size, Rgb([255, 255, 255]));
        let center = size as f32 / 2.0;

        // Find max dwell for normalization
        let max_dwell = self.points.iter().map(|p| p.2).fold(0.0f32, f32::max);

        // Draw trajectory line (connect consecutive points)
        for i in 0..self.points.len() {
            let (x, y, dwell) = self.points[i];
            let px = (center + x).round() as i32;
            let py = (center + y).round() as i32;

            // Draw single pixel at each sample point
            // Color: darker = more dwell time (slower), lighter = less dwell (faster)
            let intensity = 1.0 - (dwell / max_dwell).min(1.0);
            let c = (intensity * 200.0) as u8;

            if px >= 0 && px < size as i32 && py >= 0 && py < size as i32 {
                img.put_pixel(px as u32, py as u32, Rgb([c, c, c]));
            }

            // Draw line segment to next point
            if i + 1 < self.points.len() {
                let (nx, ny, _) = self.points[i + 1];
                let npx = (center + nx).round() as i32;
                let npy = (center + ny).round() as i32;
                // Simple Bresenham-ish line
                let steps = ((npx - px).abs().max((npy - py).abs())) as usize + 1;
                for s in 0..=steps {
                    let t = s as f32 / steps.max(1) as f32;
                    let lx = px as f32 + (npx - px) as f32 * t;
                    let ly = py as f32 + (npy - py) as f32 * t;
                    let lxi = lx.round() as i32;
                    let lyi = ly.round() as i32;
                    if lxi >= 0 && lxi < size as i32 && lyi >= 0 && lyi < size as i32 {
                        img.put_pixel(lxi as u32, lyi as u32, Rgb([c, 0, 0])); // red line
                    }
                }
            }
        }

        // Mark start (green) and end (blue)
        if let Some(&(x, y, _)) = self.points.first() {
            let px = (center + x).round().clamp(0.0, size as f32 - 1.0) as u32;
            let py = (center + y).round().clamp(0.0, size as f32 - 1.0) as u32;
            img.put_pixel(px, py, Rgb([0, 200, 0]));
        }
        if let Some(&(x, y, _)) = self.points.last() {
            let px = (center + x).round().clamp(0.0, size as f32 - 1.0) as u32;
            let py = (center + y).round().clamp(0.0, size as f32 - 1.0) as u32;
            img.put_pixel(px, py, Rgb([0, 0, 200]));
        }

        img
    }
}

#[test]
fn visualize_shake_trajectories() {
    let dir = std::path::Path::new("target/test_output");
    std::fs::create_dir_all(dir).unwrap();

    // Generate multiple trajectories with different amplitudes
    for (i, amp) in [3.0, 5.0, 10.0, 20.0].iter().enumerate() {
        let traj = ShakeTrajectory::generate(*amp, 200);

        let _xs: Vec<f32> = traj.points.iter().map(|p| p.0).collect();
        let _ys: Vec<f32> = traj.points.iter().map(|p| p.1).collect();
        let max_disp = traj
            .points
            .iter()
            .map(|p| (p.0 * p.0 + p.1 * p.1).sqrt())
            .fold(0.0f32, f32::max);

        // Compute speeds
        let mut speeds: Vec<f32> = Vec::new();
        for j in 1..traj.points.len() {
            let dx = traj.points[j].0 - traj.points[j - 1].0;
            let dy = traj.points[j].1 - traj.points[j - 1].1;
            speeds.push((dx * dx + dy * dy).sqrt());
        }
        let max_speed = speeds.iter().cloned().fold(0.0f32, f32::max);

        println!(
            "Trajectory {} (amp={:.0}px): max_r={:.1} max_speed={:.2} avg_speed={:.2}",
            i,
            amp,
            max_disp,
            max_speed,
            speeds.iter().sum::<f32>() / speeds.len().max(1) as f32,
        );

        // Trajectory image
        let vis = traj.visualize(200);
        vis.save(dir.join(format!("shake_trajectory_{}_amp{}.png", i, amp)))
            .unwrap();

        // Dwell weight chart: X=time, Y=dwell weight, blue=low red=high
        let chart_w = 400u32;
        let chart_h = 150u32;
        let mut chart = RgbImage::from_pixel(chart_w, chart_h, Rgb([255, 255, 255]));
        let dwells: Vec<f32> = traj.points.iter().map(|p| p.2).collect();
        let max_dwell = dwells.iter().cloned().fold(0.0f32, f32::max);
        if !dwells.is_empty() && max_dwell > 0.0 {
            let bar_w = chart_w as f32 / dwells.len() as f32;
            for (j, &dw) in dwells.iter().enumerate() {
                let bar_h = (dw / max_dwell * (chart_h - 10) as f32) as u32;
                let x0 = (j as f32 * bar_w) as u32;
                let x1 = ((j + 1) as f32 * bar_w) as u32;
                let t = dw / max_dwell;
                let r = (t * 255.0) as u8;
                let b = ((1.0 - t) * 255.0) as u8;
                for y in (chart_h - bar_h)..chart_h {
                    for x in x0..x1.min(chart_w) {
                        chart.put_pixel(x, y, Rgb([r, 0, b]));
                    }
                }
            }
        }
        chart
            .save(dir.join(format!("shake_dwell_{}_amp{}.png", i, amp)))
            .unwrap();
    }

    println!("Trajectory images saved to target/test_output/shake_trajectory_*.png");
    println!("Green=start, Blue=end, Dark=slow(high dwell), Light=fast(low dwell)");
}
