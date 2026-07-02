//! Batch comparison tool: stitches every predicted edge map next to its
//! ground-truth counterpart and saves the result to an output directory.
//!
//! Both directories must contain files with **identical names**.
//! Files present in `--left-dir` but missing from `--right-dir` are skipped
//! with a warning; mismatched image sizes are handled gracefully by
//! `compare_with_gt` (height is padded with black, widths are independent).
//!
//! Build & run:
//! ```
//! cargo build --release
//! .\target\release\compare-pairs.exe \
//!     --left-dir  output_imgs \
//!     --right-dir UDED\gt \
//!     --output-dir comparisons
//! ```

// Pull in only what this binary needs.
mod compare;

use clap::Parser;
use compare::compare_with_gt;
use image::open;
use std::path::PathBuf;

/// Stitch every predicted edge map (left) next to its GT image (right)
/// and save the side-by-side panel to `--output-dir`.
#[derive(Parser, Debug)]
#[command(
    name  = "compare-pairs",
    about = "Side-by-side comparison of predicted edges vs ground truth",
    version
)]
struct Cli {
    /// Directory containing predicted edge maps (left panel).
    #[arg(long, default_value = "output_imgs")]
    left_dir: PathBuf,

    /// Directory containing ground-truth images (right panel, labelled "GT").
    #[arg(long, default_value = r"UDED\gt")]
    right_dir: PathBuf,

    /// Output directory for the comparison panels.
    #[arg(long, default_value = "comparisons")]
    output_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Create output directory if it doesn't exist.
    std::fs::create_dir_all(&cli.output_dir)?;

    // Collect all PNG files in the left directory, sorted by name.
    let mut entries: Vec<_> = std::fs::read_dir(&cli.left_dir)
        .map_err(|e| anyhow::anyhow!("Cannot read left-dir '{}': {e}", cli.left_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    println!(
        "Comparing {} image(s) from '{}' → '{}'",
        entries.len(),
        cli.left_dir.display(),
        cli.output_dir.display()
    );
    println!("{}", "─".repeat(60));

    let mut saved  = 0u32;
    let mut skipped = 0u32;

    for entry in &entries {
        let filename   = entry.file_name();
        let left_path  = entry.path();
        let right_path = cli.right_dir.join(&filename);
        let out_path   = cli.output_dir.join(&filename);

        // Skip if the GT counterpart doesn't exist.
        if !right_path.exists() {
            eprintln!("  [skip] no GT match for {:?}", filename);
            skipped += 1;
            continue;
        }

        // Load both images as greyscale.
        let predicted = open(&left_path)
            .map_err(|e| anyhow::anyhow!("Cannot open '{}': {e}", left_path.display()))?
            .into_luma8();

        let gt = open(&right_path)
            .map_err(|e| anyhow::anyhow!("Cannot open '{}': {e}", right_path.display()))?
            .into_luma8();

        // Stitch and label.
        let panel = compare_with_gt(&predicted, &gt);

        panel
            .save(&out_path)
            .map_err(|e| anyhow::anyhow!("Cannot save '{}': {e}", out_path.display()))?;

        println!(
            "  {:>30}  →  {}×{}",
            filename.to_string_lossy(),
            panel.width(),
            panel.height()
        );
        saved += 1;
    }

    println!("{}", "─".repeat(60));
    println!(
        "Done: {} saved, {} skipped  →  '{}'",
        saved,
        skipped,
        cli.output_dir.display()
    );

    Ok(())
}
