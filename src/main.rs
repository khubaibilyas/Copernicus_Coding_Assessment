mod compare;
mod convolution;
mod nms;
mod sobel;
mod threshold;

use anyhow::{Context, Result};
use clap::Parser;
use image::{GrayImage, ImageReader};
use std::path::PathBuf;

use nms::non_maximum_suppression;
use sobel::sobel_gradients;
use threshold::{auto_threshold, hysteresis_threshold};

// CLI struct to parse cli arguments.
#[derive(Parser, Debug)]
#[command(
    name = "edge-detect",
    version,
    about = "Mini edge detection pipeline: Sobel → NMS → hysteresis threshold"
)]
struct Cli {
    // Input image path. Accepts any format supported by the `image` crate. Colour images are converted to greyscale.
    #[arg(short, long)]
    input: PathBuf,

    // Output image path.
    #[arg(short, long)]
    output: PathBuf,

    // Edge magnitude threshold for hysteresis. When omitted, an automatic threshold is selected from the 85th
    //percentile of non-zero NMS values.
    #[arg(short, long)]
    threshold: Option<f32>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Step 1 — Load image and convert to greyscale. Use f32 for image processing.

    let img = ImageReader::open(&cli.input)
        .with_context(|| format!("cannot open '{}'", cli.input.display()))?
        .decode()
        .with_context(|| format!("cannot decode '{}'", cli.input.display()))?
        .into_luma8(); // convert to 8-bit greyscale.

    let width = img.width() as usize;
    let height = img.height() as usize;

    //Use 1D image buffer for easier memory allocation on heap. Index is from 0-height*width.
    let pixels_f32: Vec<f32> = img.as_raw().iter().map(|&p| p as f32).collect();

    println!("Loaded  : '{}' ({}×{})", cli.input.display(), width, height);

    // Steps 2 & 3 — Compute Sobel magnitude and orientation using the convolve2D interface.

    let sobel = sobel_gradients(&pixels_f32, width, height);

    // Step 4 — Non-maximum suppression.

    let suppressed = non_maximum_suppression(&sobel.magnitude, &sobel.direction, width, height);
    println!("NMS     : Applied");

    // Step 5 — Hysteresis Thresholding.

    let high_thresh : f32 = if cli.threshold.is_some() {
        print!("Threshold (user): ");
        cli.threshold.unwrap()
    }
    else{
        print!("Threshold (auto): ");
        auto_threshold(&suppressed)
    };

    println!("{:.2}", &high_thresh);

    // We take lower threshold of hysteresis is 33% of the high threshold.
    let low_thresh = high_thresh * 0.33;
    let edge_image: Vec<u8> = hysteresis_threshold(&suppressed, width, height, low_thresh, high_thresh);

    // Save the output  image.

    let out_img = GrayImage::from_raw(width as u32, height as u32, edge_image)
        .ok_or_else(|| anyhow::anyhow!("buffer size mismatch when creating output image"))?;

    out_img
        .save(&cli.output)
        .with_context(|| format!("cannot save output to '{}'", cli.output.display()))?;

    println!("Saved   : '{}'", cli.output.display());
    Ok(())
}
