pub fn calc_time(length: f64, mut our_time: f64) -> f64 {
    if length >= 14.0 {
        if length <= 20.0 {
            our_time *= 0.90 + (length - 15.0) * 0.45;
        } else if length <= 24.0 {
            our_time *= 0.90 + (20.0 - 15.0) * 0.45;
        } else if length <= 28.0 {
            our_time *= 0.80 + (28.0 - length) * 0.25;
        } else if length <= 70.0 {
            our_time *= 0.10 + 5.00 / (length - 20.0);
        } else {
            our_time *= 0.02 + (150.0 - length) * 0.0002;
        }
    } else {
        our_time *= 0.06 + 2.00 / (-length + 18.0);
    }
    our_time * 0.935
}
