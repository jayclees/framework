use super::Responsable;
use crate::http::error::HttpError;

pub fn text(string: String) -> Result<Box<dyn Responsable>, HttpError> {
    Ok(Box::new(string))
}
