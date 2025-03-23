pub trait AnyhowErrorToStringChain {
    /// 将 `anyhow::Error` 转换为chain格式  
    /// # Example
    /// 0: error message  
    /// 1: error message  
    /// 2: error message  
    fn to_string_chain(&self) -> String;
}

impl AnyhowErrorToStringChain for anyhow::Error {
    fn to_string_chain(&self) -> String {
        use std::fmt::Write;
        self.chain()
            .enumerate()
            .fold(String::new(), |mut output, (i, e)| {
                let _ = writeln!(output, "{i}: {e}");
                output
            })
    }
}

pub trait PathIsImage {
    /// 判断路径是否为图片文件  
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use tauri_manga::extensions::PathIsImage;
    ///
    /// let path = Path::new("test.jpg");
    /// assert_eq!(path.is_image(), true);
    /// ```
    fn is_image(&self) -> bool;
}

impl PathIsImage for std::path::Path {
    fn is_image(&self) -> bool {
        self.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase)
            .is_some_and(|ext| matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif"))
    }
}
