///!本库可能发生的独立错误。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("the path `{0}` is not a folder")]
    NonFolder(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}
