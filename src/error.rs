use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("the input value `{0}` is not an invalid integer")]
    InvalidIntInput(String),

    #[error("the path `{0}` is not a folder")]
    NonFolder(String),

    #[error("the image file `{0}` is empty")]
    EmptyImage(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    // 缺失扩展名
    #[error("missing extension name")]
    MissingExtension,

    #[cfg(feature = "image")]
    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[cfg(feature = "magickwand")]
    #[error(transparent)]
    MagickError(#[from] magick_rust::MagickError),

    // 无效的 Unicode（常见于路径转换为字符串）
    #[error("invalid unicode")]
    InvalidUnicode,
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Message(message.to_owned())
    }
}
