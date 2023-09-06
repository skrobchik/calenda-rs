use itertools::Itertools;

use crate::timeslot::{self, DAY_RANGE, TIMESLOT_RANGE};
use serde::Serialize;
use serde::Deserialize;

pub const MAX_CLASS_ID: usize = 256;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
struct SingleClassEntry {
    day_idx: timeslot::Day,
    timeslot_idx: timeslot::Timeslot,
    class_id: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClassEntryDelta {
    pub class_id: usize,
    pub src_day_idx: timeslot::Day,
    pub src_timeslot_idx: timeslot::Timeslot,
    pub dst_day_idx: timeslot::Day,
    pub dst_timeslot_idx: timeslot::Timeslot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClassCalendar {
    matrix: Vec<Vec<u8>>,
    class_entries: Vec<SingleClassEntry>,
}

impl ClassCalendar {
    pub(crate) fn new() -> Self {
        Self {
            matrix: vec![Vec::new(); timeslot::TIMESLOT_COUNT * timeslot::DAY_COUNT],
            class_entries: Vec::new(),
        }
    }

    pub(crate) fn get_matrix(&self) -> &Vec<Vec<u8>> {
        &self.matrix
    }

    pub(crate) fn get_timeslot(&self, day_idx: timeslot::Day, timeslot_idx: timeslot::Timeslot) -> &Vec<u8> {
        &self.matrix[day_idx * timeslot::DAY_COUNT + timeslot_idx]
    }

    fn get_timeslot_mut(&mut self, day_idx: timeslot::Day, timeslot_idx: timeslot::Timeslot) -> &mut Vec<u8> {
        &mut self.matrix[day_idx * timeslot::DAY_COUNT + timeslot_idx]
    }

    pub(crate) fn get_count(&self, day_idx: timeslot::Day, timeslot_idx: timeslot::Timeslot, class_id: usize) -> u8 {
        assert!(timeslot::DAY_RANGE.contains(&day_idx));
        assert!(timeslot::TIMESLOT_RANGE.contains(&timeslot_idx));
        assert!(class_id <= MAX_CLASS_ID);
        let timeslot = self.get_timeslot(day_idx, timeslot_idx);
        timeslot.get(class_id).map(|x| *x).unwrap_or(0_u8)
    }

    pub(crate) fn move_one_class_random<R: rand::Rng>(&mut self, rng: &mut R) -> ClassEntryDelta {
        let entry_idx = rng.gen_range(0..self.class_entries.len());
        let entry = self.class_entries.get_mut(entry_idx).unwrap();
        let class_id = entry.class_id;
        let src_day_idx = entry.day_idx;
        let src_timeslot_idx = entry.timeslot_idx;
        let dst_day_idx = rng.gen_range(timeslot::DAY_RANGE);
        let dst_timeslot_idx = rng.gen_range(timeslot::TIMESLOT_RANGE);
        entry.day_idx = dst_day_idx;
        entry.timeslot_idx = dst_timeslot_idx;
        {
            let src_timeslot = self.get_timeslot_mut(src_day_idx, src_timeslot_idx);
            assert!(src_timeslot[class_id] >= 1);
            src_timeslot[class_id] -= 1;
        }
        {
            let dst_timeslot = self.get_timeslot_mut(dst_day_idx, dst_timeslot_idx);
            if class_id >= dst_timeslot.len() {
                dst_timeslot.resize(class_id+1, 0);
            }
            dst_timeslot[class_id] = dst_timeslot[class_id].checked_add(1).unwrap();
        }
        ClassEntryDelta { class_id, src_day_idx, src_timeslot_idx, dst_day_idx, dst_timeslot_idx }
    }

    pub(crate) fn move_one_class(&mut self, source_day_idx: timeslot::Day, source_timeslot_idx: timeslot::Timeslot, target_day_idx: timeslot::Day, target_timeslot_idx: timeslot::Timeslot, class_id: usize) {
        self.remove_one_class(source_day_idx, source_timeslot_idx, class_id);
        self.add_one_class(target_day_idx, target_timeslot_idx, class_id);
    }

    pub(crate) fn add_one_class(&mut self, day_idx: timeslot::Day, timeslot_idx: timeslot::Timeslot, class_id: usize) {
        assert!(timeslot::DAY_RANGE.contains(&day_idx));
        assert!(timeslot::TIMESLOT_RANGE.contains(&timeslot_idx));
        assert!(class_id <= MAX_CLASS_ID);
        let timeslot = self.get_timeslot_mut(day_idx, timeslot_idx);
        if class_id >= timeslot.len() {
            timeslot.resize(class_id+1, 0_u8);
        }
        timeslot[class_id] = timeslot[class_id].checked_add(1).unwrap();
        self.class_entries.push(SingleClassEntry { day_idx, timeslot_idx, class_id });
    }


    pub(crate) fn remove_one_class(&mut self, day_idx: timeslot::Day, timeslot_idx: timeslot::Timeslot, class_id: usize) {
        assert!(class_id <= MAX_CLASS_ID);
        assert!(DAY_RANGE.contains(&day_idx));
        assert!(TIMESLOT_RANGE.contains(&timeslot_idx));
        
        let timeslot = self.get_timeslot_mut(day_idx, timeslot_idx);
        assert!(timeslot.len() > class_id);
        
        timeslot[class_id] = timeslot[class_id].checked_sub(1).unwrap();

        let entry_idx = self.class_entries.iter().find_position(|x| **x == SingleClassEntry {
            day_idx,
            timeslot_idx,
            class_id,
        }).unwrap().0;
        self.class_entries.swap_remove(entry_idx);
    }

    pub(super) fn remove_one_class_anywhere(&mut self, class_id: usize) {
        let (entry_idx, entry) = self.class_entries.iter().find_position(|e| e.class_id == class_id).unwrap();

        let timeslot_idx = entry.timeslot_idx;
        let day_idx = entry.day_idx;

        let timeslot = self.get_timeslot_mut(day_idx, timeslot_idx);

        timeslot[class_id] = timeslot[class_id].checked_sub(1).unwrap();

        self.class_entries.swap_remove(entry_idx);
    }

    /// O(1)
    pub(crate) fn get_total_class_count(&self) -> usize {
        self.class_entries.len()
    }
}

mod test {
    #[test]
    fn class_calendar_test(){
        let mut class_calendar = super::ClassCalendar::new();
        class_calendar.add_one_class(0, 1, 4);
        class_calendar.add_one_class(0, 1, 4);
        assert_eq!(class_calendar.get_count(0, 1, 3), 0);
        assert_eq!(class_calendar.get_count(0, 1, 4), 2);
        assert_eq!(class_calendar.get_count(0, 1, 5), 0);
        class_calendar.remove_one_class_anywhere(4);
        assert_eq!(class_calendar.get_count(0, 1, 4), 1);
        class_calendar.add_one_class(0, 1, 4);
        class_calendar.add_one_class(0, 1, 4);
        class_calendar.add_one_class(0, 1, 4);
        class_calendar.add_one_class(0, 1, 4);
        class_calendar.add_one_class(0, 1, 4);
        assert_eq!(class_calendar.get_count(0, 1, 4), 6);
        class_calendar.remove_one_class_anywhere(4);
        class_calendar.remove_one_class_anywhere(4);
        class_calendar.remove_one_class_anywhere(4);
        class_calendar.remove_one_class_anywhere(4);
        assert_eq!(class_calendar.get_count(0, 1, 4), 2);
        class_calendar.remove_one_class(0, 1, 4);
        assert_eq!(class_calendar.get_count(0, 1, 4), 1);
        class_calendar.remove_one_class_anywhere(4);
        assert_eq!(class_calendar.get_count(0, 1, 4), 0);
    }
}

impl  Default for ClassCalendar {
    fn default() -> Self {
        Self::new()
    }
}