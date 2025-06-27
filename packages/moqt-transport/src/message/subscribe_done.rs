use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscribeDone {
    pub request_id: u64,
    pub status_code: u64,
    pub stream_count: u64,
    pub reason: String,
}

impl SubscribeDone {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        vi.encode(self.request_id, buf)?;
        vi.encode(self.status_code, buf)?;
        vi.encode(self.stream_count, buf)?;

        let bytes = self.reason.as_bytes();
        if bytes.len() > 8192 {
            return Err(IoError::new(ErrorKind::InvalidData, "reason too long").into());
        }
        vi.encode(bytes.len() as u64, buf)?;
        buf.put_slice(bytes);

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;
        let status_code = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "status code"))?;
        let stream_count = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "stream count"))?;
        let reason_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "reason length"))?
            as usize;

        if reason_len > 8192 {
            return Err(IoError::new(ErrorKind::InvalidData, "reason too long").into());
        }
        if buf.len() < reason_len {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "reason").into());
        }

        let value = buf.split_to(reason_len);
        let reason = String::from_utf8(value.to_vec())
            .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;

        Ok(SubscribeDone {
            request_id,
            status_code,
            stream_count,
            reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_with_reason() {
        let msg = SubscribeDone {
            request_id: 1,
            status_code: 3,
            stream_count: 2,
            reason: "track ended".into(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeDone::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_without_reason() {
        let msg = SubscribeDone {
            request_id: 5,
            status_code: 4,
            stream_count: 0,
            reason: String::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeDone::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_fails_on_oversized_reason() {
        let mut buf = BytesMut::new();
        let mut vi = crate::coding::VarInt;
        vi.encode(1, &mut buf).unwrap(); // request_id
        vi.encode(2, &mut buf).unwrap(); // status_code
        vi.encode(3, &mut buf).unwrap(); // stream_count
        vi.encode(8193, &mut buf).unwrap(); // reason length > allowed
        buf.resize(buf.len() + 8193, 0);

        assert!(SubscribeDone::decode(&mut buf).is_err());
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        let mut vi = crate::coding::VarInt;
        vi.encode(10, &mut buf).unwrap(); // request id only

        match SubscribeDone::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
