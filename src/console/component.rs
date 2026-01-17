use std::{fmt};
use std::any::Any;
use crate::console::ConsoleMessage;

pub trait ConsoleComponent: fmt::Display {
    fn is_message_marker(&self) -> bool {
        false
    }
    fn clone_box(&self) -> Box<dyn ConsoleComponent>;
}
impl Clone for Box<dyn ConsoleComponent> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub struct Component {}
impl Component {
    pub fn empty() -> ConsoleMessage {
        ConsoleMessage::new()
    }
}