# edge-detect

A command-line edge detection pipeline written in Rust, implemented as a take-home exercise.  
The pipeline follows the classical Canny architecture: Gaussian smoothing → gradient computation (Scharr operator) → non-maximum suppression → hysteresis thresholding.

---

## Structure

The project is split into a library crate (`edge_detect`) and a thin binary (`edge-detect`), organised into five modules:

| Module | Responsibility |
|---|---|
| `main` | CLI parsing, I/O, pipeline orchestration |
| `convolution` | Generic 2D convolution over flat row-major buffers |
| `sobel` | Gradient magnitude and orientation via the Scharr operator |
| `nms` | Non-maximum suppression |
| `threshold` | Hysteresis thresholding |

Each module is independently unit-testable. The Scharr operator is used in place of standard Sobel because it better approximates the true image gradient at diagonal orientations. Hysteresis thresholding is applied in the final stage.

---

## Usage

```bash
cargo run --release --bin edge-detect -- \
  --input <input_img> \
  --output <output_img> \
  [--threshold <value>]
```

**Example**

```bash
cargo run --release --bin edge-detect -- \
  --input UDED/imgs/12-cameraman.png \
  --output output_imgs/12-cameraman_edges.png \
  --threshold 0.15
```

---

## Evaluation

Ground truth edge images were sourced from the [UDED dataset](https://github.com/xavysp/UDED).

- **`compare.rs`** — places the algorithm output and ground truth side by side for visual inspection (generated with LLM assistance).
- **`run_pipeline.bat`** — runs the full pipeline over every image in `UDED/imgs/`, writing edge outputs to `output_imgs/` and side-by-side comparisons to `comparisons/`.

---

## Resources

Videos that were useful for getting up to speed with Rust:

- [The Rust Programming Language — Introduction](https://www.youtube.com/watch?v=0y6RKiIk6cs)
- [Rust Crash Course](https://www.youtube.com/watch?v=br3GIIQeefY)
- [Rust for Beginners](https://www.youtube.com/watch?v=784JWR4oxOI)

Lessons learned are documented in [this Google Doc](https://docs.google.com/document/d/1RCHmQ-_pXGLGYKczTezbK-9o2UkHUHrFaMUogIkCFdM/edit?usp=sharing).

---

## What I would improve given more time

1. **In-place buffer mutation across pipeline stages.**  
   Currently each stage allocates and returns a new buffer, taking ownership of the previous stage's output. A more efficient design would pass a mutable reference to a shared output buffer and write into it in place, eliminating intermediate heap allocations.

2. **Fixed-point intermediate arithmetic.**  
   The pipeline carries `f32` buffers throughout, which is convenient but memory-intensive. Convolution intermediates can exceed the `[0, 255]` range, so true 8-bit arithmetic is not straightforward — but a fixed-point representation with appropriate scaling would reduce memory footprint significantly at the cost of some implementation complexity.

3. **Configurable pipeline parameters.**  
   Kernel size, border padding mode, and thresholding strategy are currently fixed at compile time. Exposing these through the CLI would make the system more flexible and make it easier to evaluate trade-offs against ground truth data.

4. **Static allocation for safety-critical deployment.**  
   The current design allocates pixel buffers on the heap at runtime. For a safety-critical embedded context (ISO 26262 ASIL-B and above), all memory should be allocated statically at startup with no dynamic allocation during operation.

5. **Numerical evaluation pipeline.**  
   Visual inspection against UDED ground truth is useful but not rigorous. A proper evaluation would compute standard edge detection metrics (F-measure, ODS, OIS) against the GT masks, enabling regression testing and quantitative comparison between parameter settings.

6. **Multi-scale edge detection.**  
   A single Gaussian pre-blur at one sigma makes the detector sensitive to edges at one spatial scale only. Applying the pipeline at multiple sigma values and fusing the results would improve robustness to fine versus coarse texture.

7. **Edge linking and length filtering.**  
   The current output contains short, disconnected edge fragments that are perceptually meaningless. A post-processing pass that connects nearby edge endpoints and discards contours below a minimum length threshold would substantially improve output quality.

8. **2D indexing ergonomics.**  
   Pixel buffers are stored as flat `Vec<f32>` with row-major indexing (`row * width + col`), which gives a single contiguous heap allocation and good cache locality. The [`ndarray`](https://docs.rs/ndarray) crate provides a proper N-dimensional array abstraction with native 2D indexing that would eliminate the manual stride arithmetic, at the cost of an additional dependency. For this exercise, the flat `Vec` is sufficient; `ndarray` would be the right choice in a larger production codebase.

---

## Rust vs C++ — most idiomatic and most awkward

**Most idiomatic: the iterator ecosystem**

The most refreshing aspect of the implementation was Rust's iterator ecosystem. In C++, operating on image buffers typically means nested `for` loops with manual index tracking — a constant source of off-by-one errors and silent out-of-bounds accesses. Rust's functional style — chaining `.iter().zip().map().collect()` to compute gradient magnitude and direction — eliminates manual indexing entirely and lets the code express the mathematical transformation directly, without burying it in the mechanics of array traversal.

**Most awkward: the absence of implicit numeric promotion**

In C++ you can freely mix `int`, `size_t`, `float`, and `unsigned` in arithmetic expressions and the compiler silently promotes them. In Rust, every boundary between `u32` (image dimensions from the `image` crate), `usize` (array indexing), `isize` (signed offsets for kernel neighbours), and `f32` (pixel values) requires an explicit `as` cast. The convolution inner loop alone accumulates six such casts that would be invisible in equivalent C++. Rust is correct to require this — silent `size_t`-to-`int` narrowing is a real and common C++ bug — but it made the low-level image processing code noticeably more cluttered than its C++ counterpart.