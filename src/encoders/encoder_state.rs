use std::{rc::Rc, cell::RefCell};



#[derive(Clone, PartialEq)]
pub struct EncoderState {
    pub(super) destroy: Rc<RefCell<bool>>,
    pub(super) enabled: Rc<RefCell<bool>>,
    pub(super) selected: Option<String>,
    pub(super) switching: Rc<RefCell<bool>>,
}

impl EncoderState {
    pub fn new() -> Self {
        Self {
            destroy: Rc::new(RefCell::new(false)),
            enabled: Rc::new(RefCell::new(true)),
            selected: None,
            switching: Rc::new(RefCell::new(false)),
        }
    }

    // Sets the enabled bit to a given value, returning true if it was a change.
    pub fn set_enabled(&mut self, value: bool) -> bool {
        if value != self.is_enabled() {
            *self.enabled.as_ref().borrow_mut() = value;
            true
        } else {
            false
        }
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.borrow()
    }

    pub fn select(&mut self, device: String) -> bool {
        self.selected = Some(device);
        if self.is_enabled() {
            *self.switching.as_ref().borrow_mut() = true;
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) {
        *self.destroy.as_ref().borrow_mut() = true;
    }
}
