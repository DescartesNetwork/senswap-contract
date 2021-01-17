const PRECISION: u64 = 1000000000; // 10**9
const DOUBLE_PRECISION: u64 = 1000000000000000000; // 10**18

pub struct UMath {
  pub i: u64,
  pub f: u64,
}

impl UMath {
  pub fn new(i: u64, f: u64) -> Option<UMath> {
    if f > PRECISION {
      return None;
    }
    Some(UMath { i, f })
  }

  pub fn add(&self, n: &UMath) -> Option<UMath> {
    let f = (self.f as u128).checked_add(n.f as u128)?;
    let df = f.checked_div(PRECISION as u128)? as u64;
    let rf = f.checked_rem(PRECISION as u128)? as u64;
    let i = self.i.checked_add(n.i)?.checked_add(df)?;
    Self::new(i, rf)
  }

  pub fn sub(&self, n: &UMath) -> Option<UMath> {
    if self.i < n.i {
      return None;
    }
    if self.i == n.i && self.f > n.f {
      return None;
    }

    if self.f < n.f {
      let i = self.i.checked_sub(n.i)?.checked_sub(1)?;
      let f = self.f.checked_add(PRECISION)?.checked_sub(n.f)?;
      return Self::new(i, f);
    }
    let i = self.i.checked_sub(n.i)?;
    let f = (self.f).checked_sub(n.f)?;
    Self::new(i, f)
  }

  pub fn mul(&self, n: &UMath) -> Option<UMath> {
    let f = (self.f as u128)
      .checked_mul(n.f as u128)?
      .checked_div(PRECISION as u128)?;
    let if1 = (self.f as u128).checked_mul(n.i as u128)?;
    let if2 = (n.f as u128).checked_mul(self.i as u128)?;

    let df = f.checked_div(PRECISION as u128)?;
    let rf = f.checked_rem(PRECISION as u128)?;
    let dif1 = if1.checked_div(PRECISION as u128)?;
    let rif1 = if1.checked_rem(PRECISION as u128)?;
    let dif2 = if2.checked_div(PRECISION as u128)?;
    let rif2 = if2.checked_rem(PRECISION as u128)?;

    let d_rf_rf1_rf2 = rf
      .checked_add(rif1)?
      .checked_add(rif2)?
      .checked_div(PRECISION as u128)?;
    let r_rf_rf1_rf2 = rf
      .checked_add(rif1)?
      .checked_add(rif2)?
      .checked_rem(PRECISION as u128)?;

    let f = r_rf_rf1_rf2 as u64;
    let i = (self.i as u128)
      .checked_mul(n.i as u128)?
      .checked_add(df)?
      .checked_add(dif1)?
      .checked_add(dif2)?
      .checked_add(d_rf_rf1_rf2)? as u64;

    Self::new(i, f)
  }

  pub fn div(&self, n: &UMath) -> Option<UMath> {
    let big_self = (self.i as u128)
      .checked_mul(PRECISION as u128)?
      .checked_add(self.f as u128)?;
    let big_n = (n.i as u128)
      .checked_mul(PRECISION as u128)?
      .checked_add(n.f as u128)?;
    let i = big_self.checked_div(big_n)? as u64;
    let f = big_self
      .checked_rem(big_n)?
      .checked_mul(PRECISION as u128)?
      .checked_div(big_n)? as u64;
    Self::new(i, f)
  }

  pub fn sqr(&self) -> Option<UMath> {
    self.mul(&self)
  }

  pub fn sqrt(&self) -> Option<UMath> {
    let big_self = (self.i as u128)
      .checked_mul(DOUBLE_PRECISION as u128)?
      .checked_add(self.f as u128)?;
    let sqrt_big_self = Self::_sqrt(&big_self)?;
    let i = sqrt_big_self.checked_div(PRECISION as u128)? as u64;
    let f = sqrt_big_self.checked_rem(PRECISION as u128)? as u64;
    Self::new(i, f)
  }

  fn _sqrt(n: &u128) -> Option<u128> {
    let mut start: u128 = 0;
    let mut end: u128 = *n;
    let mut sqrt_n: u128 = 0;
    while start <= end {
      let mid = start.checked_add(end)?.checked_div(2)?;
      // mid is too big
      if let None = mid.checked_pow(2) {
        end = mid.checked_sub(1)?;
        continue;
      }
      // ok to compute
      if mid.checked_pow(2)? == *n {
        return Some(mid);
      }
      if mid.checked_pow(2)? < *n {
        start = mid.checked_add(1)?;
        sqrt_n = mid;
      } else {
        end = mid.checked_sub(1)?;
      }
    }
    Some(sqrt_n)
  }
}
