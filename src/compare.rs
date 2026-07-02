//! Side-by-side image comparison utility.
//!
//! Provides [`place_side_by_side`] to stitch two greyscale images horizontally
//! and render a text label on the right image, and the convenience wrapper
//! [`compare_with_gt`] that labels the right image as "GT".

use image::{GrayImage, Luma};

// ---------------------------------------------------------------------------
// Tiny 5×7 bitmap font (uppercase ASCII)
// ---------------------------------------------------------------------------
//
// Each character is encoded as 7 rows of 5 bits stored in the LOW 5 bits of
// a u8.  Bit 4 (0x10) is the leftmost column, bit 0 (0x01) is the rightmost.
//
// Example — 'T':
//   Row 0: # # # # #  → 0b1_1111 = 0x1F
//   Row 1:     #      → 0b0_0100 = 0x04
//   …

const FONT_COLS: u32 = 5;
const FONT_ROWS: u32 = 7;

/// Returns the 5×7 bitmap for an uppercase ASCII character.
/// Unknown characters fall back to a 5×7 question-mark pattern.
fn char_bitmap(c: char) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        // --- Letters most likely to appear in labels ---
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x10, 0x10, 0x17, 0x11, 0x11, 0x0E], // ← label char
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x11, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04], // ← label char
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        // --- Digits ---
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x06, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        // --- Punctuation / space ---
        ' ' => [0x00; 7],
        ':' => [0x00, 0x04, 0x00, 0x00, 0x04, 0x00, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        // Fallback: question mark
        _   => [0x0E, 0x11, 0x01, 0x06, 0x04, 0x00, 0x04],
    }
}

// ---------------------------------------------------------------------------
// Pixel-level drawing helpers
// ---------------------------------------------------------------------------

/// Render a single character at `(x0, y0)` with the given pixel scale.
/// Each font pixel becomes a `scale × scale` block.
fn draw_char(img: &mut GrayImage, x0: u32, y0: u32, c: char, scale: u32, colour: u8) {
    let bitmap = char_bitmap(c);
    for (row, &bits) in bitmap.iter().enumerate() {
        for col in 0..FONT_COLS {
            // Bit 4 (MSB of the low 5 bits) = leftmost column.
            let on = (bits >> (FONT_COLS - 1 - col)) & 1 == 1;
            if on {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x0 + col * scale + dx;
                        let py = y0 + row as u32 * scale + dy;
                        if px < img.width() && py < img.height() {
                            img.put_pixel(px, py, Luma([colour]));
                        }
                    }
                }
            }
        }
    }
}

/// Render a string of text starting at `(x0, y0)`.
/// Characters are spaced by `(FONT_COLS + 1) * scale` pixels (1-column gap).
fn draw_text(img: &mut GrayImage, x0: u32, y0: u32, text: &str, scale: u32, colour: u8) {
    let char_stride = (FONT_COLS + 1) * scale; // column advance per character
    for (i, c) in text.chars().enumerate() {
        draw_char(img, x0 + i as u32 * char_stride, y0, c, scale, colour);
    }
}

/// Fill a rectangle `[x, x+w) × [y, y+h)` with `colour`, clamped to image bounds.
fn fill_rect(img: &mut GrayImage, x: u32, y: u32, w: u32, h: u32, colour: u8) {
    for py in y..(y + h).min(img.height()) {
        for px in x..(x + w).min(img.width()) {
            img.put_pixel(px, py, Luma([colour]));
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Place two greyscale images side by side and draw a text label on the
/// **right** image.
///
/// The combined image has:
/// * Width  = `left.width() + gap + right.width()`
/// * Height = `max(left.height(), right.height())`
///
/// Both images are top-aligned; any vertical difference is filled with black.
/// The `gap` strip between the two images is filled with mid-grey (128) so
/// the boundary is clearly visible.
///
/// The `label` is rendered in the **top-left corner** of the right image at
/// 3× scale (each font pixel → 3×3 output pixels) with a solid black backing
/// rectangle for legibility on both dark and light backgrounds.
///
/// # Arguments
/// * `left`  – Left panel (e.g. the predicted edge map).
/// * `right` – Right panel (e.g. the ground-truth edge map).
/// * `label` – Short text string to brand the right panel.
/// * `gap`   – Width in pixels of the separator strip between the two panels.
///
/// # Returns
/// A new owned [`GrayImage`].
pub fn place_side_by_side(
    left: &GrayImage,
    right: &GrayImage,
    label: &str,
    gap: u32,
) -> GrayImage {
    let (lw, lh) = left.dimensions();
    let (rw, rh) = right.dimensions();

    let out_w = lw + gap + rw;
    let out_h = lh.max(rh);

    // Start with an all-black canvas.
    let mut out = GrayImage::new(out_w, out_h);

    // --- Gap strip (mid-grey separator) ---
    fill_rect(&mut out, lw, 0, gap, out_h, 128);

    // --- Copy left panel ---
    for y in 0..lh {
        for x in 0..lw {
            out.put_pixel(x, y, *left.get_pixel(x, y));
        }
    }

    // --- Copy right panel ---
    let rx = lw + gap; // x-offset of the right panel in the output
    for y in 0..rh {
        for x in 0..rw {
            out.put_pixel(rx + x, y, *right.get_pixel(x, y));
        }
    }

    // --- Label on the right panel ---
    const SCALE: u32 = 3;
    const PADDING: u32 = 4;

    // Measure the label in pixels.
    let n_chars = label.chars().count() as u32;
    let char_stride = (FONT_COLS + 1) * SCALE;
    // Subtract the trailing gap that char_stride adds after the last character.
    let text_w = n_chars * char_stride - SCALE; // width of glyphs only
    let text_h = FONT_ROWS * SCALE;

    let box_x = rx;                      // flush with the left edge of the right panel
    let box_y = 0;                        // flush with the top of the output
    let box_w = text_w + PADDING * 2;
    let box_h = text_h + PADDING * 2;

    // Black backing box.
    fill_rect(&mut out, box_x, box_y, box_w, box_h, 0);

    // White label text.
    draw_text(
        &mut out,
        box_x + PADDING,
        box_y + PADDING,
        label,
        SCALE,
        255,
    );

    out
}

/// Convenience wrapper: places `predicted` on the left and `gt` on the right
/// with a "GT" label, separated by an 8-pixel gap.
///
/// Typical use:
/// ```no_run
/// use image::open;
/// let predicted = open("edges.png").unwrap().into_luma8();
/// let gt        = open("UDED/gt/12-cameraman.png").unwrap().into_luma8();
/// let panel     = compare_with_gt(&predicted, &gt);
/// panel.save("comparison.png").unwrap();
/// ```
pub fn compare_with_gt(predicted: &GrayImage, gt: &GrayImage) -> GrayImage {
    place_side_by_side(predicted, gt, "GT", 8)
}