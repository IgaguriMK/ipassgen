use std::collections::BTreeSet;
use std::io::BufRead;

use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;

use crate::err::Error;

const ENTROPY_ESTIMATE_SAMPLES: usize = 100_000;
const ENTROPY_SAFETY_MERGIN_RATIO: f64 = 1.05;

pub struct Symbols {
    list: Vec<String>,
}

impl Symbols {
    pub fn from_iter(iter: impl Iterator<Item = String>) -> Symbols {
        let mut set: BTreeSet<String> = iter.collect();
        set.remove("");
        Symbols {
            list: set.into_iter().collect(),
        }
    }

    pub fn from_vec(list: Vec<String>) -> Symbols {
        Symbols::from_iter(list.into_iter())
    }

    pub fn from_chars(chars: impl Iterator<Item = char>) -> Symbols {
        Symbols::from_iter(chars.map(|c| {
            let mut s = String::new();
            s.push(c);
            s
        }))
    }

    pub fn from_bufread<R: BufRead>(r: R) -> Result<Symbols, Error> {
        let mut list = Vec::new();

        for l in r.lines() {
            list.push(l?);
        }

        Ok(Symbols::from_vec(list))
    }

    /**********************/
    pub fn generate(&self, n: usize, sep: &str, max_bytes: usize) -> Result<String, Error> {
        loop {
            let mut res = String::new();

            for i in 0..n {
                if i > 0 {
                    res.push_str(sep);
                }
                let s = self.list.choose(&mut OsRng).unwrap();
                res.push_str(&s);
            }

            if res.len() <= max_bytes {
                return Ok(res);
            }
        }
    }
    pub fn base_entropy(&self, n: usize) -> f64 {
        if self.list.is_empty() {
            return 0.0;
        }
        (n as f64) * (self.list.len() as f64).log2()
    }

    pub fn estimate_entropy(
        &self,
        n: usize,
        sep_len: usize,
        max_bytes: usize,
    ) -> Result<f64, Error> {
        let base_entropy = self.base_entropy(n);
        if base_entropy == 0.0 {
            return Ok(0.0);
        }

        let max_symbol_len = self.max_len();
        let total_sep_len = (n - 1) * sep_len;
        let max_len = n * max_symbol_len + total_sep_len;

        if max_len < max_bytes {
            return Ok(base_entropy);
        }

        let lengthes: Vec<usize> = self.list.iter().map(String::len).collect();
        let mut rng = XorShiftRng::from_rng(OsRng)?;
        let mut success = 0usize;
        for _ in 0..ENTROPY_ESTIMATE_SAMPLES {
            let mut l = total_sep_len;
            for _ in 0..n {
                l += lengthes.choose(&mut rng).unwrap();
            }

            if l <= max_bytes {
                success += 1;
            }
        }

        if success == 0 {
            return Ok(0.0);
        }

        let success_rate = (success as f64) / (ENTROPY_ESTIMATE_SAMPLES as f64);
        Ok((base_entropy + success_rate.log2() - ENTROPY_SAFETY_MERGIN_RATIO.log2()).max(0.0))
    }

    /**********************/

    fn max_len(&self) -> usize {
        self.list.iter().map(String::len).max().unwrap_or(0)
    }
}
