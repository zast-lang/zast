use crate::{error_handler::zast_errors::ZastError, lexer::tokens::Span};

pub mod error_span;
pub mod errors_messages;
pub mod zast_errors;

#[derive(Default, Debug)]
pub struct ZastErrorCollector {
    errors: Vec<ZastError>,
}

impl ZastErrorCollector {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn report_all_errors(&self) {
        for i in 0..self.errors.len() {
            self.report_error(i);
        }
    }

    pub fn report_error(&self, error_idx: usize) {
        let error = &self.errors[error_idx];
        eprintln!(
            "Error at: {} | {}",
            Span::format_span(error.get_span()),
            error.get_error_msg()
        );
    }

    pub fn add_error(&mut self, zast_error: ZastError) {
        self.errors.push(zast_error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}
