// Apply hysteresis thresholding to a gradient image.
// Pixels are first categorised to be 'over', 'middle' or 'under' the thresholds. And then each 'middle' pixels are
// then categorised to be 'over' threshold if it neighbours an 'over' threshold.

//Define enum types to categorise pixels for DFS pass.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum EdgeState {
    Over = 0,
    Middle = 1,
    Under = 2,
}
pub fn hysteresis_threshold(
    image: &[f32],
    width: usize,
    height: usize,
    low_thresh: f32,
    high_thresh: f32,
) -> Vec<u8> {
    assert!(
        high_thresh >= low_thresh,
        "high_thresh ({high_thresh}) must be >= low_thresh ({low_thresh})"
    );

    // classify every pixel
    let mut labels: Vec<EdgeState> = image
        .iter()
        .map(|&v| {
            if v >= high_thresh {
                EdgeState::Over
            } else if v >= low_thresh {
                EdgeState::Middle
            } else {
                EdgeState::Under
            }
        })
        .collect();

    // flood-fill from 'over' pixels, promoting 'middle' neighbours
    // Create a stack and push all 'Over' pixels. Then loop through its neighbours, promote them and add them to stack as well.
    let mut stack: Vec<usize> = labels
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| if v == EdgeState::Over { Some(i) } else { None })
        .collect();

    while let Some(idx) = stack.pop() {
        let y = idx / width;
        let x = idx % width;

        // Examine all 8-connected neighbours.
        for dy in -1_isize..2 {
            for dx in -1_isize..2 {
                if dy == 0 && dx == 0 {
                    continue; // skip the centre pixel
                }
                let ny = y as isize + dy;
                let nx = x as isize + dx;
                if ny >= 0 && ny < height as isize && nx >= 0 && nx < width as isize {
                    let nidx = ny as usize * width + nx as usize;
                    if labels[nidx] == EdgeState::Middle {
                        labels[nidx] = EdgeState::Over;
                        stack.push(nidx);
                    }
                }
            }
        }
    }

    // All 'Over' pixels are now set to 255 u8 and other pixels are set to 0.
    let output : Vec<u8> = labels
        .iter()
        .cloned()
        .map(|v| if v == EdgeState::Over { 255_u8 } else { 0_u8 })
        .collect();

    output
}

// Choose an automatic threshold from the magnitude image by choosing the 85%th percentile of all the non-zero pixels.
// Returns `10.0` as a safe fallback if no non-zero pixels exist.
pub fn auto_threshold(image: &[f32]) -> f32 {
    let mut vals: Vec<f32> = image
        .iter()
        .copied()
        .filter(|&v| v > 0.0)
        .collect();

    if vals.is_empty() {
        return 10.0;
    }

    vals.sort_by(f32::total_cmp);
    let idx = ((vals.len() as f32) * 0.85) as usize;
    vals[idx]
}