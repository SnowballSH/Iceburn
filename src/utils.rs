use lazy_static::lazy_static;

use std::sync::{Mutex, MutexGuard};

static mut SEED: u64 = 1493682623;

pub fn pseudo_random() -> u64 {
    let mut n = unsafe { SEED };
    n ^= n << 13;
    n ^= n >> 17;
    n ^= n << 5;
    unsafe {
        SEED = n;
    }
    n
}

fn gen_sqchart() -> [[u8; 2]; 128] {
    let mut res = [[0u8; 2]; 128];
    for a in 0..16 {
        for b in 0..8 {
            res[b * 16 + a] = [('a' as usize + a) as u8, ('8' as usize - b) as u8];
        }
    }
    res
}

#[inline]
pub fn extract_mutex<T>(m: &Mutex<T>) -> MutexGuard<T> {
    m.lock().unwrap()
}

lazy_static! {
    pub static ref SQUARE_CHART: [[u8; 2]; 128] = gen_sqchart();
}
