use crate::error::{AppError, AppResult};
use crate::inference::InferenceEngine;
use crate::AppState;
use image::{imageops::FilterType, ImageBuffer, Rgba, RgbaImage};
use serde::Serialize;
use std::path::Path;
use tauri::State;

/// Images larger than this (on the long edge) get downsampled before
/// inference purely for practical CPU performance; the final alpha mask
/// is still upsampled back to the image's true original resolution before
/// export, so output dimensions are never reduced.
const MAX_INFERENCE_LONG_EDGE: u32 = 2048;

/// Hard ceiling to avoid pathological memory use on extreme images
/// (e.g. a 40000x40000 decompression-bomb-style file).
const MAX_DECODE_PIXELS: u64 = 60_000_000; // ~60 megapixels

#[derive(Serialize)]
pub struct RemovalResult {
    /// Path to a temp PNG containing the RGBA result at original resolution,
    /// used by the frontend for the checkerboard preview.
    pub preview_path: String,
    pub width: u32,
    pub height: u32,
}

pub fn ensure_engine_loaded(state: &State<AppState>) -> AppResult<()> {
    let mut engine_guard = state.engine.lock().unwrap();
    if engine_guard.is_some() {
        return Ok(());
    }
    let model_path = state
        .model_path
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| AppError::ModelUnavailable("model path not resolved".into()))?;

    let engine = InferenceEngine::load(&model_path)?;
    *engine_guard = Some(engine);
    Ok(())
}

pub fn process_image(state: &State<AppState>, input_path: &str) -> AppResult<RemovalResult> {
    let path = Path::new(input_path);

    let img = image::open(path).map_err(|_| AppError::UnsupportedOrCorruptImage)?;

    let (orig_w, orig_h) = (img.width(), img.height());
    if (orig_w as u64) * (orig_h as u64) > MAX_DECODE_PIXELS {
        return Err(AppError::ImageTooLarge(format!(
            "{}x{} exceeds the supported size",
            orig_w, orig_h
        )));
    }

    let rgba_original = img.to_rgba8();

    // Downscale for inference only, if needed, for practical CPU speed.
    let long_edge = orig_w.max(orig_h);
    let inference_img = if long_edge > MAX_INFERENCE_LONG_EDGE {
        let scale = MAX_INFERENCE_LONG_EDGE as f32 / long_edge as f32;
        let w = (orig_w as f32 * scale).round().max(1.0) as u32;
        let h = (orig_h as f32 * scale).round().max(1.0) as u32;
        image::imageops::resize(&rgba_original, w, h, FilterType::Lanczos3)
    } else {
        rgba_original.clone()
    };
    let rgb_for_inference = image::DynamicImage::ImageRgba8(inference_img.clone()).to_rgb8();

    ensure_engine_loaded(state)?;

    let mut engine_guard = state.engine.lock().unwrap();
    let engine = engine_guard
        .as_mut()
        .ok_or_else(|| AppError::ModelUnavailable("engine failed to initialize".into()))?;

    let raw_mask_1024 = engine.predict_mask(&rgb_for_inference)?;
    drop(engine_guard);

    // Upsample mask from the model's native 1024x1024 back to the
    // inference image size, then to the true original resolution.
    let mask_img: ImageBuffer<image::Luma<f32>, Vec<f32>> =
        ImageBuffer::from_raw(1024, 1024, raw_mask_1024)
            .ok_or_else(|| AppError::InferenceFailure("mask buffer size mismatch".into()))?;

    let mask_at_original = resize_mask_to(&mask_img, orig_w, orig_h);
    let refined_mask = refine_mask(&mask_at_original, orig_w, orig_h);

    let output = composite_rgba(&rgba_original, &refined_mask, orig_w, orig_h);

    let temp_path = std::env::temp_dir().join(format!(
        "bgremover_preview_{}.png",
        uuid_like_suffix()
    ));
    output
        .save(&temp_path)
        .map_err(|e| AppError::ExportFailure(e.to_string()))?;

    Ok(RemovalResult {
        preview_path: temp_path.to_string_lossy().to_string(),
        width: orig_w,
        height: orig_h,
    })
}

/// Resizes a float luma mask to target dimensions using bilinear filtering,
/// preserving smooth alpha gradients (important for hair/fur edges) rather
/// than nearest-neighbor blockiness.
fn resize_mask_to(
    mask: &ImageBuffer<image::Luma<f32>, Vec<f32>>,
    target_w: u32,
    target_h: u32,
) -> Vec<f32> {
    let (src_w, src_h) = mask.dimensions();
    let mut out = vec![0f32; (target_w * target_h) as usize];

    for y in 0..target_h {
        let sy = (y as f32 + 0.5) / target_h as f32 * src_h as f32 - 0.5;
        let y0 = sy.floor().clamp(0.0, src_h as f32 - 1.0) as u32;
        let y1 = (y0 + 1).min(src_h - 1);
        let fy = (sy - y0 as f32).clamp(0.0, 1.0);

        for x in 0..target_w {
            let sx = (x as f32 + 0.5) / target_w as f32 * src_w as f32 - 0.5;
            let x0 = sx.floor().clamp(0.0, src_w as f32 - 1.0) as u32;
            let x1 = (x0 + 1).min(src_w - 1);
            let fx = (sx - x0 as f32).clamp(0.0, 1.0);

            let p00 = mask.get_pixel(x0, y0)[0];
            let p10 = mask.get_pixel(x1, y0)[0];
            let p01 = mask.get_pixel(x0, y1)[0];
            let p11 = mask.get_pixel(x1, y1)[0];

            let top = p00 * (1.0 - fx) + p10 * fx;
            let bottom = p01 * (1.0 - fx) + p11 * fx;
            out[(y * target_w + x) as usize] = top * (1.0 - fy) + bottom * fy;
        }
    }

    out
}

/// Cleans up the raw saliency mask:
/// 1. Removes small disconnected background-fragment islands ("crumbs")
///    via a lightweight morphological approach on a thresholded copy.
/// 2. Applies a narrow-band Gaussian-style smoothing pass only near edges
///    (not globally) to soften jaggedness/anti-alias hard transitions
///    without destroying fine foreground detail elsewhere.
fn refine_mask(mask: &[f32], w: u32, h: u32) -> Vec<f32> {
    let w = w as usize;
    let h = h as usize;

    // Step 1: binary threshold copy for connected-component filtering.
    let threshold = 0.5f32;
    let mut binary: Vec<bool> = mask.iter().map(|&v| v > threshold).collect();

    remove_small_islands(&mut binary, w, h, true); // remove small foreground specks
    remove_small_islands(&mut binary, w, h, false); // remove small background holes inside subject

    // Step 2: rebuild a soft mask that respects the cleaned binary mask
    // but keeps original soft/gradient values near boundaries (so hair,
    // fur, and semi-transparent edges are preserved rather than hard-cut).
    let mut soft = vec![0f32; w * h];
    for i in 0..w * h {
        soft[i] = if binary[i] {
            mask[i].max(0.5)
        } else {
            mask[i].min(0.5)
        };
    }

    // Step 3: edge-aware smoothing — small box blur applied to the mask,
    // blended in proportion to local gradient magnitude, so flat regions
    // stay untouched (full detail preserved) while jagged boundaries get
    // softened into clean anti-aliased edges.
    edge_aware_smooth(&soft, w, h)
}

fn remove_small_islands(binary: &mut [bool], w: usize, h: usize, foreground: bool) {
    let min_island_size = ((w * h) as f64 * 0.00015).max(24.0) as usize;
    let mut visited = vec![false; w * h];
    let mut stack = Vec::new();

    for start in 0..w * h {
        if visited[start] || binary[start] != foreground {
            continue;
        }
        let mut island = Vec::new();
        stack.push(start);
        visited[start] = true;

        while let Some(idx) = stack.pop() {
            island.push(idx);
            let x = idx % w;
            let y = idx / w;

            let neighbors = [
                (x.checked_sub(1), Some(y)),
                (Some(x + 1).filter(|&v| v < w), Some(y)),
                (Some(x), y.checked_sub(1)),
                (Some(x), Some(y + 1).filter(|&v| v < h)),
            ];

            for (nx, ny) in neighbors {
                if let (Some(nx), Some(ny)) = (nx, ny) {
                    let nidx = ny * w + nx;
                    if !visited[nidx] && binary[nidx] == foreground {
                        visited[nidx] = true;
                        stack.push(nidx);
                    }
                }
            }
        }

        if island.len() < min_island_size {
            for idx in island {
                binary[idx] = !foreground;
            }
        }
    }
}

fn edge_aware_smooth(mask: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = mask.to_vec();
    let radius: i32 = 1;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let center = mask[idx];

            // Estimate local gradient to detect edges.
            let left = if x > 0 { mask[idx - 1] } else { center };
            let right = if x + 1 < w { mask[idx + 1] } else { center };
            let up = if y > 0 { mask[idx - w] } else { center };
            let down = if y + 1 < h { mask[idx + w] } else { center };
            let gradient = ((right - left).abs() + (down - up).abs()) * 0.5;

            if gradient < 0.03 {
                // Flat region (solidly foreground or solidly background):
                // leave untouched to preserve detail exactly.
                continue;
            }

            // Near an edge: average a small neighborhood for a clean,
            // anti-aliased transition instead of a jagged one.
            let mut sum = 0f32;
            let mut count = 0f32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                        sum += mask[ny as usize * w + nx as usize];
                        count += 1.0;
                    }
                }
            }
            out[idx] = sum / count;
        }
    }

    out
}

fn composite_rgba(original: &RgbaImage, alpha_mask: &[f32], w: u32, h: u32) -> RgbaImage {
    let mut out = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let src = original.get_pixel(x, y);
            let a = (alpha_mask[(y * w + x) as usize].clamp(0.0, 1.0) * 255.0).round() as u8;
            out.put_pixel(x, y, Rgba([src[0], src[1], src[2], a]));
        }
    }
    out
}

pub fn export_png(source_rgba_path: &str, dest_path: &str) -> AppResult<()> {
    let img = image::open(source_rgba_path).map_err(|_| AppError::ExportFailure("preview missing".into()))?;
    let rgba = img.to_rgba8();
    rgba.save_with_format(dest_path, image::ImageFormat::Png)
        .map_err(|e| AppError::ExportFailure(e.to_string()))?;
    Ok(())
}

fn uuid_like_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
