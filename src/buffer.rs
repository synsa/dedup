use memchr::memchr;

use args::{Options};
use error::DedupError;

use std::io;
use std::default::Default;
use std::collections::HashSet;

pub struct UnsortedBufferDeduper<'a, W: io::Write + 'a> {
    buffer: &'a [u8],
    opts: Options,
    out: W,
    dup_store: HashSet<&'a [u8]>,
}

impl<'a, W: io::Write + 'a> UnsortedBufferDeduper<'a, W> {
    pub fn new<R: AsRef<[u8]>>(buffer: &'a R, output: W, opts: Options) -> Self {
        UnsortedBufferDeduper {
            buffer: buffer.as_ref(),
            out: output,
            dup_store: Default::default(),
            opts,
        }
    }

    pub fn run(mut self) -> Result<u64, DedupError> {
        let delim = self.opts.delim;
        let mut count: u64 = 0;
        while let Some(u) = memchr(delim, self.buffer) {
            let (mut ele, rest) = self.buffer.split_at(u);
            if self.opts.crlf {
                if let Some(&b'\r') = ele.last() {
                    ele = &ele[..ele.len() - 1];
                }
            }
            if self.dup_store.insert(ele) {
                self.out.write_all(ele)?;
                self.out.write_all(&[delim])?;
            }
            self.buffer = &rest[1..];
            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    static BREAKFAST: &str = "\
spam
ham
eggs
ham
ham eggs
eggs
ham
spam
";

    static BREAKFAST_DEDUP: &str = "\
spam
ham
eggs
ham eggs
";

    #[test]
    fn buf_breakfast_dedup() {
        let mut output: Vec<u8> = Vec::new();
        {
            let dedup = UnsortedBufferDeduper::new(&BREAKFAST, &mut output, Options::default());
            dedup.run().unwrap();
        }
        assert_eq!(BREAKFAST_DEDUP, str::from_utf8(&output).unwrap());
    }
}