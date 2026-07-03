use rayon::prelude::*;

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

    output
        .par_iter_mut()
        .enumerate()
        .for_each(|(pixel_idx, mut out_pixel)| {
            let y = pixel_idx / width;
            let x = pixel_idx % width;

            *out_pixel = 0.0_f32;
            //Not sure if this can be parallelised as well.
            for ky in 0..kh{
                for kx in 0..kw{
                    let iy = y as isize + (ky as isize - half_kh);
                    let ix = x as isize + (kx as isize - half_kw);

                    if iy >= 0 && iy < height as isize && ix >= 0 && ix < width as isize {
                        //Source pixel and kernel values by wrapping y and x into 1D array idx.
                        *out_pixel += image[iy as usize * width + ix as usize] * kernel[ky * kw + kx];
                    }
                }
            }
        });

    output
}