use core::fmt::{self, Write};

const STDOUT: libc::c_int = 1;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            libc::write(STDOUT, s.as_ptr() as *const _, s.len());
        }
        Ok(())
    }
}

pub fn println(args: fmt::Arguments) {
    let mut stdout = Stdout;
    stdout.write_fmt(args).unwrap();
    stdout.write_str("\n").unwrap();
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::io::println(core::format_args!($($arg)*));
    };
}
