use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Couldn't read that file. It may be corrupted or in an unsupported format.")]
    UnsupportedOrCorruptImage,

    #[error("The image is too large to process on this system ({0}).")]
    ImageTooLarge(String),

    #[error("The AI model could not be loaded. Try reinstalling the application.")]
    ModelUnavailable(String),

    #[error("Background removal failed while running the AI model: {0}")]
    InferenceFailure(String),

    #[error("Couldn't save the exported image: {0}")]
    ExportFailure(String),

    #[error("Not enough memory available to process this image.")]
    OutOfMemory,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;
