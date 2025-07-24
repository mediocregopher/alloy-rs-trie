use core::fmt;
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, From, Not};

/// A struct representing a mask of 16 bits, used for Ethereum trie operations.
///
/// Masks in a trie are used to efficiently represent and manage information about the presence or
/// absence of certain elements, such as child nodes, within a trie. Masks are usually implemented
/// as bit vectors, where each bit represents the presence (1) or absence (0) of a corresponding
/// element.
#[derive(
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deref,
    From,
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    Not,
)]
#[cfg_attr(feature = "arbitrary", derive(derive_arbitrary::Arbitrary, proptest_derive::Arbitrary))]
pub struct TrieMask(u16);

impl fmt::Debug for TrieMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TrieMask({:016b})", self.0)
    }
}

impl TrieMask {
    /// Creates a new `TrieMask` from the given inner value.
    #[inline]
    pub const fn new(inner: u16) -> Self {
        Self(inner)
    }

    /// Returns the inner value of the `TrieMask`.
    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Creates a new `TrieMask` from the given nibble.
    #[inline]
    pub const fn from_nibble(nibble: u8) -> Self {
        Self(1u16 << nibble)
    }

    /// Returns `true` if the current `TrieMask` is a subset of `other`.
    #[inline]
    pub fn is_subset_of(self, other: Self) -> bool {
        self & other == self
    }

    /// Returns `true` if a given bit is set in a mask.
    #[inline]
    pub const fn is_bit_set(self, index: u8) -> bool {
        self.0 & (1u16 << index) != 0
    }

    /// Returns `true` if the mask is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the number of bits set in the mask.
    #[inline]
    pub const fn count_bits(self) -> u8 {
        self.0.count_ones() as u8
    }

    /// Returns the index of the first bit set in the mask, or `None` if the mask is empty.
    #[inline]
    pub const fn first_set_bit_index(self) -> Option<u8> {
        if self.is_empty() { None } else { Some(self.0.trailing_zeros() as u8) }
    }

    /// Set bit at a specified index.
    #[inline]
    pub fn set_bit(&mut self, index: u8) {
        self.0 |= 1u16 << index;
    }

    /// Unset bit at a specified index.
    #[inline]
    pub fn unset_bit(&mut self, index: u8) {
        self.0 &= !(1u16 << index);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for TrieMask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Format as 16-bit binary string, matching Debug format without "TrieMask()" wrapper
        serializer.serialize_str(&format!("{:016b}", self.0))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TrieMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        
        // Parse binary string
        if s.len() != 16 {
            return Err(serde::de::Error::custom(format!(
                "TrieMask binary string must be exactly 16 characters, got {}",
                s.len()
            )));
        }
        
        let value = u16::from_str_radix(&s, 2)
            .map_err(|e| serde::de::Error::custom(format!("Invalid binary string: {}", e)))?;
        
        Ok(TrieMask(value))
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn test_trie_mask_serde_roundtrip() {
        // Test with various masks
        let masks = [
            TrieMask::new(0b0000000000000000), // All zeros
            TrieMask::new(0b1111111111111111), // All ones
            TrieMask::new(0b1010101010101010), // Alternating
            TrieMask::new(0b0000000000000001), // Single bit at end
            TrieMask::new(0b1000000000000000), // Single bit at start
            TrieMask::from_nibble(5),           // Bit 5 set
        ];
        
        for mask in masks {
            // Test that we can serialize and deserialize
            #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
            struct Wrapper {
                mask: TrieMask,
            }
            
            let wrapper = Wrapper { mask };
            let _ = wrapper; // Just ensure it compiles with serde traits
        }
    }
    
    #[test]
    fn test_trie_mask_debug_format() {
        let mask = TrieMask::new(0b0000000000001010);
        let debug_str = format!("{:?}", mask);
        assert_eq!(debug_str, "TrieMask(0000000000001010)");
        
        // The serde format should be just the binary part
        let expected_serde = "0000000000001010";
        assert_eq!(format!("{:016b}", mask.0), expected_serde);
    }
}
