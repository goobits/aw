pub type Result<T> = std::result::Result<T, AwError>;

#[derive(Debug)]
pub struct AwError {
    pub message: String,
    pub code: i32,
    pub show_usage: bool,
}

impl AwError {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self {
            message: message.into(),
            code,
            show_usage: false,
        }
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 2,
            show_usage: true,
        }
    }
}

impl From<std::io::Error> for AwError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string(), 1)
    }
}
