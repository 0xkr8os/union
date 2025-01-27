use sha3::{Digest, Keccak256};
use unionlabs::{
    errors::{ExpectedLength, InvalidLength},
    uint::U256,
    ByteArrayExt,
};

/// @dev Below is the encoding for `Chunk`, total 60*n+1+m bytes.
/// ```text
///   * Field           Bytes       Type            Index       Comments
///   * numBlocks       1           uint8           0           The number of blocks in this chunk
///   * block[0]        60          BlockContext    1           The first block in this chunk
///   * ......
///   * block[i]        60          BlockContext    60*i+1      The (i+1)'th block in this chunk
///   * ......
///   * block[n-1]      60          BlockContext    60*n-59     The last block in this chunk
///   * l2Transactions  dynamic     bytes           60*n+1
/// ```
#[derive(macros::Debug, Clone, PartialEq)]
pub struct ChunkV0 {
    pub blocks: Vec<BlockContext>,
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub l2_transactions: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ChunkV0DecodeError {
    #[error("the provided bytes were empty")]
    EmptyBytes,
    #[error("incorrect chunk length")]
    IncorrectChunkLength(#[source] InvalidLength),
    #[error("incorrect l2 transaction length")]
    IncorrectL2TransactionLength,
    #[error("not enough bytes to parse the l2 tx length")]
    NotEnoughL2TransactionLengthBytes,
}

impl ChunkV0 {
    /// Decode a v0 chunk.
    ///
    /// # Errors
    ///
    /// See [`ChunkV0DecodeError`] for the possible failure modes for this function.
    #[allow(clippy::missing_panics_doc)] // panic is unreachable
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, ChunkV0DecodeError> {
        let bz = bz.as_ref();

        let len: usize = bz
            .first()
            .copied()
            .ok_or(ChunkV0DecodeError::EmptyBytes)?
            .into();

        let bz_len = bz.len();
        let expected_len = (len * BlockContext::LENGTH) + 1;
        if bz_len < expected_len {
            return Err(ChunkV0DecodeError::IncorrectChunkLength(InvalidLength {
                expected: ExpectedLength::Gte(expected_len),
                found: bz_len,
            }));
        }

        let mut blocks = vec![];

        for i in 0..len {
            blocks.push(BlockContext::decode(
                bz[((BlockContext::LENGTH * i) + 1)..=(BlockContext::LENGTH * (i + 1))]
                    .try_into()
                    .expect("size is the expected length; qed;"),
            ));
        }

        let mut l2_transactions = vec![];

        let mut ptr = (BlockContext::LENGTH * len) + 1;

        loop {
            if bz[ptr..].is_empty() {
                break;
            }

            let tx_len = u32::from_be_bytes(
                bz.get(ptr..ptr + 4)
                    .ok_or(ChunkV0DecodeError::NotEnoughL2TransactionLengthBytes)?
                    .try_into()
                    .unwrap(),
            );

            ptr += 4;

            l2_transactions.push(
                bz.get(ptr..ptr + tx_len as usize)
                    .ok_or(ChunkV0DecodeError::IncorrectL2TransactionLength)?
                    .to_vec(),
            );

            ptr += tx_len as usize;
        }

        Ok(Self {
            blocks,
            l2_transactions,
        })
    }
}

/// @dev Below is the encoding for `Chunk`, total 60*n+1 bytes.
/// The only difference between `ChunkCodecV0` is we remove `l2Transactions` from chunk encoding.
/// ```text
///   * Field           Bytes       Type            Index       Comments
///   * numBlocks       1           uint8           0           The number of blocks in this chunk
///   * block[0]        60          BlockContext    1           The first block in this chunk
///   * ......
///   * block[i]        60          BlockContext    60*i+1      The (i+1)'th block in this chunk
///   * ......
///   * block[n-1]      60          BlockContext    60*n-59     The last block in this chunk
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkV1 {
    pub blocks: Vec<BlockContext>,
}

impl ChunkV1 {
    /// Decode a v1 chunk.
    ///
    /// # Errors
    ///
    /// See [`ChunkV1DecodeError`] for the possible failure modes for this function.
    #[allow(clippy::missing_panics_doc)] // panic is unreachable
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, ChunkV1DecodeError> {
        let bz = bz.as_ref();

        let len: usize = bz
            .first()
            .copied()
            .ok_or(ChunkV1DecodeError::EmptyBytes)?
            .into();

        let bz_len = bz.len();
        if bz_len != len * BlockContext::LENGTH + 1 {
            return Err(ChunkV1DecodeError::IncorrectChunkLength(InvalidLength {
                expected: ExpectedLength::Exact(len),
                found: bz_len,
            }));
        }

        let mut blocks = vec![];

        for i in 0..len {
            blocks.push(BlockContext::decode(
                bz[((BlockContext::LENGTH * i) + 1)..=(BlockContext::LENGTH * (i + 1))]
                    .try_into()
                    .expect("size is the expected length; qed;"),
            ));
        }

        Ok(Self { blocks })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ChunkV1DecodeError {
    #[error("the provided bytes were empty")]
    EmptyBytes,
    #[error("incorrect chunk length")]
    IncorrectChunkLength(#[source] InvalidLength),
}

/// @dev Below is the encoding for `BlockContext`, total 60 bytes.
/// ```text
///   * Field                   Bytes      Type         Index  Comments
///   * blockNumber             8          uint64       0      The height of this block.
///   * timestamp               8          uint64       8      The timestamp of this block.
///   * baseFee                 32         uint256      16     The base fee of this block. Currently, it is always 0, because we disable EIP-1559.
///   * gasLimit                8          uint64       48     The gas limit of this block.
///   * numTransactions         2          uint16       56     The number of transactions in this block, both L1 & L2 txs.
///   * numL1Messages           2          uint16       58     The number of l1 messages in this block.
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BlockContext {
    pub block_number: u64,
    pub timestamp: u64,
    pub base_fee: U256,
    pub gas_limit: u64,
    pub num_transactions: u16,
    pub num_l1_messages: u16,
}

impl BlockContext {
    const LENGTH: usize = 60;

    #[must_use]
    pub(crate) fn decode(bz: [u8; Self::LENGTH]) -> Self {
        Self {
            block_number: u64::from_be_bytes(bz.array_slice::<0, 8>()),
            timestamp: u64::from_be_bytes(bz.array_slice::<8, 8>()),
            base_fee: U256::from_be_bytes(bz.array_slice::<16, 32>()),
            gas_limit: u64::from_be_bytes(bz.array_slice::<48, 8>()),
            num_transactions: u16::from_be_bytes(bz.array_slice::<56, 2>()),
            num_l1_messages: u16::from_be_bytes(bz.array_slice::<58, 2>()),
        }
    }

    /// NOTE: V0 and V1 are the exact same.
    ///
    /// <https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/libraries/codec/ChunkCodecV0.sol#L76>
    /// <https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/libraries/codec/ChunkCodecV1.sol#L65>
    pub(crate) fn copy_block_context(&self, hasher: &mut Keccak256) {
        hasher.update(self.block_number.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.base_fee.to_be_bytes());
        hasher.update(self.gas_limit.to_be_bytes());
        hasher.update(self.num_transactions.to_be_bytes());
    }
}
