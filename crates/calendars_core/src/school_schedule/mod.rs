use std::collections::BTreeMap;

use chrono::{Datelike, Days, TimeZone, Timelike, Utc};
use slotmap::SecondaryMap;

use crate::{
  week_calendar, AllowedClassroomTypes, Class, ClassCalendar, ClassKey, Classroom, Day, Group,
  OptimizationConstraints, Professor, ProfessorKey, Semester, SingleClassEntry, Timeslot,
};
use icalendar::{Component, EventLike};
mod metadata_types;
use metadata_types::{ClassMetadata, ProfessorMetadata, ScheduleMetadata};

use serde::{Deserialize, Serialize};

use crate::week_calendar::WeekCalendar;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ClassFilter {
  #[default]
  Any,
  None,
  Professor(ProfessorKey),
  Classroom(Classroom),
  Semester(Semester),
}

#[derive(thiserror::Error, Debug)]
#[error("Class hours in calendars do not match.")]
pub struct ClassHourCountNotMatchingError {}

#[derive(Debug)]
pub struct ClassEntry<'a> {
  school_schedule: &'a mut SchoolSchedule,
  class_key: ClassKey,
}

impl<'a, 'b> ClassEntry<'a> {
  fn get_class(&'b mut self) -> &'b mut Class {
    self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(self.class_key)
      .unwrap()
  }

  pub fn set_hours(&mut self, class_hours: u8) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(self.class_key)
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
            .remove_one_class_anywhere(self.class_key)
            .unwrap();
        }
        class.class_hours = class_hours;
      }
      std::cmp::Ordering::Greater => {
        let positive_delta = class_hours - curr_class_hours;
        for _ in 0..positive_delta {
          self
            .school_schedule
            .class_calendar
            .add_one_class(
              Day::from_usize(0).unwrap(),
              Timeslot::from_usize(0).unwrap(),
              self.class_key,
            )
            .unwrap();
        }
        class.class_hours = class_hours;
      }
    };
  }

  pub fn set_professor_id(&mut self, professor_key: ProfessorKey) {
    let class = self.get_class();
    class.professor_key = professor_key;
  }

  pub fn set_group(&mut self, group: Group) {
    let class = self.get_class();
    class.group = group;
  }

  pub fn set_semester(&mut self, semester: Semester) {
    let class = self.get_class();
    class.semester = semester;
  }

  pub fn set_allowed_classroom_types(&mut self, allowed_classroom_types: AllowedClassroomTypes) {
    let class = self.get_class();
    class.allowed_classroom_types = allowed_classroom_types;
  }

  pub fn set_optative(&mut self, optative: bool) {
    let class = self.get_class();
    class.optative = optative;
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassroomAssignmentKey {
  pub day: week_calendar::Day,
  pub timeslot: week_calendar::Timeslot,
  pub class_key: ClassKey,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SchoolSchedule {
  metadata: ScheduleMetadata,
  simulation_constraints: OptimizationConstraints,
  class_calendar: ClassCalendar,
  classroom_assignments: BTreeMap<ClassroomAssignmentKey, Classroom>,
}

impl SchoolSchedule {
  pub fn get_simulation_constraints(&self) -> &OptimizationConstraints {
    &self.simulation_constraints
  }

  pub fn get_class(&self, class_key: ClassKey) -> Option<&Class> {
    self.simulation_constraints.classes.get(class_key)
  }

  pub fn get_class_entry(&mut self, class_key: ClassKey) -> Option<ClassEntry> {
    Some(ClassEntry {
      school_schedule: self,
      class_key,
    })
  }

  pub fn get_class_metadata(&self, class_key: ClassKey) -> Option<&ClassMetadata> {
    self.metadata.classes.get(class_key)
  }

  pub fn get_class_metadata_mut(&mut self, class_key: ClassKey) -> Option<&mut ClassMetadata> {
    self.metadata.classes.get_mut(class_key)
  }

  pub fn get_professor_mut(&mut self, professor_key: ProfessorKey) -> Option<&mut Professor> {
    self
      .simulation_constraints
      .professors
      .get_mut(professor_key)
  }

  pub fn get_professor_metadata(&self, professor_key: ProfessorKey) -> Option<&ProfessorMetadata> {
    self.metadata.professors.get(professor_key)
  }

  pub fn get_professor_metadata_mut(
    &mut self,
    professor_key: ProfessorKey,
  ) -> Option<&mut ProfessorMetadata> {
    self.metadata.professors.get_mut(professor_key)
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

  pub fn add_new_professor(&mut self) -> ProfessorKey {
    let professor_metadata = &mut self.metadata.professors;
    let professors = &mut self.simulation_constraints.professors;
    let professor_key = professors.insert(Professor {
      availability: WeekCalendar::default(),
      priority: 0.0,
    });
    professor_metadata.insert(
      professor_key,
      ProfessorMetadata {
        name: "Nuevo Profesor".to_string(),
      },
    );
    professor_key
  }

  pub fn add_new_class(&mut self, professor_key: ProfessorKey) -> ClassKey {
    let class_key = self
      .simulation_constraints
      .classes
      .insert(Default::default());
    self
      .simulation_constraints
      .classes
      .get_mut(class_key)
      .unwrap()
      .professor_key = professor_key;

    let class_metadata_list = &mut self.metadata.classes;

    class_metadata_list.insert(
      class_key,
      ClassMetadata {
        name: "New Class".to_string(),
        rgba: [255, 255, 224, 255], // Light Yellow
        class_code: "0000".to_string(),
      },
    );

    class_key
  }

  pub fn class_calendar(&self) -> &ClassCalendar {
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

  pub fn filter_class(&self, class_entry: &SingleClassEntry, class_filter: &ClassFilter) -> bool {
    match class_filter {
      ClassFilter::Professor(professor_key) => {
        *professor_key == self.get_class(class_entry.class_key).unwrap().professor_key
      }
      ClassFilter::Classroom(classroom) => {
        todo!()
      }
      ClassFilter::Semester(semester) => {
        *semester == self.get_class(class_entry.class_key).unwrap().semester
      }
      ClassFilter::Any => true,
      ClassFilter::None => false,
    }
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
      class_key: ClassKey,
      day: week_calendar::Day,
      start_timeslot: week_calendar::Timeslot,
      /// inclusive
      end_timeslot: week_calendar::Timeslot,
    }
    let mut class_ranges: Vec<ClassRange> = Vec::new();
    for class_entry in self
      .class_calendar
      .class_entries()
      .iter()
      .filter(|class_entry| self.filter_class(class_entry, class_filter))
    {
      let new_range = ClassRange {
        class_key: class_entry.class_key,
        day: class_entry.day,
        start_timeslot: class_entry.timeslot,
        end_timeslot: class_entry.timeslot,
      };
      if let Some(prev_range) = class_ranges.iter_mut().find(|r| {
        r.class_key == new_range.class_key
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
        .with_hour(8 + u32::try_from(usize::from(class_range.start_timeslot)).unwrap())
        .unwrap()
        .with_timezone(&Utc);
      let end_time = semester_start
        .checked_add_days(Days::new(usize::from(class_range.day) as u64))
        .unwrap()
        .with_hour(8 + u32::try_from(usize::from(class_range.end_timeslot)).unwrap()) // +1 because end_timeslot is inclusive
        .unwrap()
        .with_timezone(&Utc);
      event.starts(start_time);
      event.ends(end_time);
      event.summary(&self.get_class_metadata(class_range.class_key).unwrap().name);

      cal.push(event);
    }
    cal
  }
}

fn count_class_hours(class_calendar: &ClassCalendar) -> SecondaryMap<ClassKey, u32> {
  let mut class_hour_count = SecondaryMap::new();
  for class_key in class_calendar.iter_class_keys() {
    let mut sum: u32 = 0;
    for day in week_calendar::Day::all() {
      for timeslot in week_calendar::Timeslot::all() {
        sum += class_calendar.get_count(day, timeslot, class_key) as u32;
      }
    }
    class_hour_count.insert(class_key, sum);
  }
  class_hour_count
}
