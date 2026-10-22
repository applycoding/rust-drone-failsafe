use crate::types::HALF_RANGE_MS;

/// Age in milliseconds from `then` to `now`, or `None` if the sample is in the future
/// (wrapping diff >= 2^31 ms).
pub fn age_ms(now: u32, then: u32) -> Option<u32> {
    let diff = now.wrapping_sub(then);
    if diff >= HALF_RANGE_MS {
        None
    } else {
        Some(diff)
    }
}

pub fn is_fresh(now: u32, t_ms: u32, limit_ms: u32) -> bool {
    match age_ms(now, t_ms) {
        Some(age) => age <= limit_ms,
        None => false,
    }
}
