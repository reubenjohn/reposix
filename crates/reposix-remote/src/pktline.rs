//! git protocol pkt-line frame reader/writer.
//!
//! A pkt-line is a 4-byte ASCII hex length header followed by a payload.
//! The four special short packets are:
//!
//! | Header | Meaning |
//! |---|---|
//! | `0000` | flush |
//! | `0001` | delim (protocol-v2 section boundary) |
//! | `0002` | response-end (protocol-v2 terminates helper response) |
//! | `0004` | empty-data (length includes the 4-byte header) |
//!
//! For the `stateless-connect` tunnel we need to (a) read frames from
//! git's stdin until we see a flush, encoding the raw bytes verbatim
//! into a buffer that we then pipe to `git upload-pack --stateless-rpc`,
//! and (b) count `want` lines inside `data` frames for audit + Phase 34
//! blob-limit instrumentation.
//!
//! This module is byte-oriented — no UTF-8 validation. The `Data`
//! variant is an owned `Vec<u8>` so callers can inspect payload bytes
//! (e.g. detect `b"want "` prefixes) without re-framing.

#![forbid(unsafe_code)]

use std::io::{self, Read, Write};

/// A single pkt-line frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Frame {
    /// 4-byte `0000` — flush.
    Flush,
    /// 4-byte `0001` — v2 delimiter between section header and body.
    Delim,
    /// 4-byte `0002` — v2 response end marker (server -> client only).
    ResponseEnd,
    /// 4-byte length header + N-byte payload.
    Data(Vec<u8>),
}

/// Read one pkt-line frame from `r`.
///
/// # Errors
/// - [`io::ErrorKind::UnexpectedEof`] on short read of the header.
/// - [`io::ErrorKind::InvalidData`] if the length header is not ASCII hex
///   or declares a length smaller than 4 and not one of the special
///   shorts (`0000`/`0001`/`0002`).
pub(crate) fn read_frame<R: Read>(r: &mut R) -> io::Result<Frame> {
    let mut hdr = [0u8; 4];
    r.read_exact(&mut hdr)?;
    let hex = std::str::from_utf8(&hdr)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "pkt-line header not ASCII"))?;
    let n = u32::from_str_radix(hex, 16)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "pkt-line header not hex"))?;
    match n {
        0 => Ok(Frame::Flush),
        1 => Ok(Frame::Delim),
        2 => Ok(Frame::ResponseEnd),
        3 => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pkt-line header length 3 is reserved and invalid",
        )),
        4 => Ok(Frame::Data(Vec::new())),
        _ => {
            // u32 -> usize is safe on 32/64-bit. Payload length is
            // total-length minus the 4-byte header.
            let payload_len = (n - 4) as usize;
            let mut payload = vec![0u8; payload_len];
            r.read_exact(&mut payload)?;
            Ok(Frame::Data(payload))
        }
    }
}

/// Append `frame`'s raw byte encoding to `out`.
///
/// Round-trips with [`read_frame`]: `read_frame(&mut &buf[..])` after
/// `encode_frame` returns the same variant.
pub(crate) fn encode_frame(frame: &Frame, out: &mut Vec<u8>) {
    match frame {
        Frame::Flush => out.extend_from_slice(b"0000"),
        Frame::Delim => out.extend_from_slice(b"0001"),
        Frame::ResponseEnd => out.extend_from_slice(b"0002"),
        Frame::Data(p) => {
            #[allow(clippy::cast_possible_truncation)]
            let total = (p.len() + 4) as u32;
            // Four-digit lowercase hex. pkt-line caps at 65516; we don't
            // enforce here because we only read frames from `upload-pack`,
            // which already enforces.
            let hdr = format!("{total:04x}");
            out.extend_from_slice(hdr.as_bytes());
            out.extend_from_slice(p);
        }
    }
}

/// Write `frame` to `w`.
///
/// Kept on the public surface so tests and future callers can stream
/// single frames without buffering. The tunnel itself uses
/// [`encode_frame`] into a `Vec<u8>` batch for speed.
///
/// # Errors
/// Any `io::Error` propagated from the writer.
#[allow(dead_code)]
pub(crate) fn write_frame<W: Write>(frame: &Frame, w: &mut W) -> io::Result<()> {
    let mut buf = Vec::with_capacity(match frame {
        Frame::Data(p) => p.len() + 4,
        _ => 4,
    });
    encode_frame(frame, &mut buf);
    w.write_all(&buf)
}

/// True if `payload` begins with `b"want "` — used by the RPC proxy to
/// count want-lines per fetch for audit + Phase 34 enforcement.
#[must_use]
pub(crate) fn is_want_line(payload: &[u8]) -> bool {
    payload.starts_with(b"want ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_flush() {
        let mut buf = Vec::new();
        encode_frame(&Frame::Flush, &mut buf);
        assert_eq!(buf, b"0000");
        let got = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(got, Frame::Flush);
    }

    #[test]
    fn round_trip_delim() {
        let mut buf = Vec::new();
        encode_frame(&Frame::Delim, &mut buf);
        assert_eq!(buf, b"0001");
        assert_eq!(read_frame(&mut &buf[..]).unwrap(), Frame::Delim);
    }

    #[test]
    fn round_trip_response_end() {
        let mut buf = Vec::new();
        encode_frame(&Frame::ResponseEnd, &mut buf);
        assert_eq!(buf, b"0002");
        assert_eq!(read_frame(&mut &buf[..]).unwrap(), Frame::ResponseEnd);
    }

    #[test]
    fn round_trip_data() {
        let mut buf = Vec::new();
        encode_frame(&Frame::Data(b"want abc\n".to_vec()), &mut buf);
        // total = payload(9) + hdr(4) = 13 = 0x000d
        assert_eq!(&buf[..4], b"000d");
        assert_eq!(&buf[4..], b"want abc\n");
        let got = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(got, Frame::Data(b"want abc\n".to_vec()));
    }

    #[test]
    fn round_trip_empty_data() {
        let mut buf = Vec::new();
        encode_frame(&Frame::Data(Vec::new()), &mut buf);
        assert_eq!(buf, b"0004");
        assert_eq!(read_frame(&mut &buf[..]).unwrap(), Frame::Data(Vec::new()));
    }

    #[test]
    fn rejects_non_hex_header() {
        let e = read_frame(&mut &b"zzzz"[..]).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_length_three() {
        let e = read_frame(&mut &b"0003"[..]).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn short_read_header_is_unexpected_eof() {
        let e = read_frame(&mut &b"00"[..]).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn short_read_payload_is_unexpected_eof() {
        // Claims 12 payload bytes (total 16 = 0x0010) but only 3 provided.
        let e = read_frame(&mut &b"0010abc"[..]).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn sequence_data_flush_response_end() {
        let mut buf = Vec::new();
        encode_frame(&Frame::Data(b"hello".to_vec()), &mut buf);
        encode_frame(&Frame::Flush, &mut buf);
        encode_frame(&Frame::ResponseEnd, &mut buf);
        let mut r = &buf[..];
        assert_eq!(read_frame(&mut r).unwrap(), Frame::Data(b"hello".to_vec()));
        assert_eq!(read_frame(&mut r).unwrap(), Frame::Flush);
        assert_eq!(read_frame(&mut r).unwrap(), Frame::ResponseEnd);
    }

    #[test]
    fn is_want_line_detects_want_prefix() {
        assert!(is_want_line(b"want abcdef\n"));
        assert!(!is_want_line(b"have abcdef\n"));
        assert!(!is_want_line(b"want")); // no trailing space
        assert!(!is_want_line(b""));
    }

    #[test]
    fn data_payload_may_contain_nul_bytes() {
        // Gotcha #3 ground-truth: a data frame carrying NULs round-trips.
        let payload = b"abc\x00\x00\x00def".to_vec();
        let mut buf = Vec::new();
        encode_frame(&Frame::Data(payload.clone()), &mut buf);
        let got = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(got, Frame::Data(payload));
    }
}
