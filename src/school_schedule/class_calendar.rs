use anyhow;

use crate::timeslot;

pub(crate) struct ClassCalendar {

}

pub struct GroupedClassEntry {

}

impl GroupedClassEntry {
    fn get_class_id() -> usize {
        todo!()
    }

    fn get_day() -> timeslot::Day {
        todo!()
    }

    fn get_timeslot() -> timeslot::Timeslot {
        todo!()
    }

    fn get_count() -> timeslot::Timeslot {
        todo!()
    }
}

pub struct  SingleClassEntry {
    
}

impl SingleClassEntry {
    fn get_class_id() -> usize {
        todo!()
    }

    fn get_day() -> timeslot::Day {
        todo!()
    }

    fn get_timeslot() -> timeslot::Timeslot {
        todo!()
    }
}

impl ClassCalendar {
    pub(crate) fn get_count(day: timeslot::Day, timeslot: timeslot::Timeslot, class_id: usize) -> u8 {
        todo!()
    }

    pub(crate) fn move_class(source_day: timeslot::Day, source_timeslot: timeslot::Timeslot, target_day: timeslot::Day, target_timeslot: timeslot::Timeslot, class_id: usize, count: u8) -> anyhow::Result<()> {
        todo!();
    }

    pub(super) fn add_class(day: timeslot::Day, timeslot: timeslot::Timeslot, class_id: usize, count: u8) {
        todo!();
    }

    pub(super) fn remove_class(day: timeslot::Day, timeslot: timeslot::Timeslot, class_id: usize, count: u8) {
        todo!();
    }

    pub(crate) fn get_single_class_entries(&self) -> Vec<&SingleClassEntry> {
        todo!();
    }

    pub(crate) fn get_grouped_class_entries(&self) -> Vec<&GroupedClassEntry> {
        todo!();
    }
}

