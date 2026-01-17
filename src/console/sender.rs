use std::fmt::{Display, Formatter};
use std::{fmt, io};
use std::cell::RefCell;
use std::sync::Arc;
use crate::console::component::ConsoleComponent;
use crate::console::{Component, ConsoleMessage};
use crate::console::receiver::Console;
use std::io::{Write};
use std::ops::Add;
use std::time::SystemTime;

#[derive(Clone)]
pub struct MessageEntry {
    pub timestamp_ms: u128,
    pub tag: Option<String>,
}
impl Add<&MessageEntry> for ConsoleMessage {
    type Output = ConsoleMessage;
    fn add(self, entry: &MessageEntry) -> Self::Output {
        self + entry.clone_box()
    }
}
impl Add<&str> for &MessageEntry {
    type Output = ConsoleMessage;
    fn add(self, other: &str) -> Self::Output {
        let mut message = ConsoleMessage::new();
        message.add_ref(self);
        message.add_ref(&crate::console::Component::text(other));
        message
    }
}

impl Add<Box<dyn ConsoleComponent>> for ConsoleMessage {
    type Output = ConsoleMessage;
    fn add(mut self, component: Box<dyn ConsoleComponent>) -> Self::Output {
        self.parts.push(component);
        self
    }
}

impl fmt::Display for MessageEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[Message Entry]")
    }
}

impl ConsoleComponent for MessageEntry {
    fn is_message_marker(&self) -> bool {
        true
    }
    fn clone_box(&self) -> Box<dyn ConsoleComponent> {
        Box::new(self.clone())
    }
}
#[derive(Clone)]
pub struct Stdout {
    inner: Arc<io::Stdout>,
    end_parts: RefCell<Vec<Box<dyn ConsoleComponent>>>,
    message_part: RefCell<Option<ConsoleMessage>>,
    formatter_fn: Option<Arc<dyn Fn(&MessageEntry) -> ConsoleMessage + Send + Sync>>,
}

impl Stdout {
    pub fn init_format(&self, tag: Option<String>) -> (Vec<Box<dyn ConsoleComponent>>, bool) {
        if let Some(formatter) = &self.formatter_fn {
            let entry = MessageEntry {
                timestamp_ms:
                    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(),
                tag,
            };
            let format = formatter(&entry);
            let mut begin_parts = Vec::new();
            let mut end_parts = Vec::new();
            let mut found_marker = false;
            for part in &format.parts {
                if (found_marker) {
                    end_parts.push(part.clone_box());
                    continue;
                }
                if part.is_message_marker() {
                    found_marker = true;
                    continue;
                }
                begin_parts.push(part.clone_box());
            }
            *self.end_parts.borrow_mut() = end_parts;
            return (begin_parts, found_marker);
        }
        (Vec::new(), false)
    }
}

/* Create an Stdout handle */
pub fn stdout() -> Stdout {
    Stdout {
        inner: Arc::new(io::stdout()), // Arc because we want the same stdout across threads
        end_parts: RefCell::new(Vec::new()),
        message_part: RefCell::new(None),
        formatter_fn: None
    }
}

/* Create an Stdout handle with a custom formatter */
pub fn fstdout<F>(formatter: F) -> Stdout
where
    F: Fn(&MessageEntry) -> ConsoleMessage + Send + Sync + 'static,
{
    Stdout {
        inner: Arc::new(io::stdout()),
        end_parts: RefCell::new(Vec::new()),
        message_part: RefCell::new(None),
        formatter_fn: Some(Arc::new(formatter))
    }
}

impl Console for Stdout {
    fn try_write<M: ConsoleComponent + 'static>(&self, component: M) -> io::Result<()> {
        let mut handle = self.inner.lock();
        // Borrow once and mutate in place to avoid nested borrowing panics.
        let mut slot = self.message_part.borrow_mut();
        if let Some(msg) = slot.as_mut() {
            msg.add(component.clone_box());
            write!(handle, "{}", component)
        } else {
            let (begin_parts, exists_message) = self.init_format(None);
            for part in &begin_parts {
                write!(handle, "{}", part)?;
            }
            let mut msg = ConsoleMessage::new();
            msg.push(component.clone_box());
            *slot = Some(msg);
            if exists_message {
                write!(handle, "{}", component)
            } else {
                Ok(())
             }
        }
    }

    fn commit(&self) -> io::Result<()> {
        let mut handle = self.inner.lock();
        for part in self.end_parts.borrow().iter() {
            if (!part.is_message_marker()) {
                write!(handle, "{}", part)?;
            } else {
                write!(handle, "{}", self.message_part.borrow().as_ref().unwrap())?;
            }
        }
        writeln!(handle)?;
        *self.message_part.borrow_mut() = None;
        Ok(())
    }
}