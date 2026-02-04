#[derive(Debug, Clone)]
pub enum Message {
    None,
    Info(String),
    Warn(String),
    Error(String),
}
