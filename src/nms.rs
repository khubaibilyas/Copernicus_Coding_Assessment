use std::f32::consts::PI;

// Non-maximum suppression on a gradient magnitude image.

pub fn non_maximum_suppression(
    magnitude: &[f32],
    direction: &[f32],
    width: usize,
    height: usize,
) -> Vec<f32> {
    assert_eq!(magnitude.len(), width * height);
    assert_eq!(direction.len(), width * height);

    let mut output = vec![0.0_f32; width * height];

    // Skip the boundary pixel
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let idx = y * width + x;
            let mag = magnitude[idx];

            // Since we check both opposite pixels along the gradient vector, we can convert angle to degrees in [0, 180).
            // If gradient direction is negative, we add 180 to bring it to the range (-45 and 135 are collinear)
            let deg = if direction[idx] > 0.0{
                (direction[idx] * 180.0 / PI) % 180.0
            } else {  (direction[idx] * 180.0 / PI) % 180.0 + 180.0 };

            // Select the two neighbours along the quantised gradient axis.
            let (n1, n2) = if deg < 22.5 || deg >= 157.5 {
                (magnitude[idx - 1], magnitude[idx + 1])
            } else if deg < 67.5 {
                (
                    magnitude[(y - 1) * width + (x + 1)],
                    magnitude[(y + 1) * width + (x - 1)],
                )
            } else if deg < 112.5 {
                (
                    magnitude[(y - 1) * width + x],
                    magnitude[(y + 1) * width + x],
                )
            } else {
                (
                    magnitude[(y - 1) * width + (x - 1)],
                    magnitude[(y + 1) * width + (x + 1)],
                )
            };

            // Keep the pixel only if it is a local maximum.
            if mag >= n1 && mag >= n2 {
                output[idx] = mag;
            }
        }
    }

    output
}