//! 包装本库 Error 的简化 Result 别名。

/// 包装 [`Error`](../enum.Error.html) 的 Result。
pub type Result<T> = std::result::Result<T, crate::Error>;
