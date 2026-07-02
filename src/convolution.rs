// 2D discrete convolution of a single-channel image with a given kernel.
// I use zero-padding around the borders. If time permits, I may extend functionality to replicating pixels around borders.
pub fn convolve2d(
    image: &[f32],
    width: usize,
    height: usize,
    kernel: &[f32],
    kw: usize,
    kh: usize,
) -> Vec<f32> {
    assert!(kw % 2 == 1, "kernel width must be odd, got {kw}");
    assert!(kh % 2 == 1, "kernel height must be odd, got {kh}");
    assert_eq!(
        kernel.len(),
        kw * kh,
        "kernel buffer length {} does not match kw*kh={}",
        kernel.len(),
        kw * kh
    );
    assert_eq!(
        image.len(),
        width * height,
        "image buffer length {} does not match width*height={}",
        image.len(),
        width * height
    );

    let half_kw = (kw / 2) as isize;
    let half_kh = (kh / 2) as isize;

    let mut output = vec![0.0_f32; width * height];

    for y in 0..height {
        for x in 0..width {
            let mut acc = 0.0_f32;

            for ky in 0..kh {
                for kx in 0..kw {
                    // Source pixel coordinates with offset from the kernel centre.
                    let iy = y as isize + (ky as isize - half_kh);
                    let ix = x as isize + (kx as isize - half_kw);

                    // If index goes out of bounds, we do not do anything to the accumulator as they are zero-padded.
                    // For replicate, we will have to change this logic.
                    if iy >= 0 && iy < height as isize && ix >= 0 && ix < width as isize {
                        //Source pixel and kernel values by wrapping y and x into 1D array idx.
                        let pixel_val = image[iy as usize * width + ix as usize];
                        let kernel_val = kernel[ky * kw + kx];
                        acc += pixel_val * kernel_val;
                    }
                }
            }

            output[y * width + x] = acc;
        }
    }

    output
}