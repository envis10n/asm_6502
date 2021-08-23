use std::fmt::Display;

#[derive(Debug)]
pub struct CompileError {
    pub line: usize,
    pub message: &'static str,
}

impl CompileError {
    pub fn new(line: usize, message: &'static str) -> Self {
        CompileError { line, message }
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "compile error on line {}: {}", self.line, self.message)
    }
}
