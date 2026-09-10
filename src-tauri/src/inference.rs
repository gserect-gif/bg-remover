use crate::error::{AppError, AppResult};
use image::{DynamicImage, RgbImage};
use ndarray::Array4;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::TensorRef;
use std::path::Path;

/// IS-Net general-use expects a fixed 1024x1024 input, normalized to
/// (pixel/255 - mean) / std with mean=std=0.5, i.e. roughly [-1, 1].
const MODEL_INPUT_SIZE: u32 = 1024;
const NORM_MEAN: f32 = 0.5;
const NORM_STD: f32 = 1.0;

pub struct InferenceEngine {
    session: Session,
}

impl InferenceEngine {
    /// Loads the ONNX model from disk. This is the expensive step
    /// (allocates the session, reads ~176MB of weights) and should only
    /// ever run once per app lifetime — callers cache the resulting engine.
    pub fn load(model_path: &Path) -> AppResult<Self> {
        if !model_path.exists() {
            return Err(AppError::ModelUnavailable(format!(
                "model file not found at {}",
                model_path.display()
            )));
        }

        // Threaded CPU execution provider. intra_threads scales with
        // available cores; ort/onnxruntime picks a sane default if we
        // don't override, but we cap it to avoid hammering low-core CPUs.
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let intra_threads = cores.clamp(2, 8);

        let session = Session::builder()
            .map_err(|e| AppError::ModelUnavailable(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| AppError::ModelUnavailable(e.to_string()))?
            .with_intra_threads(intra_threads)
            .map_err(|e| AppError::ModelUnavailable(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| AppError::ModelUnavailable(e.to_string()))?;

        Ok(Self { session })
    }

    /// Runs segmentation on an RGB image and returns a single-channel
    /// saliency mask (0.0 = background, 1.0 = foreground) at the model's
    /// native output resolution (1024x1024). Caller is responsible for
    /// resizing the mask back to the original image dimensions.
    pub fn predict_mask(&mut self, rgb: &RgbImage) -> AppResult<Vec<f32>> {
        let resized = DynamicImage::ImageRgb8(rgb.clone()).resize_exact(
            MODEL_INPUT_SIZE,
            MODEL_INPUT_SIZE,
            image::imageops::FilterType::Triangle,
        );
        let resized_rgb = resized.to_rgb8();

        // Build NCHW float32 tensor.
        let mut input = Array4::<f32>::zeros((1, 3, MODEL_INPUT_SIZE as usize, MODEL_INPUT_SIZE as usize));
        for (x, y, px) in resized_rgb.enumerate_pixels() {
            let (x, y) = (x as usize, y as usize);
            input[[0, 0, y, x]] = (px[0] as f32 / 255.0 - NORM_MEAN) / NORM_STD;
            input[[0, 1, y, x]] = (px[1] as f32 / 255.0 - NORM_MEAN) / NORM_STD;
            input[[0, 2, y, x]] = (px[2] as f32 / 255.0 - NORM_MEAN) / NORM_STD;
        }

        let input_ref = TensorRef::from_array_view(&input)
            .map_err(|e| AppError::InferenceFailure(e.to_string()))?;

        let input_name = self
            .session
            .inputs
            .first()
            .map(|i| i.name.clone())
            .ok_or_else(|| AppError::InferenceFailure("model has no inputs".into()))?;

        let outputs = self
            .session
            .run(ort::inputs![input_name.as_str() => input_ref])
            .map_err(|e| AppError::InferenceFailure(e.to_string()))?;

        let output_name = self
            .session
            .outputs
            .first()
            .map(|o| o.name.clone())
            .ok_or_else(|| AppError::InferenceFailure("model has no outputs".into()))?;

        let (shape, data) = outputs[output_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::InferenceFailure(e.to_string()))?;

        // IS-Net outputs raw saliency logits/values in [0, 1] already
        // (sigmoid applied in-graph for this export); shape is [1,1,H,W].
        let h = shape[shape.len() - 2] as usize;
        let w = shape[shape.len() - 1] as usize;
        if h != MODEL_INPUT_SIZE as usize || w != MODEL_INPUT_SIZE as usize {
            return Err(AppError::InferenceFailure(format!(
                "unexpected output shape {:?}",
                shape
            )));
        }

        // Min-max normalize defensively in case the export doesn't apply
        // sigmoid internally — keeps the mask well-conditioned either way.
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for &v in data.iter() {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        let range = (max - min).max(1e-6);
        let normalized: Vec<f32> = data.iter().map(|&v| (v - min) / range).collect();

        Ok(normalized)
    }
}
