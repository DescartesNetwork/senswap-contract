use solana_program::pubkey::Pubkey;

pub trait Boolean {
  fn xor(&self, pk: &Pubkey) -> Self;
}

impl Boolean for Pubkey {
  fn xor(&self, pk: &Pubkey) -> Self {
    let a: [u8; 32] = self.to_bytes();
    let b: [u8; 32] = pk.to_bytes();
    let c: [u8; 32] = a.zip(b).map(|(x, y)| x ^ y);
    return Pubkey::new_from_array(c);
  }
}
