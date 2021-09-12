static mut SEED: u32 = 1493682623;

pub fn pseudo_random() -> u32 {
    let mut n = unsafe { SEED };
    n ^= n << 13;
    n ^= n >> 17;
    n ^= n << 5;
    unsafe {
        SEED = n;
    }
    n
}
