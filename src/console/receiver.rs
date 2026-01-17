use std::any::Any;
use std::io;
use std::io::{stdout, Write};
use crate::console::{ConsoleComponent};

pub trait Console {
    fn buffer(&self, _: &dyn ConsoleComponent) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "Not implemented"))
    }
    fn try_write<M: ConsoleComponent + 'static>(&self, component: M) -> io::Result<()>;
    fn commit(&self) -> io::Result<()>;

    fn try_send<M: ConsoleComponent + 'static>(&self, component: M) -> io::Result<()> {
        self.try_write(component)?;
        self.commit()
    }
    fn write<M: ConsoleComponent + 'static>(&self, component: M) {
        self.try_write(component).unwrap();
    }
    fn send<M: ConsoleComponent + 'static>(&self, component: M) {
        self.write(component);
        self.commit().unwrap();
    }
}

pub struct ConsoleReceiver {
}