use std::error::Error;

pub trait Renderer {
    fn render(&self, book: &::book::MDBook) ->  Result<(), Box<Error>>;
    fn copy_theme(&self, book: &::book::MDBook) -> Result<(), Box<Error>>;
}
