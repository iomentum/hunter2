pub use hunter2_proc_macros::Hunter2;

use std::fmt::{Debug, Formatter, Result, Write};

// Allows to override the Debug implementation provided by a given type.
#[doc(hidden)]
pub struct Hidden<'borrow, T>(pub &'borrow T);

impl<'borrow, T> From<&'borrow T> for Hidden<'borrow, T> {
    fn from(ref_: &'borrow T) -> Hidden<'borrow, T> {
        Hidden(ref_)
    }
}

impl<'borrow, T> Debug for Hidden<'borrow, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut writer = HiddenWriter::new(f);

        // TODO(scrabsha): handle the alternate flag:
        // https://doc.rust-lang.org/std/fmt/struct.Formatter.html#method.alternate
        write!(writer, "{:?}", self.0)
    }
}

struct HiddenWriter<'fmt, 'a>(&'fmt mut Formatter<'a>);

impl<'fmt, 'a> HiddenWriter<'fmt, 'a> {
    fn new(f: &'fmt mut Formatter<'a>) -> HiddenWriter<'fmt, 'a> {
        HiddenWriter(f)
    }
}

impl<'fmt, 'a> Write for HiddenWriter<'fmt, 'a> {
    fn write_str(&mut self, s: &str) -> Result {
        // TODO(scrabsha): this will repeatitively call
        // `HiddenWriter::write_char`, which may not be that optimized.
        //
        // An option to remove this would be to store a sequence of `*` in an
        // `&'static str`, to slice it on demand and write this slice instead.
        s.chars().try_for_each(|c| self.write_char(c))
    }

    fn write_char(&mut self, c: char) -> Result {
        let char_to_write = match c {
            '\n' => '\n',
            _ => '*',
        };

        self.0.write_char(char_to_write)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
