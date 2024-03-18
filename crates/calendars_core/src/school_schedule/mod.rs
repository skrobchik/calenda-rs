use std::collections::BTreeMap;

use chrono::{Datelike, Days, TimeZone, Timelike, Utc};
use egui::Color32;

use crate::week_calendar;
use icalendar::{Component, EventLike};

use serde::{Deserialize, Serialize};

pub mod simulation_types;
pub use simulation_types::*;

pub mod class_calendar;
pub mod metadata_types;
pub use metadata_types::*;

use crate::{class_filter::ClassFilter, week_calendar::WeekCalendar};

use self::class_calendar::{ClassCalendar, ClassId};

#[derive(thiserror::Error, Debug)]
#[error("Class hours in calendars do not match.")]
pub struct ClassHourCountNotMatchingError {}

#[derive(Debug)]
pub struct ClassEntry<'a> {
  school_schedule: &'a mut SchoolSchedule,
  class_id: ClassId,
}

impl<'a> ClassEntry<'a> {
  pub fn set_hours(&mut self, class_hours: u8) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(usize::from(self.class_id))
      .unwrap();
    let curr_class_hours = class.class_hours;
    match class_hours.cmp(&curr_class_hours) {
      std::cmp::Ordering::Equal => {}
      std::cmp::Ordering::Less => {
        let negative_delta = curr_class_hours - class_hours;
        for _ in 0..negative_delta {
          self
            .school_schedule
            .class_calendar
            .remove_one_class_anywhere(self.class_id);
        }
        class.class_hours = class_hours;
      }
      std::cmp::Ordering::Greater => {
        let positive_delta = class_hours - curr_class_hours;
        for _ in 0..positive_delta {
          self.school_schedule.class_calendar.add_one_class(
            0.try_into().unwrap(),
            0.try_into().unwrap(),
            self.class_id,
          );
        }
        class.class_hours = class_hours;
      }
    };
  }

  pub fn set_professor_id(&mut self, professor_id: usize) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(usize::from(self.class_id))
      .unwrap();
    class.professor_id = professor_id;
  }

  pub fn set_group(&mut self, group: Group) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(usize::from(self.class_id))
      .unwrap();
    class.group = group;
  }

  pub fn set_semester(&mut self, semester: Semester) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(usize::from(self.class_id))
      .unwrap();
    class.semester = semester;
  }

  pub fn set_classroom_type(&mut self, classroom_type: ClassroomType) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(usize::from(self.class_id))
      .unwrap();
    class.classroom_type = classroom_type;
  }

  pub fn set_optative(&mut self, optative: bool) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(usize::from(self.class_id))
      .unwrap();
    class.optative = optative;
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassroomAssignmentKey {
  pub day: week_calendar::Day,
  pub timeslot: week_calendar::Timeslot,
  pub class_id: ClassId,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SchoolSchedule {
  metadata: ScheduleMetadata,
  simulation_constraints: SimulationConstraints,
  class_calendar: ClassCalendar,
  classroom_assignments: BTreeMap<ClassroomAssignmentKey, Classroom>,
}

impl SchoolSchedule {
  pub fn get_simulation_constraints(&self) -> &SimulationConstraints {
    &self.simulation_constraints
  }

  pub fn get_class(&self, class_id: ClassId) -> Option<&Class> {
    self
      .simulation_constraints
      .classes
      .get(usize::from(class_id))
  }

  pub fn get_class_entry_mut(&mut self, class_id: ClassId) -> Option<ClassEntry> {
    Some(ClassEntry {
      school_schedule: self,
      class_id,
    })
  }

  pub fn get_class_metadata(&self, class_id: ClassId) -> Option<&ClassMetadata> {
    self.metadata.classes.get(usize::from(class_id))
  }

  pub fn get_class_metadata_mut(&mut self, class_id: ClassId) -> Option<&mut ClassMetadata> {
    self.metadata.classes.get_mut(usize::from(class_id))
  }

  pub fn get_professor_mut(&mut self, professor_id: usize) -> Option<&mut Professor> {
    self.simulation_constraints.professors.get_mut(professor_id)
  }

  pub fn get_professor_metadata(&self, professor_id: usize) -> Option<&ProfessorMetadata> {
    self.metadata.professors.get(professor_id)
  }

  pub fn get_professor_metadata_mut(
    &mut self,
    professor_id: usize,
  ) -> Option<&mut ProfessorMetadata> {
    self.metadata.professors.get_mut(professor_id)
  }

  pub fn get_num_classes(&self) -> usize {
    assert_eq!(
      self.simulation_constraints.classes.len(),
      self.metadata.classes.len()
    );
    self.simulation_constraints.classes.len()
  }

  pub fn get_num_professors(&self) -> usize {
    assert_eq!(
      self.simulation_constraints.professors.len(),
      self.metadata.professors.len()
    );
    self.simulation_constraints.professors.len()
  }

  pub fn add_new_professor(&mut self) -> usize {
    let professor_metadata = &mut self.metadata.professors;
    let professors = &mut self.simulation_constraints.professors;

    professor_metadata.push(ProfessorMetadata {
      name: "New Professor".to_string(),
    });

    professors.push(Professor {
      availability: WeekCalendar::default(),
      priority: 0.0,
    });

    assert_eq!(professors.len(), professor_metadata.len());

    professors.len() - 1
  }

  pub fn add_new_class(&mut self) -> ClassId {
    let class_metadata_list: &mut Vec<ClassMetadata> = &mut self.metadata.classes;
    let class_list = &mut self.simulation_constraints.classes;

    class_metadata_list.push(ClassMetadata {
      name: "New Class".to_string(),
      color: Color32::LIGHT_YELLOW,
      class_code: "0000".to_string(),
    });

    class_list.push(Class {
      professor_id: 0,
      classroom_type: ClassroomType::AulaSimple,
      class_hours: 2,
      semester: Semester::S1,
      group: Group::G1,
      optative: false,
    });
    let class_id: ClassId = (class_list.len() - 1).try_into().unwrap();
    self.class_calendar.add_one_class(
      4.try_into().unwrap(),
      week_calendar::TIMESLOT_18_00.try_into().unwrap(),
      class_id,
    );
    self.class_calendar.add_one_class(
      4.try_into().unwrap(),
      week_calendar::TIMESLOT_19_00.try_into().unwrap(),
      class_id,
    );

    assert_eq!(
      self.class_calendar.get_count(
        4.try_into().unwrap(),
        week_calendar::TIMESLOT_18_00.try_into().unwrap(),
        class_id
      ) + self.class_calendar.get_count(
        4.try_into().unwrap(),
        week_calendar::TIMESLOT_19_00.try_into().unwrap(),
        class_id
      ),
      class_list[usize::from(class_id)].class_hours
    );
    assert_eq!(class_list.len(), class_metadata_list.len());
    class_id
  }

  pub fn get_class_calendar(&self) -> &ClassCalendar {
    &self.class_calendar
  }

  pub fn replace_class_calendar(
    &mut self,
    class_calendar: ClassCalendar,
  ) -> Result<(), ClassHourCountNotMatchingError> {
    let current_class_hour_count = count_class_hours(&self.class_calendar);
    let class_hour_count = count_class_hours(&class_calendar);
    if current_class_hour_count != class_hour_count {
      return Err(ClassHourCountNotMatchingError {});
    }
    self.class_calendar = class_calendar;
    Ok(())
  }

  pub fn export_ics(&self, class_filter: &ClassFilter) -> icalendar::Calendar {
    // let school_timezone = chrono_tz::Mexico::BajaNorte;
    let school_timezone = chrono_tz::Europe::Dublin;
    let semester_start = school_timezone
      .with_ymd_and_hms(2022, 8, 8, 0, 0, 0)
      .unwrap();
    let _semester_end = school_timezone
      .with_ymd_and_hms(2023, 5, 27, 0, 0, 0)
      .unwrap();
    assert_eq!(semester_start.weekday().num_days_from_monday(), 0);
    assert_eq!(semester_start.hour(), 0);
    assert_eq!(semester_start.minute(), 0);
    assert_eq!(semester_start.second(), 0);

    let mut cal = icalendar::Calendar::new();
    struct ClassRange {
      class_id: ClassId,
      day: week_calendar::Day,
      start_timeslot: week_calendar::Timeslot,
      /// inclusive
      end_timeslot: week_calendar::Timeslot,
    }
    let mut class_ranges: Vec<ClassRange> = Vec::new();
    let mut first = true;
    for class_entry in self.class_calendar.get_entries().iter().filter(|entry| {
      let b = class_filter.filter(
        entry.class_id,
        &self.simulation_constraints,
        &self.class_calendar,
        entry.day_idx,
        entry.timeslot_idx,
        first,
      );
      first = false;
      b
    }) {
      let new_range = ClassRange {
        class_id: class_entry.class_id,
        day: class_entry.day_idx,
        start_timeslot: class_entry.timeslot_idx,
        end_timeslot: class_entry.timeslot_idx,
      };
      if let Some(prev_range) = class_ranges.iter_mut().find(|r| {
        r.class_id == new_range.class_id
          && r.day == new_range.day
          && usize::from(r.end_timeslot).checked_add(1_usize).map_or(
            false,
            |prev_range_end_timeslot_plus_one| {
              prev_range_end_timeslot_plus_one == usize::from(new_range.start_timeslot)
            },
          )
      }) {
        prev_range.end_timeslot = new_range.end_timeslot;
      } else {
        class_ranges.push(new_range);
      }
    }
    for class_range in class_ranges {
      let mut event = icalendar::Event::new();
      let start_time = semester_start
        .checked_add_days(Days::new(usize::from(class_range.day) as u64))
        .unwrap()
        .with_hour(crate::week_calendar::timeslot_to_hour(
          class_range.start_timeslot,
        ))
        .unwrap()
        .with_timezone(&Utc);
      let end_time = semester_start
        .checked_add_days(Days::new(usize::from(class_range.day) as u64))
        .unwrap()
        .with_hour(crate::week_calendar::timeslot_to_hour(class_range.end_timeslot) + 1) // +1 because end_timeslot is inclusive
        .unwrap()
        .with_timezone(&Utc);
      event.starts(start_time);
      event.ends(end_time);
      event.summary(&self.get_class_metadata(class_range.class_id).unwrap().name);

      cal.push(event);
    }
    cal
  }
}

fn count_class_hours(class_calendar: &ClassCalendar) -> Vec<u8> {
  let mut class_hour_count: Vec<u8> = Vec::new();
  for timeslot in class_calendar.iter_timeslots() {
    for (class_id, count) in timeslot.iter().enumerate() {
      if *count == 0 {
        continue;
      }
      if class_id >= class_hour_count.len() {
        class_hour_count.resize(class_id + 1, 0);
      }
      class_hour_count[class_id] += count;
    }
  }
  class_hour_count
}
