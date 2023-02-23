use crate::timeslots::*;

pub const DAY_COUNT: usize = 7; // 7 days in a week

pub type WeekCalendar<T> = [[T; TIMESLOT_COUNT]; DAY_COUNT];

#[derive(Clone, Copy)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}

pub fn weekday_index(day: Weekday) -> usize {
    match day {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    }
}

pub fn get_day<T: Copy>(week_calendar: &WeekCalendar<T>, day: Weekday) -> [T; TIMESLOT_COUNT] {
    week_calendar[weekday_index(day)]
}