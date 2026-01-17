use std::ops::Add;
use crate::console::{ConsoleMessage};

impl Add<u128> for ConsoleMessage {
    type Output = ConsoleMessage;
    fn add(mut self, value: u128) -> Self::Output {
        self.parts.push(Box::new(crate::console::Component::text(&value.to_string())));
        self
    }
}
