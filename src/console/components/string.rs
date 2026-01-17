use crate::console::{ConsoleSendable};

impl ConsoleSendable for &str {
    fn clone_box(&self) -> Box<dyn ConsoleSendable> {
        Box::new(self.to_string())
    }
}