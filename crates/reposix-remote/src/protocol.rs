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
pub(crate) struct Protocol<R: Read, W: Write> {
    reader: BufReader<R>,
    writer: BufWriter<W>,
}

impl<R: Read, W: Write> Protocol<R, W> {
    /// Build a new protocol I/O wrapper.
    pub(crate) fn new(r: R, w: W) -> Self {
        Self {
            reader: BufReader::new(r),
            writer: BufWriter::new(w),
        }
    }

    /// Read one line from the protocol stream. Returns `Ok(None)` at EOF.
    /// The trailing `\n` is stripped; internal whitespace is preserved.
    ///
    /// This path is the UTF-8-decoding command-line reader used for the
    /// remote-helper control protocol (`capabilities`, `list`, `import`,
    /// `export`, etc.) — all of which are pure ASCII. Blob payloads must
    /// use [`read_raw_line`] or [`read_bytes_exact`] instead to preserve
    /// byte-fidelity (CRLF, non-UTF-8).
    ///
    /// [`read_raw_line`]: Self::read_raw_line
    /// [`read_bytes_exact`]: Self::read_bytes_exact
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying reader.
    pub(crate) fn read_line(&mut self) -> io::Result<Option<String>> {
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

    /// Read one line as raw bytes. Returns `Ok(None)` at EOF.
    /// The trailing `\n` is stripped if present; any preceding `\r` is
    /// **preserved** (unlike [`read_line`]). No UTF-8 validation is
    /// performed — non-UTF-8 bytes flow through unchanged.
    ///
    /// Used by [`super`]'s `ProtoReader` for blob-body plumbing where
    /// `\r\n` line terminators and non-UTF-8 payloads must round-trip
    /// byte-for-byte.
    ///
    /// [`read_line`]: Self::read_line
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying reader.
    pub(crate) fn read_raw_line(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut buf: Vec<u8> = Vec::new();
        let n = self.reader.read_until(b'\n', &mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        if buf.last() == Some(&b'\n') {
            buf.pop();
        }
        Ok(Some(buf))
    }

    /// Read exactly `buf.len()` raw bytes from the stream. No newline or
    /// UTF-8 processing.
    ///
    /// Used for pulling fast-export blob payloads announced via `data N`
    /// lines. Companion to [`read_raw_line`] (which handles the headers).
    ///
    /// [`read_raw_line`]: Self::read_raw_line
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying reader, including
    /// [`io::ErrorKind::UnexpectedEof`] on short reads.
    #[allow(dead_code)] // public API surface; exposed for future callers
                        // that want explicit N-byte reads.
    pub(crate) fn read_bytes_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf)
    }

    /// Return a mutable reference to the internal `BufReader`. Used by
    /// the pkt-line parser in the `stateless-connect` handler so it can
    /// consume bytes from the SAME buffer as [`read_line`] — crucial
    /// for gotcha #3 (binary stdin throughout; mixing two `BufReader`
    /// instances on the same underlying stream corrupts the pkt-line
    /// stream).
    ///
    /// [`read_line`]: Self::read_line
    pub(crate) fn reader_mut(&mut self) -> &mut BufReader<R> {
        &mut self.reader
    }

    /// Return a mutable reference to the internal `BufWriter` so callers
    /// can stream large response payloads (e.g. `upload-pack` output)
    /// without copying through [`send_raw`].
    ///
    /// [`send_raw`]: Self::send_raw
    #[allow(dead_code)] // public surface; today's callers use send_raw.
    pub(crate) fn writer_mut(&mut self) -> &mut BufWriter<W> {
        &mut self.writer
    }

    /// Write a line to stdout (appending `\n`).
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub(crate) fn send_line(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Write a single blank line (the protocol's response terminator).
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub(crate) fn send_blank(&mut self) -> io::Result<()> {
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Write raw bytes (no newline appended). Used for fast-import blob
    /// payloads where the byte length is announced via a `data N` line.
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub(crate) fn send_raw(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)
    }

    /// Flush the writer's buffer to the underlying stdout.
    ///
    /// # Errors
    /// Returns any [`io::Error`] from the underlying writer.
    pub(crate) fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    /// Emit a diagnostic to stderr — the SOLE non-protocol output channel.
    /// Stdout is reserved for the git remote helper protocol; using
    /// `eprintln!` here is the documented, mechanical lock against
    /// accidental stdout pollution.
    #[allow(dead_code)] // public lock; main.rs uses a top-level diag() wrapper today
    pub(crate) fn diag(msg: &str) {
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
