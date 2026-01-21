/// Film Modeling Module
/// 
/// Handles Characteristic Curves (H-D Curves) and Color Coupling.
/// Section 3 & 5 of the technical document.

#[derive(Debug, Clone, Copy)]
pub struct SegmentedCurve {
    pub d_min: f32,
    pub d_max: f32,
    pub gamma: f32,
    pub exposure_offset: f32, // E0 in the doc, controls speed
}

impl SegmentedCurve {
    pub fn new(d_min: f32, d_max: f32, gamma: f32, exposure_offset: f32) -> Self {
        Self {
            d_min,
            d_max,
            gamma,
            exposure_offset,
        }
    }

    /// Maps log10(Exposure) to Density.
    /// Implements a simplified sigmoid-like S-curve based on the segmented model logic
    /// but smoothed for better visual results if exact break points aren't provided.
    pub fn map(&self, log_e: f32) -> f32 {
        // Simplified implementation:
        // Linear region: D = D_min + gamma * (log_e - log_e0)
        // We clamp it to [D_min, D_max] and add soft knees.
        
        let log_e0 = self.exposure_offset.log10();
        let linear_d = self.d_min + self.gamma * (log_e - log_e0);
        
        // Midpoint of linear section:
        let d_mid = (self.d_min + self.d_max) / 2.0;
        let log_e_mid = log_e0 + (d_mid - self.d_min) / self.gamma;
        
        let toe_limit = log_e_mid - 0.7; // arbitrary soft knee start
        let shoulder_limit = log_e_mid + 0.7;
        
        if log_e > toe_limit && log_e < shoulder_limit {
            // Linear Region
             self.d_min + self.gamma * (log_e - log_e0)
        } else if log_e <= toe_limit {
            // Toe Region
            if log_e < log_e0 {
                // Hard floor at D_min for very low exposure
                self.d_min.max(linear_d)
            } else {
                 linear_d
            }
        } else {
            // Shoulder Region
            if linear_d > self.d_max {
                self.d_max
            } else {
                linear_d
            }
        }
    }
    
    /// A smoother implementation using interpolation, closer to real film.
    pub fn map_smooth(&self, log_e: f32) -> f32 {
        let log_e0 = self.exposure_offset.log10();
        let x = log_e - log_e0;
        
        // A sigmoid that goes from D_min to D_max with slope gamma at origin
        // y = D_min + (D_max - D_min) * (1 / (1 + exp(-k * x)))
        // Derivative y' = range * k * sigmoid * (1-sigmoid). At x=0, sigmoid=0.5.
        // y'(0) = range * k * 0.25 = gamma
        // k = 4 * gamma / range
        
        let range = self.d_max - self.d_min;
        if range <= 0.0 { return self.d_min; }
        
        let k = 4.0 * self.gamma / range;
        
        let sigmoid = 1.0 / (1.0 + (-k * x).exp());
        self.d_min + range * sigmoid
    }
}

/// Represents a specific film stock with 3 curves and a color matrix.
pub struct FilmStock {
    pub r_curve: SegmentedCurve,
    pub g_curve: SegmentedCurve,
    pub b_curve: SegmentedCurve,
    
    // 3x3 Matrix for crosstalk. Rows: R_out, G_out, B_out. Cols: R_in, G_in, B_in.
    // D_out = Matrix * D_in
    pub color_matrix: [[f32; 3]; 3],
}

impl FilmStock {
    pub fn new_standard_daylight() -> Self {
        let _base_curve = SegmentedCurve::new(0.1, 3.0, 1.8, 0.18); // D_min=0.1, D_max=3.0, Gamma=1.8 (high contrast), ISO base
        
        // Slight variations per channel to mimic film look (warm shadows, cool highlights etc)
        let r_curve = SegmentedCurve::new(0.12, 2.9, 1.8, 0.18);
        let g_curve = SegmentedCurve::new(0.10, 3.0, 1.8, 0.18);
        let b_curve = SegmentedCurve::new(0.11, 2.8, 1.7, 0.18);
        
        // Crosstalk matrix (Diagonal = 1.0 means no extra mixing if applied after curve)
        // Real film has some subtractive interaction.
        // Let's assume a small amount of crosstalk.
        let matrix = [
            [1.00, 0.05, 0.02],
            [0.04, 1.00, 0.04],
            [0.01, 0.05, 1.00],
        ];
        
        Self {
            r_curve,
            g_curve,
            b_curve,
            color_matrix: matrix,
        }
    }
    
    /// Apply the film simulation to RGB log-exposures
    pub fn map_log_exposure(&self, log_e: [f32; 3]) -> [f32; 3] {
        // 1. Map each channel through its curve (Simulates Section 3)
        let d_r = self.r_curve.map_smooth(log_e[0]);
        let d_g = self.g_curve.map_smooth(log_e[1]);
        let d_b = self.b_curve.map_smooth(log_e[2]);
        
        // 2. Apply Color Matrix (Simulates Section 5 - Layer Coupling)
        // [Dr']   [ M00 M01 M02 ] [ Dr ]
        // [Dg'] = [ M10 M11 M12 ] [ Dg ]
        // [Db']   [ M20 M21 M22 ] [ Db ]
        
        let d_r_out = self.color_matrix[0][0] * d_r + self.color_matrix[0][1] * d_g + self.color_matrix[0][2] * d_b;
        let d_g_out = self.color_matrix[1][0] * d_r + self.color_matrix[1][1] * d_g + self.color_matrix[1][2] * d_b;
        let d_b_out = self.color_matrix[2][0] * d_r + self.color_matrix[2][1] * d_g + self.color_matrix[2][2] * d_b;
        
        [d_r_out, d_g_out, d_b_out]
    }
}
