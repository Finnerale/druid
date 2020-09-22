use crate::hotkey::HotKey;

pub struct Menu;

impl Menu {
    pub fn new() -> Menu {
        Menu {}
    }

    pub fn new_for_popup() -> Menu {
        Menu {}
    }

    pub fn add_dropdown(&mut self, mut _menu: Menu, _text: &str, _enabled: bool) {}

    pub fn add_item(
        &mut self,
        _id: u32,
        _text: &str,
        _key: Option<&HotKey>,
        _enabled: bool,
        _selected: bool,
    ) {
    }

    pub fn add_separator(&mut self) {}
}
