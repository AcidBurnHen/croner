#[derive(Clone)]
pub struct Printer {
    to_print: bool,
}

impl Printer {
    pub fn new(to_print: bool) -> Self {
        Self { to_print }
    }

    #[inline]
    pub fn write<S: AsRef<str>>(&self, msg: S) {
        if self.to_print {
            println!("{}", msg.as_ref())
        }
    }
}
