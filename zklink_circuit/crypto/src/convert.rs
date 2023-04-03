use crate::franklin_crypto::bellman::pairing::ff::{PrimeField, PrimeFieldRepr, BitIterator};

/// Extension trait denoting common conversion method for field elements.
pub trait FeConvert: PrimeField {
    /// Converts the field element into a byte array.
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        self.into_repr().write_be(&mut buf).unwrap();

        buf
    }

    /// Reads a field element from its byte sequence representation.
    fn from_bytes(value: &[u8]) -> Result<Self, anyhow::Error> {
        let mut repr = Self::Repr::default();

        // `repr.as_ref()` converts `repr` to a list of `u64`. Each element has 8 bytes,
        // so to obtain size in bytes, we multiply the array size with the size of `u64`.
        let expected_input_size = repr.as_ref().len() * 8;
        if value.len() != expected_input_size {
            anyhow::bail!("Incorrect input size")
        }
        repr.read_be(value)
            .map_err(|e| anyhow::format_err!("Cannot parse value {:?}: {}", value, e))?;
        Self::from_repr(repr).map_err(|e| {
            anyhow::format_err!("Cannot convert into prime field value {:?}: {}", value, e)
        })
    }

    /// Returns hex representation of the field element without `0x` prefix.
    fn to_hex(&self) -> String {
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        self.into_repr().write_be(&mut buf).unwrap();
        hex::encode(&buf)
    }

    /// Reads a field element from its hexadecimal representation.
    fn from_hex(value: &str) -> Result<Self, anyhow::Error> {
        let value = if let Some(value) = value.strip_prefix("0x") {
            value
        } else {
            value
        };

        // Buffer is reversed and read as little endian, since we pad it with zeros to
        // match the expected length.
        let mut buf = hex::decode(&value)
            .map_err(|e| anyhow::format_err!("could not decode hex: {}, reason: {}", value, e))?;
        buf.reverse();
        let mut repr = Self::Repr::default();

        // `repr.as_ref()` converts `repr` to a list of `u64`. Each element has 8 bytes,
        // so to obtain size in bytes, we multiply the array size with the size of `u64`.
        buf.resize(repr.as_ref().len() * 8, 0);
        repr.read_le(&buf[..])
            .map_err(|e| anyhow::format_err!("could not read {}: {}", value, e))?;
        Self::from_repr(repr).map_err(|e| {
            anyhow::format_err!("could not convert into prime field: {}: {}", value, e)
        })
    }

    fn into_big_uint(self) -> num::BigUint{
        let mut reader = Vec::with_capacity(32);
        self.into_repr().write_be(&mut reader).unwrap();
        num::BigUint::from_bytes_be(&reader)
    }

    fn into_usize(self) -> usize{
        use std::convert::TryInto;
        let mut reader = Vec::with_capacity(32);
        self.into_repr().write_be(&mut reader).unwrap();
        reader.reverse();

        let (usize_reader, _) = reader.split_at(std::mem::size_of::<usize>());
        usize::from_le_bytes(usize_reader.try_into().unwrap())
    }

    fn get_be_fixed_bits(&self, width: usize) -> Vec<bool>{
        let mut bits: Vec<bool> = BitIterator::new(self.into_repr()).collect();
        bits.reverse();
        bits.resize(width, false);
        bits.reverse();
        bits
    }

    fn from_big_uint(uint: num::BigUint) -> Option<Self>{
        let mut vec_big = uint.to_bytes_le();
        let mut fr_repr = Self::Repr::default();
        vec_big.resize(fr_repr.as_ref().len() * 8, 0);
        fr_repr.read_le(&vec_big[..]).ok()?;
        Self::from_repr(fr_repr).ok()
    }

    fn from_u64(num: u64) -> Self {
       Self::from_repr(num.into()).unwrap()
    }
}

impl<T> FeConvert for T where T: PrimeField {}

#[cfg(test)]
mod tests {
    use num::BigUint;
    use franklin_crypto::bellman::Field;
    use super::*;

    use crate::rand::{Rand, SeedableRng, XorShiftRng};
    use crate::Fr;

    /// Checks that converting FE to the hex form and back results
    /// in the same FE.
    #[test]
    fn fe_hex_roundtrip() {
        let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);

        let fr = Fr::rand(&mut rng);

        let encoded_fr = fr.to_hex();
        let decoded_fr = Fr::from_hex(&encoded_fr).expect("Can't decode encoded fr");

        assert_eq!(fr, decoded_fr);
    }

    /// Checks that converting FE to the bytes form and back results
    /// in the same FE.
    #[test]
    fn fe_bytes_roundtrip() {
        let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);

        let fr = Fr::rand(&mut rng);

        let encoded_fr = fr.to_bytes();
        let decoded_fr = Fr::from_bytes(&encoded_fr).expect("Can't decode encoded fr");

        assert_eq!(fr, decoded_fr);
    }


    #[test]
    fn test_fr(){
        let f = Fr::from_repr(18u64.into()).unwrap();
        println!("{}", f);

        let b = BigUint::from(1000000u128);
        let mut vec_big = b.to_bytes_le();
        println!("b:{:x}",b);
        let mut fr_repr = Fr::zero().into_repr();
        vec_big.resize(fr_repr.as_ref().len() * 8, 0);
        fr_repr.read_le(&*vec_big).expect("pack bigUint as field element");
        let b_fr = Fr::from_repr(fr_repr).unwrap();
        println!("{}", b_fr);

        use std::convert::TryInto;
        let mut reader = Vec::with_capacity(32);
        b_fr.into_repr().write_be(&mut reader).unwrap();
        reader.reverse();
        let (reader, _) = reader.split_at(std::mem::size_of::<usize>());
        println!("{}",usize::from_le_bytes(reader.try_into().unwrap()))
    }
}
