//! Git remote helper line protocol I/O.
//!
//! The single-source-of-truth for stdout writes (which are protocol-reserved)
//! and stderr diagnostics. Outside this module, `println!` is a hard ban
//! enforced by `#![deny(clippy::print_stdout)]` at `main.rs`.

#![forbid(unsafe_code)]

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};

/// Wraps a paired Read+Write (typically stdin+stdout) for the git
/// remote-helper line protocol.
#[allow(clippy::print_stdout, clippy::print_stderr)] // this struct OWNS the protocol
pub struct Protocol<R: Read, W: Write> {
    reader: BufReader<R>,
    writer: BufWriter<W>,
}

impl<R: Read, W: Write> Protocol<R, W> {
    /// Build a new protocol I/O wrapper.
    pub fn new(r: R, w: W) -> Self {
        Self {
            reader: BufReader::new(r),
            writer: BufWriter::new(w),
        }
    }

    /// Read one line from the protocol stream. Returns `Ok(None)` at EOF.
    /// The trailing `\n` is stripped; internal whitespace is preserved.
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying reader.
    pub fn read_line(&mut self) -> io::Result<Option<String>> {
        let mut buf = String::new();
        let n = self.reader.read_line(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        Ok(Some(buf))
    }

    /// Write a line to stdout (appending `\n`).
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub fn send_line(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Write a single blank line (the protocol's response terminator).
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub fn send_blank(&mut self) -> io::Result<()> {
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Write raw bytes (no newline appended). Used for fast-import blob
    /// payloads where the byte length is announced via a `data N` line.
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub fn send_raw(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)
    }

    /// Flush the writer's buffer to the underlying stdout.
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    /// Emit a diagnostic to stderr — the SOLE non-protocol output channel.
    /// Stdout is reserved for the git remote helper protocol; using
    /// `eprintln!` here is the documented, mechanical lock against
    /// accidental stdout pollution.
    #[allow(dead_code)] // public lock; main.rs uses a top-level diag() wrapper today
    pub fn diag(msg: &str) {
        eprintln!("{msg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_line_strips_newline() {
        let input = b"capabilities\n".as_slice();
        let mut output: Vec<u8> = Vec::new();
        let mut p = Protocol::new(input, &mut output);
        let got = p.read_line().unwrap();
        assert_eq!(got.as_deref(), Some("capabilities"));
    }

    #[test]
    fn read_line_returns_none_at_eof() {
        let input = b"".as_slice();
        let mut output: Vec<u8> = Vec::new();
        let mut p = Protocol::new(input, &mut output);
        assert_eq!(p.read_line().unwrap(), None);
    }

    #[test]
    fn send_line_appends_newline() {
        let input = b"".as_slice();
        let mut output: Vec<u8> = Vec::new();
        let mut p = Protocol::new(input, &mut output);
        p.send_line("import").unwrap();
        p.flush().unwrap();
        drop(p);
        assert_eq!(output, b"import\n");
    }
}
