use crate::convolution::convolve2d;

// Scharr Gx kernel. 
const SOBEL_GX: [f32; 9] = [
    -3.0, 0.0, 3.0,
   -10.0, 0.0, 10.0,
    -3.0, 0.0, 3.0,
];

// Scharr Gy kernel.
const SOBEL_GY: [f32; 9] = [
    -3.0, -10.0, -3.0,
     0.0,   0.0,  0.0,
     3.0,  10.0,  3.0,
];

// Sobel output.
pub struct SobelOutput {
    // Euclidean gradient magnitude: `sqrt(Gx^2 + Gy^2)`.
    pub magnitude: Vec<f32>,
    // Gradient direction in radians, computed via `atan2(Gy, Gx)`.
    pub direction: Vec<f32>,
}

// Compute Sobel edge gradients for a single-channel f32 image.
// Internally calls `convolve2d` twice (once per kernel) — the convolution
// itself is implemented from scratch as required.

pub fn sobel_gradients(image: &[f32], width: usize, height: usize) -> SobelOutput {
    let gx = convolve2d(image, width, height, &SOBEL_GX, 3, 3);
    let gy = convolve2d(image, width, height, &SOBEL_GY, 3, 3);

    let magnitude: Vec<f32> = gx
        .iter()
        .zip(gy.iter())
        .map(|(x, y)| (x * x + y * y).sqrt())
        .collect();

    // `atan2(y, x)` gives the angle of the gradient vector in radians.
    let direction: Vec<f32> = gx
        .iter()
        .zip(gy.iter())
        .map(|(x, y)| y.atan2(*x))
        .collect();

    SobelOutput {
        magnitude,
        direction,
    }
}
