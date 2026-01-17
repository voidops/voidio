#[cfg(test)]
mod tests {
    use voidio::console::{fstdout, stdout, Component, Console};
    #[test]
    fn logging () {
        {
            let cout = stdout();
            cout.send("Normal message to stdout");
            cout.send(Component::text("****************************").with_color(0x00FFFF));
        }
    }
    #[test]
    fn formatted_stdout()
    {
        let cout = fstdout(|msg| {
            "[" + Component::text("VoidIO").with_color(0xFF55FF) + "] " + msg
        });

        cout.send("New Message");

        cout.write("Partial message... ");

        cout.send(Component::text("OK").with_color(0x00FF00));
    }
}
