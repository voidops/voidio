use std::fmt;
use crate::console::ConsoleComponent;

#[derive(Clone)]
pub struct ConsoleMessage {
    pub(crate) parts: Vec<Box<dyn ConsoleComponent>>,
}

impl ConsoleMessage {
    pub fn new() -> Self {
        ConsoleMessage {
            parts: Vec::new(),
        }
    }
    pub fn push(&mut self, component: Box<dyn ConsoleComponent>) {
        self.parts.push(component);
    }
    pub fn add<C: ConsoleComponent + 'static>(&mut self, component: C) {
        self.parts.push(Box::new(component));
    }

    pub fn add_ref(&mut self, component: &dyn ConsoleComponent) {
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

impl ConsoleComponent for ConsoleMessage {
    fn clone_box(&self) -> Box<dyn ConsoleComponent> {
        let mut message = ConsoleMessage::new();
        for part in &self.parts {
            message.add_ref(part);
        }
        Box::new(message)
    }
}

impl ConsoleComponent for Box<dyn ConsoleComponent> {

    fn is_message_marker(&self) -> bool {
        (**self).is_message_marker()
    }

    fn clone_box(&self) -> Box<dyn ConsoleComponent> {
        (**self).clone_box()
    }
}

impl std::ops::Add<&dyn ConsoleComponent> for ConsoleMessage {
    type Output = ConsoleMessage;

    fn add(mut self, other: &dyn ConsoleComponent) -> Self::Output {
        self.add_ref(other);
        self
    }
}