use std::fmt::{Display, Formatter};
use std::{fmt, io};
use std::cell::RefCell;
use std::sync::Arc;
use crate::console::component::ConsoleSendable;
use crate::console::ConsoleMessage;
use crate::console::receiver::Console;
use std::io::{Write};

#[derive(Clone)]
pub struct MessageMarker {}
impl fmt::Display for MessageMarker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[MESSAGE MARKER]")
    }
}

impl ConsoleSendable for MessageMarker {
    fn is_message_marker(&self) -> bool {
        true
    }
    fn clone_box(&self) -> Box<dyn ConsoleSendable> {
        Box::new(self.clone())
    }
}
#[derive(Clone)]
pub struct Stdout {
    inner: Arc<io::Stdout>,
    begin_parts: Vec<Box<dyn ConsoleSendable>>,
    end_parts: Vec<Box<dyn ConsoleSendable>>,
    message_part: RefCell<Option<ConsoleMessage>>,
}

/* Create an Stdout handle */
pub fn stdout() -> Stdout {
    Stdout {
        inner: Arc::new(io::stdout()), // Arc because we want the same stdout across threads
        begin_parts: Vec::new(),
        end_parts: Vec::new(),
        message_part: RefCell::new(None),
    }
}

/* Create an Stdout handle with a custom formatter */
pub fn fstdout<F>(formatter: F) -> Stdout
where
    F: Fn(&dyn ConsoleSendable) -> ConsoleMessage + Send + Sync + 'static
{
    let marker = MessageMarker {};
    let format = formatter(&marker);
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
    Stdout {
        inner: Arc::new(io::stdout()),
        begin_parts,
        end_parts,
        message_part: RefCell::new(None)
    }
}

impl Console for Stdout {
    fn try_write<M: ConsoleSendable + 'static>(&self, component: M) -> io::Result<()> {
        let mut handle = self.inner.lock();
        // Borrow once and mutate in place to avoid nested borrowing panics.
        let mut slot = self.message_part.borrow_mut();
        if let Some(msg) = slot.as_mut() {
            msg.add(component.clone_box());
        } else {
            for part in &self.begin_parts {
                write!(handle, "{}", part)?;
            }
            let mut msg = ConsoleMessage::new();
            msg.add(component.clone_box());
            *slot = Some(msg);
        }
        write!(handle, "{}", component)
    }

    fn commit(&self) -> io::Result<()> {
        let mut handle = self.inner.lock();
        for part in &self.end_parts {
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