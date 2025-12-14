//! msg-io provides a framework for encoding and decoding messages over streams.
//!
//! It supports both sync and async I/O models via feature flags, though my main intention is for async.
//! The core traits `Encoder` and `Decoder` allow the user to implement the logic for decoding their messages.
//!
//! # Features
//!
//! - `sync`: Enables synchronous I/O support using standard library traits.
//! - `async` (default): Enables asynchronous I/O support using `futures` traits.
//! - `tokio`: Enables integration with the `tokio` runtime and its I/O traits.
//!
#[cfg(feature = "async")]
pub mod r#async;
pub mod constants;
pub mod decoder;
pub mod encoder;
#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(test)]
mod tests {
    #[cfg(feature = "sync")]
    use super::sync;
    #[cfg(feature = "tokio")]
    use super::tokio as tokio_crate;
    use super::{decoder, encoder};

    struct RawEncoder;
    impl encoder::Encoder<Vec<u8>> for RawEncoder {
        fn encode(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, String> {
            Ok(data.clone())
        }
    }

    struct Uint16FramedEncoder;
    impl encoder::Encoder<Vec<u8>> for Uint16FramedEncoder {
        fn encode(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, String> {
            let len = data.len();
            if len > u16::MAX as usize {
                return Err("Data too large to encode".to_string());
            }
            let mut encoded = Vec::with_capacity(2 + len);
            encoded.extend_from_slice(&(len as u16).to_be_bytes());
            encoded.extend_from_slice(data);
            Ok(encoded)
        }
    }
    struct Uint16FramedDecoder;
    impl decoder::Decoder<Vec<u8>> for Uint16FramedDecoder {
        fn decode(&mut self, data: &[u8]) -> decoder::DecoderResult<Vec<u8>> {
            match data.len() {
                len if len >= 2 => {
                    let msg_len = u16::from_be_bytes([data[0], data[1]]) as usize;
                    if let Some(data) = data.get(2..2 + msg_len) {
                        let msg = data.to_vec();
                        return decoder::DecoderResult::Done(msg, 2 + msg_len);
                    } else {
                        decoder::DecoderResult::Continue
                    }
                }
                _ => decoder::DecoderResult::Continue,
            }
        }
    }

    #[cfg(feature = "sync")]
    #[test]
    fn test_sync_message_io() {
        let pipe = match std::io::pipe() {
            Ok((reader, writer)) => (reader, writer),
            Err(e) => panic!("Failed to create pipe: {}", e),
        };
        let mut reader = sync::MessageIo::new_reader(pipe.0, Uint16FramedDecoder);
        let mut writer = sync::MessageIo::new_writer(pipe.1, Uint16FramedEncoder);

        // Test Successful Case
        let data = b"hello world!".to_vec();
        writer
            .write_message::<Vec<u8>>(&data)
            .expect("Failed to write message");
        let received = reader
            .read_message::<Vec<u8>>()
            .expect("Failed to read message")
            .expect("No message received");
        assert_eq!(data, received);

        // Test too large message
        let large_data = vec![0u8; 70000]; // larger than u16::MAX
        let write_result = writer.write_message::<Vec<u8>>(&large_data);
        assert!(write_result.is_err(), "Expected error for large message");

        drop(writer); // Close writer to simulate end of stream
        drop(reader); // Close reader

        // Test incomplete message
        let pipe = match std::io::pipe() {
            Ok((reader, writer)) => (reader, writer),
            Err(e) => panic!("Failed to create pipe: {}", e),
        };
        let mut reader = sync::MessageIo::new_reader(pipe.0, Uint16FramedDecoder);
        let mut writer = sync::MessageIo::new_writer(pipe.1, RawEncoder);
        let incomplete_data = b"\x00\x10hello".to_vec(); // Declares length 16, but only 5 bytes provided
        let _ = writer.write_message::<Vec<u8>>(&incomplete_data);
        drop(writer); // Close writer to simulate end of stream
        let read_result = reader.read_message::<Vec<u8>>();
        assert!(
            matches!(read_result, Ok(None)),
            "Expected None for incomplete message"
        );
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async_message_io() {
        let (rx, tx) = tokio::net::UnixStream::pair().expect("Failed to create UnixStream pair");
        let mut reader = tokio_crate::MessageTokio::new_reader(rx, Uint16FramedDecoder);
        let mut writer = tokio_crate::MessageTokio::new_writer(tx, Uint16FramedEncoder);

        // Test Successful Case
        let data = b"hello async world!".to_vec();
        writer
            .write_message::<Vec<u8>>(&data)
            .await
            .expect("Failed to write message");
        let received = reader
            .read_message::<Vec<u8>>()
            .await
            .expect("Failed to read message")
            .expect("No message received");
        assert_eq!(data, received);

        // Test too large message
        let large_data = vec![0u8; 70000]; // larger than u16::MAX
        let write_result = writer.write_message::<Vec<u8>>(&large_data).await;
        assert!(write_result.is_err(), "Expected error for large message");

        // Test incomplete message (use a fresh pair so writer can use a raw encoder)
        let (rx, tx) = tokio::net::UnixStream::pair().expect("Failed to create UnixStream pair");
        let mut reader = tokio_crate::MessageTokio::new_reader(rx, Uint16FramedDecoder);
        let mut writer = tokio_crate::MessageTokio::new_writer(tx, RawEncoder);
        let incomplete_data = b"\x00\x10hello".to_vec(); // Declares length 16, but only 5 bytes provided
        let _ = writer.write_message::<Vec<u8>>(&incomplete_data).await;
        drop(writer); // Close writer to simulate end of stream
        let read_result = reader.read_message::<Vec<u8>>().await;
        assert!(
            matches!(read_result, Ok(None)),
            "Expected None for incomplete message"
        );
    }
}
