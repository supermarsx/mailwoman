//! COMPRESS=DEFLATE (RFC 4978): raw DEFLATE (RFC 1951, no zlib header) in
//! both directions once the server accepts `COMPRESS DEFLATE`.
//!
//! The command is sent after authentication and after TLS is up. RFC 4978 §2
//! forbids compressing on top of TLS-level compression; rustls never
//! negotiates TLS compression, so that case cannot arise here.
//!
//! Every flush of the writer ends the compressed data with a sync flush, so the
//! server can decode a command as soon as it is sent (RFC 4978 §4). The reader
//! inflates into the caller's buffer, so one read never produces more bytes
//! than the caller asked for, however well the input compresses.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress, Status};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// The command that starts compression (sent without the tag).
pub const COMMAND: &str = "COMPRESS DEFLATE";

/// The capability that advertises it.
pub const CAPABILITY: &str = "COMPRESS=DEFLATE";

/// Compressed bytes the writer holds before it stops accepting input until
/// the inner stream catches up.
const HIGH_WATER: usize = 64 * 1024;

/// Size of the buffer for compressed bytes read from the inner stream.
const READ_CHUNK: usize = 16 * 1024;

/// A stream that deflates what is written and inflates what is read.
pub struct DeflateStream<S> {
    inner: S,
    deflate: Compress,
    /// Compressed bytes not yet written to `inner`, from `out_pos`.
    out: Vec<u8>,
    out_pos: usize,
    /// Data was compressed since the last sync flush.
    unsynced: bool,
    inflate: Decompress,
    /// Compressed bytes read from `inner`, valid in `in_start..in_end`.
    input: Box<[u8]>,
    in_start: usize,
    in_end: usize,
    eof: bool,
}

impl<S> DeflateStream<S> {
    pub fn new(inner: S) -> Self {
        DeflateStream {
            inner,
            deflate: Compress::new(Compression::default(), false),
            out: Vec::new(),
            out_pos: 0,
            unsynced: false,
            inflate: Decompress::new(false),
            input: vec![0; READ_CHUNK].into_boxed_slice(),
            in_start: 0,
            in_end: 0,
            eof: false,
        }
    }

    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Bytes in and out of the compressor and decompressor, for diagnostics:
    /// `(plain written, compressed written, compressed read, plain read)`.
    pub fn totals(&self) -> (u64, u64, u64, u64) {
        (
            self.deflate.total_in(),
            self.deflate.total_out(),
            self.inflate.total_in(),
            self.inflate.total_out(),
        )
    }

    fn compress(&mut self, mut input: &[u8], flush: FlushCompress) -> io::Result<()> {
        loop {
            if self.out.capacity() - self.out.len() < 1024 {
                self.out.reserve(8 * 1024);
            }
            let spare = self.out.capacity() - self.out.len();
            let before_in = self.deflate.total_in();
            let before_len = self.out.len();
            self.deflate
                .compress_vec(input, &mut self.out, flush)
                .map_err(io::Error::other)?;
            let consumed = (self.deflate.total_in() - before_in) as usize;
            input = &input[consumed..];
            let produced = self.out.len() - before_len;
            // Done once all input is taken and the compressor did not fill the
            // space it was given (a full buffer may hide more flush output).
            if input.is_empty() && produced < spare {
                return Ok(());
            }
        }
    }
}

impl<S: AsyncWrite + Unpin> DeflateStream<S> {
    /// Write pending compressed bytes to the inner stream.
    fn poll_drain(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        while self.out_pos < self.out.len() {
            let n = ready!(Pin::new(&mut self.inner).poll_write(cx, &self.out[self.out_pos..]))?;
            if n == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
            self.out_pos += n;
        }
        self.out.clear();
        self.out_pos = 0;
        Poll::Ready(Ok(()))
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for DeflateStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if this.out.len() - this.out_pos > HIGH_WATER {
            ready!(this.poll_drain(cx))?;
        }
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        this.compress(buf, FlushCompress::None)?;
        this.unsynced = true;
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.unsynced {
            this.compress(&[], FlushCompress::Sync)?;
            this.unsynced = false;
        }
        ready!(this.poll_drain(cx))?;
        Pin::new(&mut this.inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        ready!(Pin::new(&mut *this).poll_flush(cx))?;
        Pin::new(&mut this.inner).poll_shutdown(cx)
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for DeflateStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        loop {
            // Inflate what is buffered (possibly nothing: the decompressor may
            // still hold output from earlier input).
            let before_in = this.inflate.total_in();
            let before_out = this.inflate.total_out();
            let status = this
                .inflate
                .decompress(
                    &this.input[this.in_start..this.in_end],
                    buf.initialize_unfilled(),
                    FlushDecompress::None,
                )
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let consumed = (this.inflate.total_in() - before_in) as usize;
            let produced = (this.inflate.total_out() - before_out) as usize;
            this.in_start += consumed;
            if produced > 0 {
                buf.advance(produced);
                return Poll::Ready(Ok(()));
            }
            if status == Status::StreamEnd || this.eof {
                // The server ended the DEFLATE stream or closed the socket.
                return Poll::Ready(Ok(()));
            }
            if consumed > 0 {
                continue;
            }
            // Need more compressed input.
            if this.in_start == this.in_end {
                this.in_start = 0;
                this.in_end = 0;
            } else if this.in_start == 0 && this.in_end == this.input.len() {
                // A full buffer the decompressor neither consumes nor turns
                // into output: not a DEFLATE stream it can make progress on.
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "compressed input makes no progress",
                )));
            } else if this.in_end == this.input.len() {
                this.input.copy_within(this.in_start..this.in_end, 0);
                this.in_end -= this.in_start;
                this.in_start = 0;
            }
            let mut rb = ReadBuf::new(&mut this.input[this.in_end..]);
            ready!(Pin::new(&mut this.inner).poll_read(cx, &mut rb))?;
            let n = rb.filled().len();
            if n == 0 {
                this.eof = true;
            }
            this.in_end += n;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use flate2::read::DeflateDecoder;
    use flate2::write::DeflateEncoder;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

    use super::*;

    #[tokio::test]
    async fn round_trip_between_two_ends() {
        let (a, b) = duplex(1024);
        let mut client = DeflateStream::new(a);
        let mut server = DeflateStream::new(b);
        let lines: Vec<String> = (0..200)
            .map(|i| format!("A{i:04} UID FETCH {i}:* (FLAGS)\r\n"))
            .collect();
        let expected = lines.concat();
        let writer = tokio::spawn(async move {
            for line in &lines {
                client.write_all(line.as_bytes()).await.unwrap();
                client.flush().await.unwrap();
            }
            client.shutdown().await.unwrap();
            client.totals()
        });
        let mut got = Vec::new();
        let mut chunk = [0u8; 7];
        while got.len() < expected.len() {
            let n =
                tokio::time::timeout(std::time::Duration::from_secs(5), server.read(&mut chunk))
                    .await
                    .expect("flushed data must arrive")
                    .unwrap();
            assert!(n > 0, "early EOF after {} bytes", got.len());
            got.extend_from_slice(&chunk[..n]);
        }
        assert_eq!(String::from_utf8(got).unwrap(), expected);
        let (plain, packed, _, _) = writer.await.unwrap();
        assert_eq!(plain, expected.len() as u64);
        assert!(packed < plain, "repetitive commands should shrink");
    }

    #[tokio::test]
    async fn each_flush_makes_the_command_decodable_on_its_own() {
        let (a, mut raw) = duplex(64 * 1024);
        let mut client = DeflateStream::new(a);
        client.write_all(b"A0001 NOOP\r\n").await.unwrap();
        client.flush().await.unwrap();
        // Read only what the flush produced and inflate it with an
        // independent raw-DEFLATE decoder: the full command must come out.
        let mut wire = vec![0u8; 1024];
        let n = tokio::time::timeout(std::time::Duration::from_secs(5), raw.read(&mut wire))
            .await
            .expect("the flush must put the command on the wire")
            .unwrap();
        let mut dec = flate2::Decompress::new(false);
        let mut plain = vec![0u8; 1024];
        dec.decompress(&wire[..n], &mut plain, FlushDecompress::Sync)
            .unwrap();
        let out = dec.total_out() as usize;
        assert_eq!(&plain[..out], b"A0001 NOOP\r\n");
        // A sync flush ends with the empty stored block 00 00 FF FF.
        assert!(wire[..n].ends_with(&[0x00, 0x00, 0xff, 0xff]));
    }

    #[tokio::test]
    async fn reads_a_stream_compressed_by_another_implementation() {
        let mut enc = DeflateEncoder::new(Vec::new(), flate2::Compression::best());
        enc.write_all(b"* OK [CAPABILITY IMAP4rev1 IDLE] done\r\n")
            .unwrap();
        enc.flush().unwrap();
        enc.write_all(b"A0002 OK fetched\r\n").unwrap();
        let wire = enc.finish().unwrap();
        // Deliver one byte at a time.
        let (mut tx, rx) = duplex(1);
        tokio::spawn(async move {
            for b in wire {
                tx.write_all(&[b]).await.unwrap();
            }
        });
        let mut client = DeflateStream::new(rx);
        let mut got = String::new();
        client.read_to_string(&mut got).await.unwrap();
        assert_eq!(
            got,
            "* OK [CAPABILITY IMAP4rev1 IDLE] done\r\nA0002 OK fetched\r\n"
        );
    }

    #[tokio::test]
    async fn written_bytes_are_plain_raw_deflate() {
        let (a, mut raw) = duplex(1 << 20);
        let mut client = DeflateStream::new(a);
        let body = "x".repeat(100_000);
        client.write_all(body.as_bytes()).await.unwrap();
        client.shutdown().await.unwrap();
        drop(client);
        let mut wire = Vec::new();
        raw.read_to_end(&mut wire).await.unwrap();
        assert!(wire.len() < 1_000);
        let mut plain = String::new();
        DeflateDecoder::new(&wire[..])
            .read_to_string(&mut plain)
            .unwrap();
        assert_eq!(plain, body);
    }

    #[tokio::test]
    async fn corrupt_input_is_an_error_not_a_panic() {
        let (mut tx, rx) = duplex(64);
        tx.write_all(&[0xff, 0xff, 0xff, 0xff, 0xff]).await.unwrap();
        drop(tx);
        let mut client = DeflateStream::new(rx);
        let mut buf = [0u8; 16];
        let err = client.read(&mut buf).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn eof_before_any_data_reads_zero() {
        let (tx, rx) = duplex(64);
        drop(tx);
        let mut client = DeflateStream::new(rx);
        let mut buf = [0u8; 16];
        assert_eq!(client.read(&mut buf).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn a_highly_compressible_reply_fills_only_the_callers_buffer() {
        let mut enc = DeflateEncoder::new(Vec::new(), flate2::Compression::best());
        enc.write_all(&vec![b'a'; 4 << 20]).unwrap();
        let wire = enc.finish().unwrap();
        let (mut tx, rx) = duplex(1 << 16);
        tokio::spawn(async move { tx.write_all(&wire).await.unwrap() });
        let mut client = DeflateStream::new(rx);
        let mut buf = [0u8; 4096];
        let n = client.read(&mut buf).await.unwrap();
        assert!(n > 0 && n <= buf.len());
        let mut total = n;
        loop {
            let n = client.read(&mut buf).await.unwrap();
            if n == 0 {
                break;
            }
            total += n;
        }
        assert_eq!(total, 4 << 20);
    }
}
