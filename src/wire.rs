//! Varint length-delimited protobuf framing over an async stream.

use anyhow::{bail, Result};
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const MAX_MESSAGE: usize = 1 << 20;

pub async fn write_msg<W, M>(w: &mut W, msg: &M) -> Result<()>
where
    W: AsyncWrite + Unpin,
    M: Message,
{
    let buf = msg.encode_length_delimited_to_vec();
    w.write_all(&buf).await?;
    w.flush().await?;
    Ok(())
}

pub async fn read_msg<R, M>(r: &mut R) -> Result<M>
where
    R: AsyncRead + Unpin,
    M: Message + Default,
{
    let mut len: usize = 0;
    let mut shift = 0;
    loop {
        let b = r.read_u8().await?;
        len |= ((b & 0x7f) as usize) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            bail!("bad length prefix");
        }
    }
    if len > MAX_MESSAGE {
        bail!("message too large ({len} bytes)");
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(M::decode(&buf[..])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::*;

    #[tokio::test]
    async fn roundtrip_over_duplex() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        let msg = RemoteMessage {
            key_inject: Some(RemoteKeyInject { key_code: 66, direction: DIR_SHORT }),
            ..Default::default()
        };
        write_msg(&mut a, &msg).await.unwrap();
        let got: RemoteMessage = read_msg(&mut b).await.unwrap();
        assert_eq!(got, msg);
    }

    #[test]
    fn key_inject_wire_bytes() {
        // RemoteMessage{ key_inject (field 10) { key_code=66, direction=3 } }
        let msg = RemoteMessage {
            key_inject: Some(RemoteKeyInject { key_code: 66, direction: 3 }),
            ..Default::default()
        };
        let framed = msg.encode_length_delimited_to_vec();
        assert_eq!(framed, vec![6, 0x52, 4, 0x08, 66, 0x10, 3]);
    }
}
