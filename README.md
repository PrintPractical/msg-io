# msg-io

msg-io is a Rust crate that provides a standard framework for working with I/O streams and structured data/protocols.

### WARNING: Random API changes, documentation mismatches, etc will continue until 0.1.0.

## Background

I find myself working on a lot of Rust projects in which I am communicating with other entities that encode their data differently over the wire. An example is some external entities send their data over Unix Domain Sockets encoded like this:

```
<32bit BE Length><Data>
```

This may be a simple example, but the framework is intended to be flexible. A naive developer may implement the following in a tokio-based Rust project:

```rust
let mut len = [0u8;4];
tokio::select! {
    _ = some_future() => {
        // do some work
    }
    // vvv - this is unsafe! - vvv
    Ok(res) = stream.read_exact(&mut len) => {
        let len = u32::from_be_bytes(len);
        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload); // Check return
    }
}
```

read_exact() is not cancellation safe! It's possible a partial read could occur, and then one of the other futures resolves, resulting in a loss of data.

I just want a simple API that allows me to read protocol-like messages in select! loops without having to worry about cancellation safety.

## Example

msg-io defines an `Encoder` and `Decoder` trait. These traits are responsible for defining how a structure is encoded/decoded onto/from the wire. This can be used to fix the situation from the background:

```rust
struct Uint32FramedDecoder;
impl decoder::Decoder<Vec<u8>> for Uint16FramedDecoder {
    fn decode(&mut self, data: &[u8]) -> decoder::DecoderResult<Vec<u8>> {
        match data.len() {
            len if len >= 4 => {
                let msg_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
                if let Some(data) = data.get(4..4 + msg_len) {
                    let msg = data.to_vec();
                    return decoder::DecoderResult::Done(msg, 4 + msg_len);
                } else {
                    decoder::DecoderResult::Continue
                }
            }
            _ => decoder::DecoderResult::Continue,
        }
    }
}
```

The \[Async\]MessageIo object handles maintaining the buffer of data between read calls. When a read call returns, the Decoder is called with the current buffer. If more data is needed, the Decoder can return `Continue`, which will read more data. If there's enough data, but something unexpected occurs, i.e. some protocol error, the decoder can return an `Error`. Otherwise, the decoder should decode the object with the available data, and then return `Done` status with the amount of data used in this cycle. Since read() is cancel safe, the `read_message()` api can be used in a select loop.

```rust
let mut reader = AsyncMessageIo::new_reader(stream, Uint32FramedDecoder);
tokio::select! {
    _ = some_future() => {
        // do some work
    }
    res = reader.read_message() => {
        // Check error / process result
    }
}
```
