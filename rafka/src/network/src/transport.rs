use crate::core::{Message, Result};
use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct MessageCodec {
    max_frame_size: usize,
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = RafkaError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 4 {
            return Ok(None);
        }

        let size = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        if size > self.max_frame_size {
            return Err(RafkaError::MessageTooLarge {
                size,
                max: self.max_frame_size,
            });
        }

        if src.len() < 4 + size {
            return Ok(None);
        }

        src.advance(4);
        let frame = src.split_to(size);
        Ok(Some(bincode::deserialize(&frame)?))
    }
}

pub struct Transport<T> {
    framed: Framed<T, MessageCodec>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> Transport<T> {
    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        self.framed.send(message).await?;
        Ok(())
    }
}
