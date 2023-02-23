pub const TIMESLOT_COUNT: usize = 24*4; // divide day in 15-minute time slots
pub const TIMESLOT_MINUTE_LENGTH: usize = 15;

pub fn timeslot_start_hour(timeslot: usize) -> u32 {
    (timeslot / 4) as u32    
}

pub fn timeslot_start_minute(timeslot: usize) -> u32 {
    match timeslot % 4 {
        0 => 0,
        1 => 15,
        2 => 30,
        3 => 45,
        _ => unreachable!()
    }
}
