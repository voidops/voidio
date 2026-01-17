use std::{fmt, ops};
use crate::console::{Component, ConsoleMessage, ConsoleSendable};

fn rgba_to_ansi(rgba: u32) -> String {
    let r = (rgba >> 16) & 0xFF;
    let g = (rgba >> 8) & 0xFF;
    let b = rgba & 0xFF;
    // 24-bit true color ANSI format
    format!("38;2;{};{};{}", r, g, b)
}

#[derive(Clone)]
pub struct TextComponent {
    pub(crate) color: Option<u32>,
    pub(crate) text: String,
}

impl TextComponent {
    pub fn with_color(self, color: u32) -> Self {
        TextComponent {
            color: Some(color),
            text: self.text,
        }
    }
}
impl fmt::Display for TextComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(color) = &self.color {
            let ansi_color = rgba_to_ansi(*color);
            write!(f, "\x1b[{}m", ansi_color)?;
            write!(f, "{}", self.text)?;
            write!(f, "\x1b[0m")?;
        } else {
            write!(f, "{}", self.text)?;
        }
        Ok(())
    }
}

impl ConsoleSendable for TextComponent {
    fn clone_box(&self) -> Box<dyn ConsoleSendable> {
        Box::new(self.clone())
    }
}

impl ConsoleSendable for String {
    fn clone_box(&self) -> Box<dyn ConsoleSendable> {
        Box::new(self.clone())
    }
}
impl Component {
    pub fn text(text: &str) -> TextComponent {
        TextComponent {
            color: None,
            text: text.to_string(),
        }
    }
}
impl ops::Add<TextComponent> for &str {
    type Output = ConsoleMessage;
    fn add(self, other: TextComponent) -> Self::Output {
        let mut message = ConsoleMessage::new();
        message.add_ref(&self.to_string());
        message.add_ref(&other);
        message
    }
}

impl ops::Add<&str> for ConsoleMessage {
    type Output = ConsoleMessage;
    fn add(mut self, other: &str) -> Self::Output {
        self.add_ref(&Component::text(other));
        self
    }
}
impl ops::Add<String> for ConsoleMessage {
    type Output = ConsoleMessage;
    fn add(mut self, other: String) -> Self::Output {
        self.add_ref(&Component::text(&other));
        self
    }
}

impl ops::Add<TextComponent> for ConsoleMessage {
    type Output = ConsoleMessage;
    fn add(mut self, other: TextComponent) -> Self::Output {
        self.add_ref(&other);
        self
    }
}