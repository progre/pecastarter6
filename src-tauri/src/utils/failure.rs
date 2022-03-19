#[derive(Debug)]
pub enum Failure {
    #[allow(dead_code)]
    Warn(String),
    Error(String),
    Fatal(String),
}
