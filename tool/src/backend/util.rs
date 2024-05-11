use std::str::FromStr;

/**
range_str: &str in form u32-32 or u32 (`11-34` or `11`)
*/
pub fn is_in_range(range_str: &str, val: u32) -> bool {
    let range: Vec<_> = range_str.split("-").collect();

    if range.len() > 1 {
        let Ok(min) = u32::from_str(&range[0]) else {
            return false
        };

        let Ok(max) = u32::from_str(&range[1]) else {
            return false
        };

        min <= val && val <= max
    } else if range.len() == 1 {
        let Ok(min) = u32::from_str(&range[0]) else {
            return false
        };

        min <= val
    } else {
        false
    }
}