use std::borrow::Cow;

use unionlabs::cosmos::ics23::length_op::LengthOp;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ApplyError {
    #[error("required 32 bytes, found ({0})")]
    Required32Bytes(usize),
    #[error("required 64 bytes, ({0})")]
    Required64Bytes(usize),
    #[error("unsupported op ({0})")]
    UnsupportedOp(LengthOp),
}

pub fn apply<'a>(length_op: &LengthOp, data: Cow<'a, [u8]>) -> Result<Cow<'a, [u8]>, ApplyError> {
    match length_op {
        LengthOp::NoPrefix => Ok(data),
        LengthOp::VarProto => {
            let mut len = Vec::new();
            prost::encoding::encode_varint(data.len() as u64, &mut len);
            len.extend_from_slice(&data);
            Ok(len.into())
        }
        LengthOp::Require32Bytes => {
            if data.len() != 32 {
                return Err(ApplyError::Required32Bytes(data.len()))?;
            }
            Ok(data)
        }
        LengthOp::Require64Bytes => {
            if data.len() != 64 {
                return Err(ApplyError::Required64Bytes(data.len()));
            }
            Ok(data)
        }
        LengthOp::Fixed32Little => {
            let mut d = (data.len() as u32).to_le_bytes().to_vec();
            d.extend_from_slice(&data);
            Ok(d.into())
        }
        op => Err(ApplyError::UnsupportedOp(*op)),
    }
}