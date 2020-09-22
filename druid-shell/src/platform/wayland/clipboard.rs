//! Interactions with the system pasteboard.

use crate::clipboard::{ClipboardFormat, FormatId};

#[derive(Debug, Clone, Default)]
pub struct Clipboard;

impl Clipboard {
    pub fn put_string(&mut self, _s: impl AsRef<str>) {}

    pub fn put_formats(&mut self, _formats: &[ClipboardFormat]) {}

    pub fn get_string(&self) -> Option<String> {
        None
    }

    pub fn preferred_format(&self, _formats: &[FormatId]) -> Option<FormatId> {
        None
    }

    pub fn get_format(&self, _format: FormatId) -> Option<Vec<u8>> {
        None
    }

    pub fn available_type_names(&self) -> Vec<String> {
        vec![]
    }
}
