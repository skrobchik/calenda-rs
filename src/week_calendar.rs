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

pub fn weekday_index(day: &Weekday) -> usize {
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

impl TryFrom<usize> for Weekday {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Weekday::Monday),
            1 => Ok(Weekday::Tuesday),
            2 => Ok(Weekday::Wednesday),
            3 => Ok(Weekday::Thursday),
            4 => Ok(Weekday::Friday),
            5 => Ok(Weekday::Saturday),
            6 => Ok(Weekday::Sunday),
            _ => Err(()),
        }
    }
}

pub trait GetDay<T> {
    fn get_day(&self, day: &Weekday) -> &[T; TIMESLOT_COUNT];
}

impl<T> GetDay<T> for WeekCalendar<T> {
    fn get_day(&self, day: &Weekday) -> &[T; TIMESLOT_COUNT] {
        self.get(weekday_index(day)).unwrap()
    }
}
