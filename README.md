# edge-detect

A command-line edge detection pipeline written in Rust.

## What it does

```
Input image → Greyscale → Sobel gradients → Non-maximum suppression → Threshold → Edge map
```

| Step | Detail |
|------|--------|
| **Load** | Any format supported by the `image` crate (PNG, JPEG, BMP, PGM…); colour images are auto-converted to greyscale. |
| **Convolve** | Custom 2-D discrete convolution implemented from scratch in `src/convolution.rs`. No library convolution function is used. |
| **Sobel gradients** | Gx and Gy computed via two convolutions with the standard 3×3 Sobel kernels. Gradient magnitude = √(Gx²+Gy²); direction = atan2(Gy, Gx). |
| **Non-maximum suppression** | The gradient direction is quantised to one of four axes (0°, 45°, 90°, 135°). A pixel is kept only if its magnitude exceeds both neighbours along that axis. This thins ridges to single-pixel edges. |
| **Threshold** | Single hard threshold (default) or Canny-style hysteresis with low = 0.4 × high. An automatic threshold (70th percentile of non-zero NMS values) is used when `--threshold` is not specified. |
| **Save** | Binary greyscale PNG (255 = edge, 0 = background). |

---

## Build

```bash
cargo build --release
```

Requires stable Rust (≥ 1.85 for the 2024 edition).  No `unsafe` blocks are used.

---

## Run

```bash
# Automatic threshold
cargo run --release -- --input UDED/imgs/12-cameraman.png --output edges.png

# Explicit threshold
cargo run --release -- --input UDED/imgs/12-cameraman.png --output edges.png --threshold 80

# Hysteresis thresholding (Canny-style)
cargo run --release -- --input UDED/imgs/12-cameraman.png --output edges.png --threshold 80 --hysteresis

# Short flags also work
cargo run --release -- -i input.png -o output.png -t 100
```

After `cargo build --release` the binary is at `target/release/edge-detect`:

```bash
./target/release/edge-detect --input img.png --output edges.png
```

---

## Run tests

```bash
cargo test
```

Tests cover:

- `convolution` — identity kernel, zero kernel, box blur, Sobel on flat image, 5×5 kernel.
- `sobel` — zero gradient on flat image, high Gx on a hard vertical edge.
- `nms` — single peak survives, isolated zero field stays zero.
- `threshold` — single threshold binarisation, hysteresis propagation chain, isolated weak pixel suppression, auto-threshold range.

---

## CLI reference

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--input`     | `-i` | *(required)* | Path to the input image |
| `--output`    | `-o` | *(required)* | Path for the output PNG edge map |
| `--threshold` | `-t` | auto (70th percentile) | Magnitude threshold on the Sobel scale (roughly 0–1442) |
| `--hysteresis`|  —   | `false` | Enable Canny-style hysteresis thresholding |

---

## Design decisions and trade-offs

### Border handling — zero-padding

When the convolution kernel extends outside the image boundary, the out-of-bounds samples are treated as 0 (zero-padding).  This is the simplest correct choice and is well-documented in the code.  **Alternatives:**

- *Replicate* (clamp to border pixel) reduces the magnitude drop at edges but adds complexity.
- *Reflect* mirrors the image and avoids discontinuities; useful when edge accuracy at the image border matters.

For most practical images the 1-pixel border artefact from zero-padding is invisible.

### Threshold scale

Sobel magnitudes are in the range [0, ≈1442] for 8-bit greyscale input.  The automatic threshold (70th percentile) is computed from the post-NMS non-zero pixels, so it adapts to image contrast without needing manual tuning.

### Hysteresis as a flag

Hysteresis was implemented as an opt-in flag rather than the default, because it introduces a second free parameter (low/high ratio).  The fixed `low = 0.4 × high` ratio follows the convention used in OpenCV's Canny implementation.

### Module structure

```
src/
  main.rs          — CLI (clap derive), pipeline orchestration, I/O
  convolution.rs   — Generic 2-D convolution, unit tests
  sobel.rs         — Sobel Gx/Gy kernels, magnitude + direction
  nms.rs           — Non-maximum suppression, unit tests
  threshold.rs     — Single threshold, hysteresis flood-fill, auto-threshold
```

Each module has a single clear responsibility and is tested independently.

---

## What I would improve with more time

1. **Gaussian pre-smoothing** — A 5×5 Gaussian blur before Sobel would reduce noise sensitivity. The convolution machinery is already in place; it's just one more kernel call.
2. **Parallelism with Rayon** — The convolution double-loop is embarrassingly parallel. Adding `use rayon::prelude::*; (0..height).into_par_iter()` would be a near-zero-risk speedup.
3. **Generic pixel type** — Parameterising `convolve2d` over `T: Into<f32>` via a trait bound would allow the same function to handle `u8`, `u16`, and `f32` images without duplication.
4. **Criterion benchmarks** — The convolution step is the performance bottleneck; a `benches/` crate would make optimisation data-driven.
5. **Replicate border mode** — Higher-quality edges at the image boundary, relevant when the region of interest extends to the frame.
6. **Progress output / logging** — Replace `println!` with the `log` + `env_logger` crates for configurable verbosity.

---

## Rust vs C++ reflections

**Most idiomatic:** The ownership model made the pipeline feel naturally compositional. Each stage takes an immutable reference to the previous stage's output and returns a new owned `Vec<f32>` — no aliasing, no lifetime questions, no manual `free`. Error handling with `anyhow::Result` and the `?` operator gave clean early-returns without the verbosity of C++ exception hierarchies or manual `if (err) return err` chains.

**Most awkward:** The numeric indexing arithmetic (`iy as usize * width + ix as usize`) after signed-to-unsigned casts felt clunky compared to C++ pointer arithmetic. Rust forces you to be explicit about sign conversions, which is safer but verbose. I also noticed that Rust's 2-D indexing (flat `Vec` + manual `y * width + x`) is essentially the same as C, and a proper `ndarray`-style type would read more naturally — though adding that dependency felt out of scope for this exercise.

---

## Dataset

The `UDED/` folder contains the [Unified Dataset for Edge Detection](https://github.com/xavysp/UDED):
30 greyscale images (`imgs/`) paired with binary ground-truth edge maps (`gt/`), drawn from BIPED, BSDS500, CITYSCAPES, and other standard benchmarks.
