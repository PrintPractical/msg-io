# msg-io

msg-io is a Rust crate that provides a standard framework for working with I/O streams and structured data/protocols.

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