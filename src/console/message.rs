use std::fmt;
use crate::console::ConsoleSendable;

#[derive(Clone)]
pub struct ConsoleMessage {
    pub(crate) parts: Vec<Box<dyn ConsoleSendable>>,
}

impl ConsoleMessage {
    pub fn new() -> Self {
        ConsoleMessage {
            parts: Vec::new(),
        }
    }

    pub fn add<C: ConsoleSendable + 'static>(&mut self, component: C) {
        self.parts.push(Box::new(component));
    }

    pub fn add_ref(&mut self, component: &dyn ConsoleSendable) {
        self.parts.push(component.clone_box());
    }
}

impl fmt::Display for ConsoleMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in &self.parts {
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

impl ConsoleSendable for ConsoleMessage {
    fn clone_box(&self) -> Box<dyn ConsoleSendable> {
        let mut message = ConsoleMessage::new();
        for part in &self.parts {
            message.add_ref(part);
        }
        Box::new(message)
    }
}

impl ConsoleSendable for Box<dyn ConsoleSendable> {

    fn is_message_marker(&self) -> bool {
        (**self).is_message_marker()
    }

    fn clone_box(&self) -> Box<dyn ConsoleSendable> {
        (**self).clone_box()
    }
}

impl std::ops::Add<&dyn ConsoleSendable> for ConsoleMessage {
    type Output = ConsoleMessage;

    fn add(mut self, other: &dyn ConsoleSendable) -> Self::Output {
        self.add_ref(other);
        self
    }
}