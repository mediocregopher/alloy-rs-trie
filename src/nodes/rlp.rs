use alloy_primitives::{B256, hex, keccak256};
use alloy_rlp::EMPTY_STRING_CODE;
use arrayvec::ArrayVec;
use core::fmt;

const MAX: usize = 33;

/// An RLP-encoded node.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct RlpNode(ArrayVec<u8, MAX>);

impl alloy_rlp::Decodable for RlpNode {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let bytes = alloy_rlp::Header::decode_bytes(buf, false)?;
        Self::from_raw_rlp(bytes)
    }
}

impl core::ops::Deref for RlpNode {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RlpNode {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[u8]> for RlpNode {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for RlpNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RlpNode({})", hex::encode_prefixed(&self.0))
    }
}

impl RlpNode {
    /// Creates a new RLP-encoded node from the given data.
    ///
    /// Returns `None` if the data is too large (greater than 33 bytes).
    #[inline]
    pub fn from_raw(data: &[u8]) -> Option<Self> {
        let mut arr = ArrayVec::new();
        arr.try_extend_from_slice(data).ok()?;
        Some(Self(arr))
    }

    /// Creates a new RLP-encoded node from the given data.
    #[inline]
    pub fn from_raw_rlp(data: &[u8]) -> alloy_rlp::Result<Self> {
        Self::from_raw(data).ok_or(alloy_rlp::Error::Custom("RLP node too large"))
    }

    /// Given an RLP-encoded node, returns it either as `rlp(node)` or `rlp(keccak(rlp(node)))`.
    #[doc(alias = "rlp_node")]
    #[inline]
    pub fn from_rlp(rlp: &[u8]) -> Self {
        if rlp.len() < 32 {
            // SAFETY: `rlp` is less than max capacity (33).
            unsafe { Self::from_raw(rlp).unwrap_unchecked() }
        } else {
            Self::word_rlp(&keccak256(rlp))
        }
    }

    /// RLP-encodes the given word and returns it as a new RLP node.
    #[inline]
    pub fn word_rlp(word: &B256) -> Self {
        let mut arr = [0u8; 33];
        arr[0] = EMPTY_STRING_CODE + 32;
        arr[1..].copy_from_slice(word.as_slice());
        Self(ArrayVec::from(arr))
    }

    /// Returns true if this is an RLP-encoded hash.
    #[inline]
    pub fn is_hash(&self) -> bool {
        self.len() == B256::len_bytes() + 1
    }

    /// Returns the RLP-encoded node as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns hash if this is an RLP-encoded hash
    #[inline]
    pub fn as_hash(&self) -> Option<B256> {
        if self.is_hash() { Some(B256::from_slice(&self.0[1..])) } else { None }
    }
}

#[cfg(feature = "arbitrary")]
impl<'u> arbitrary::Arbitrary<'u> for RlpNode {
    fn arbitrary(g: &mut arbitrary::Unstructured<'u>) -> arbitrary::Result<Self> {
        let len = g.int_in_range(0..=MAX)?;
        let mut arr = ArrayVec::new();
        arr.try_extend_from_slice(g.bytes(len)?).unwrap();
        Ok(Self(arr))
    }
}

#[cfg(feature = "arbitrary")]
impl proptest::arbitrary::Arbitrary for RlpNode {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;
        proptest::collection::vec(proptest::prelude::any::<u8>(), 0..=MAX)
            .prop_map(|vec| Self::from_raw(&vec).unwrap())
            .boxed()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RlpNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = hex::encode_prefixed(&self.0);
        serializer.serialize_str(&hex_string)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RlpNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        let bytes = hex::decode(&hex_string).map_err(serde::de::Error::custom)?;
        
        if bytes.len() > MAX {
            return Err(serde::de::Error::custom(format!(
                "RlpNode hex string too long: {} bytes (max {})",
                bytes.len(),
                MAX
            )));
        }
        
        let mut arr = ArrayVec::new();
        arr.try_extend_from_slice(&bytes).map_err(|_| {
            serde::de::Error::custom(format!(
                "RlpNode data too long: {} bytes (max {})",
                bytes.len(),
                MAX
            ))
        })?;
        Ok(RlpNode(arr))
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    // Helper struct to test serialization
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestWrapper {
        node: RlpNode,
    }

    #[test]
    fn test_rlp_node_serde_compiles() {
        // This test just verifies that our serde implementation compiles correctly
        // We can't test the actual JSON output without serde_json
        
        // Test with empty node
        let empty = RlpNode::default();
        let _wrapper = TestWrapper { node: empty };

        // Test with some data
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let node = RlpNode::from_raw(&data).unwrap();
        let _wrapper = TestWrapper { node };

        // Test with max size data (33 bytes)
        let max_data = vec![0xff; 33];
        let max_node = RlpNode::from_raw(&max_data).unwrap();
        let _wrapper = TestWrapper { node: max_node };
    }
}
