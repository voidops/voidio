use crate::console::{ConsoleComponent};

impl ConsoleComponent for &str {
    fn clone_box(&self) -> Box<dyn ConsoleComponent> {
        Box::new(self.to_string())
    }
}