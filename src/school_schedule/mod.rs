use chrono::{Datelike, Days, TimeZone, Timelike, Utc};
use egui::Color32;

use icalendar::{Component, EventLike};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub(crate) mod simulation_types;
pub(crate) use simulation_types::*;

pub(crate) mod class_calendar;
pub(crate) mod metadata_types;
pub(crate) use metadata_types::*;

use crate::{
  timeslot::{self, DAY_RANGE, TIMESLOT_RANGE},
  week_calendar::WeekCalendar,
};

use self::class_calendar::ClassCalendar;

#[derive(thiserror::Error, Debug)]
#[error("Class hours in calendars do not match.")]
pub(crate) struct ClassHourCountNotMatchingError {}

pub(crate) fn parse_semester_group(s: &str) -> Option<(Semester, Group)> {
  match s.get(0..4).and_then(|s| s.chars().collect_tuple()) {
    Some(('0', c1, '0', c2)) => match (
      c1.to_digit(10).and_then(|d1| d1.try_into().ok()),
      c2.to_digit(10).and_then(|d2| d2.try_into().ok()),
    ) {
      (Some(semester), Some(group)) => Some((semester, group)),
      _ => None,
    },
    _ => None,
  }
}

#[derive(Debug)]
pub(crate) struct ClassEntry<'a> {
  school_schedule: &'a mut SchoolSchedule,
  class_id: usize,
}

impl<'a> ClassEntry<'a> {
  pub(crate) fn set_hours(&mut self, class_hours: u8) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(self.class_id)
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
          self
            .school_schedule
            .class_calendar
            .add_one_class(0, 0, self.class_id);
        }
        class.class_hours = class_hours;
      }
    };
  }

  pub(crate) fn set_professor_id(&mut self, professor_id: usize) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(self.class_id)
      .unwrap();
    class.professor_id = professor_id;
  }

  pub(crate) fn set_group(&mut self, group: Group) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(self.class_id)
      .unwrap();
    class.group = group;
  }

  pub(crate) fn set_semester(&mut self, semester: Semester) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(self.class_id)
      .unwrap();
    class.semester = semester;
  }

  pub(crate) fn set_classroom_type(&mut self, classroom_type: ClassroomType) {
    let class = self
      .school_schedule
      .simulation_constraints
      .classes
      .get_mut(self.class_id)
      .unwrap();
    class.classroom_type = classroom_type;
  }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct SchoolSchedule {
  metadata: ScheduleMetadata,
  simulation_constraints: SimulationConstraints,
  class_calendar: ClassCalendar,
}

impl SchoolSchedule {
  pub(crate) fn get_simulation_constraints(&self) -> &SimulationConstraints {
    &self.simulation_constraints
  }

  pub(crate) fn get_class(&self, class_id: usize) -> Option<&Class> {
    self.simulation_constraints.classes.get(class_id)
  }

  pub(crate) fn get_class_entry_mut(&mut self, class_id: usize) -> Option<ClassEntry> {
    Some(ClassEntry {
      school_schedule: self,
      class_id,
    })
  }

  pub(crate) fn get_class_metadata(&self, class_id: usize) -> Option<&ClassMetadata> {
    self.metadata.classes.get(class_id)
  }

  pub(crate) fn get_class_metadata_mut(&mut self, class_id: usize) -> Option<&mut ClassMetadata> {
    self.metadata.classes.get_mut(class_id)
  }

  pub(crate) fn get_professor(&self, professor_id: usize) -> Option<&Professor> {
    self.simulation_constraints.professors.get(professor_id)
  }

  pub(crate) fn get_professor_mut(&mut self, professor_id: usize) -> Option<&mut Professor> {
    self.simulation_constraints.professors.get_mut(professor_id)
  }

  pub(crate) fn get_professor_metadata(
    &mut self,
    professor_id: usize,
  ) -> Option<&ProfessorMetadata> {
    self.metadata.professors.get(professor_id)
  }

  pub(crate) fn get_professor_metadata_mut(
    &mut self,
    professor_id: usize,
  ) -> Option<&mut ProfessorMetadata> {
    self.metadata.professors.get_mut(professor_id)
  }

  pub(crate) fn get_num_classes(&self) -> usize {
    assert_eq!(
      self.simulation_constraints.classes.len(),
      self.metadata.classes.len()
    );
    self.simulation_constraints.classes.len()
  }

  pub(crate) fn get_num_professors(&self) -> usize {
    assert_eq!(
      self.simulation_constraints.professors.len(),
      self.metadata.professors.len()
    );
    self.simulation_constraints.professors.len()
  }

  pub(crate) fn add_new_professor(&mut self) -> usize {
    let professor_metadata = &mut self.metadata.professors;
    let professors = &mut self.simulation_constraints.professors;

    professor_metadata.push(ProfessorMetadata {
      name: "New Professor".to_string(),
    });

    professors.push(Professor {
      availability: WeekCalendar::default(),
    });

    assert_eq!(professors.len(), professor_metadata.len());

    professors.len() - 1
  }

  pub(crate) fn add_new_class(&mut self) -> usize {
    let class_metadata_list: &mut Vec<ClassMetadata> = &mut self.metadata.classes;
    let class_list = &mut self.simulation_constraints.classes;

    class_metadata_list.push(ClassMetadata {
      name: "New Class".to_string(),
      color: Color32::LIGHT_YELLOW,
    });

    class_list.push(Class {
      professor_id: 0,
      classroom_type: ClassroomType::Single,
      class_hours: 2,
      semester: Semester::S1,
      group: Group::G1,
    });
    let class_id = class_list.len() - 1;
    self
      .class_calendar
      .add_one_class(4, timeslot::TIMESLOT_18_00, class_id);
    self
      .class_calendar
      .add_one_class(4, timeslot::TIMESLOT_19_00, class_id);

    assert_eq!(
      self
        .class_calendar
        .get_count(4, timeslot::TIMESLOT_18_00, class_id)
        + self
          .class_calendar
          .get_count(4, timeslot::TIMESLOT_19_00, class_id),
      class_list[class_id].class_hours
    );
    assert_eq!(class_list.len(), class_metadata_list.len());
    class_id
  }

  pub(crate) fn get_class_calendar(&self) -> &ClassCalendar {
    &self.class_calendar
  }

  pub(crate) fn replace_class_calendar(
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

  pub(crate) fn export_ics<P: AsRef<std::path::Path>>(&self, export_path: P) {
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
      class_id: usize,
      day: usize,
      start_timeslot: usize,
      /// inclusive
      end_timeslot: usize,
    }
    let mut class_ranges: Vec<ClassRange> = Vec::new();
    for day in DAY_RANGE {
      for timeslot in TIMESLOT_RANGE {
        let classes = self.class_calendar.get_timeslot(day, timeslot);
        for (class_id, &count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
          for _ in 0..count {
            let new_range = ClassRange {
              class_id,
              day,
              start_timeslot: timeslot,
              end_timeslot: timeslot,
            };
            if let Some(prev_range) = class_ranges.iter_mut().find(|r| {
              r.class_id == new_range.class_id
                && r.day == new_range.day
                && r
                  .end_timeslot
                  .checked_add(1)
                  .map_or(false, |prev_range_end_timeslot_plus_one| {
                    prev_range_end_timeslot_plus_one == new_range.start_timeslot
                  })
            }) {
              prev_range.end_timeslot = new_range.end_timeslot;
            } else {
              class_ranges.push(new_range);
            }
          }
        }
      }
    }
    for class_range in class_ranges {
      let mut event = icalendar::Event::new();
      let start_time = semester_start
        .checked_add_days(Days::new(class_range.day as u64))
        .unwrap()
        .with_hour(crate::timeslot::timeslot_to_hour(
          class_range.start_timeslot,
        ))
        .unwrap()
        .with_timezone(&Utc);
      let end_time = semester_start
        .checked_add_days(Days::new(class_range.day as u64))
        .unwrap()
        .with_hour(crate::timeslot::timeslot_to_hour(class_range.end_timeslot) + 1) // +1 because end_timeslot is inclusive
        .unwrap()
        .with_timezone(&Utc);
      event.starts(start_time);
      event.ends(end_time);
      event.summary(&self.get_class_metadata(class_range.class_id).unwrap().name);

      cal.push(event);
    }
    std::fs::write(export_path, cal.to_string()).unwrap();
  }
}

fn count_class_hours(class_calendar: &ClassCalendar) -> Vec<u8> {
  let mut class_hour_count: Vec<u8> = Vec::new();
  let matrix = class_calendar.get_matrix();
  for timeslot in matrix.iter() {
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
