use blake3::Hasher;

pub type Hash32 = [u8; 32];

pub fn hash_bytes(data: &[u8]) -> Hash32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

pub fn hash_pair(left: &Hash32, right: &Hash32) -> Hash32 {
    let mut hasher = Hasher::new();
    hasher.update(left);
    hasher.update(right);
    *hasher.finalize().as_bytes()
}

pub fn zero_hashes(height: usize) -> Vec<Hash32> {
    let mut zeros = Vec::with_capacity(height + 1);
    // Convention: zero leaf is hash_bytes([0;32]).
    zeros.push(hash_bytes(&[0u8; 32]));
    for i in 0..height {
        let z = zeros[i];
        zeros.push(hash_pair(&z, &z));
    }
    zeros
}

pub fn hash32_to_hex(h: &Hash32) -> String {
    hex::encode(h)
}

pub fn hash32_from_hex(s: &str) -> Result<Hash32, hex::FromHexError> {
    let bytes = hex::decode(s)?;
    Ok(bytes
        .as_slice()
        .try_into()
        .map_err(|_| hex::FromHexError::InvalidStringLength)?)
}
