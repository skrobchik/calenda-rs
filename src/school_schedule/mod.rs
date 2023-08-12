use std::ops::{Index, IndexMut};

use egui::Color32;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub(crate) mod simulation_types;
pub(crate) use simulation_types::*;

pub(crate) mod metadata_types;
pub(crate) use metadata_types::*;

use crate::{
  timeslot::{DAY_RANGE, TIMESLOT_RANGE},
  week_calendar::{WeekCalendar, Weekday},
};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(Default)]
pub(crate) struct TimeslotClassHours {
  data: Vec<u8>,
}

impl TimeslotClassHours {
  pub(crate) fn iter(&self) -> std::slice::Iter<'_, u8> {
    self.data.iter()
  }

  pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, u8> {
    self.data.iter_mut()
  }

  pub(crate) fn len(&self) -> usize {
    self.data.len()
  }
}



impl Index<usize> for TimeslotClassHours {
  type Output = u8;

  fn index(&self, index: usize) -> &Self::Output {
    if index >= self.data.len() {
      return &0;
    }
    &self.data[index]
  }
}

impl IndexMut<usize> for TimeslotClassHours {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    if index >= self.data.len() {
      self.data.resize(index + 1, 0);
    }
    &mut self.data[index]
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct SchoolSchedule {
  metadata: ScheduleMetadata,
  pub(crate) simulation_information: SimulationConstraints,
  pub(crate) schedule: WeekCalendar<TimeslotClassHours>,
}

pub(crate) struct ClassData<'a> {
  pub(crate) count: u8,
  pub(crate) class_id: usize,
  pub(crate) class: &'a Class,
  pub(crate) class_metadata: &'a ClassMetadata,
}

impl SchoolSchedule {
  pub(crate) fn get_class_data(&self, day: Weekday, timeslot: usize) -> Vec<ClassData<'_>> {
    let slot = self.schedule.get(day, timeslot).unwrap();
    let classes = &self.simulation_information.classes;
    let class_metadata = &self.metadata.classes;
    slot
      .iter()
      .enumerate()
      .filter(|(_class_id, count)| **count > 0)
      .map(|(class_id, count)| ClassData {
        count: *count,
        class: &classes[class_id],
        class_metadata: &class_metadata[class_id],
        class_id,
      })
      .collect_vec()
  }

  pub(crate) fn get_classes(&self) -> Vec<(&Class, &ClassMetadata)> {
    self
      .simulation_information
      .classes
      .iter()
      .zip(&self.metadata.classes)
      .collect()
  }

  pub(crate) fn get_class(&self, class_id: usize) -> Option<&Class> {
    self.simulation_information.classes.get(class_id)
  }

  pub(crate) fn get_class_metadata(&self, class_id: usize) -> Option<&ClassMetadata> {
    self.metadata.classes.get(class_id)
  }

  pub(crate) fn get_class_metadata_mut(&mut self, class_id: usize) -> Option<&mut ClassMetadata> {
    self.metadata.classes.get_mut(class_id)
  }

  pub(crate) fn get_class_classroom_type_mut(&mut self, class_id: usize) -> Option<&mut ClassroomType> {
    self.simulation_information.classes.get_mut(class_id).map(|class| &mut class.classroom_type)
  }

  pub(crate) fn get_class_class_hours(&self, class_id: usize) -> Option<u8> {
    self.simulation_information.classes.get(class_id).map(|class| class.class_hours)
  }

  pub(crate) fn set_class_class_hours(&mut self, class_id: usize, class_hours: u8) -> Option<()> {
    self.simulation_information.classes.get_mut(class_id).map(|class| {
      class.class_hours = class_hours;

      // TODO: Update Calendar (might be desynced)
    })
  }

  pub(crate) fn get_num_classes(&self) -> usize {
    assert_eq!(self.simulation_information.classes.len(), self.metadata.classes.len());
    self.simulation_information.classes.len()
  }

  pub(crate) fn get_num_professors(&self) -> usize {
    assert_eq!(self.simulation_information.professors.len(), self.metadata.professors.len());
    self.simulation_information.professors.len()
  }

  fn add_hours_to_schedule(&mut self, class_id: usize, count: u8) {
    self.schedule.get_mut(1_usize, 1_usize).unwrap()[class_id] =
      self.schedule.get(1_usize, 1_usize).unwrap()[class_id]
        .checked_add(count)
        .unwrap();
  }

  /// Attempts to remove `count` instances of class with `class_id` from schedule.
  /// Returns the amount that was left to remove.
  /// If `count` instances were removed succesfully, return value is 0.
  /// If there are not enough classes to remove, return the amount that was left to remove.
  fn remove_hours_from_schedule(&mut self, class_id: usize, count: u8) -> u8 {
    let mut count = count;
    for day in DAY_RANGE {
      for timeslot in TIMESLOT_RANGE {
        let dc = count.min(self.schedule.get(day, timeslot).unwrap()[class_id]);
        count -= dc;
        self.schedule.get_mut(day, timeslot).unwrap()[class_id] -= dc;
        if count == 0 {
          return count;
        }
      }
    }
    count
  }

  pub(crate) fn next_class_id(&self) -> usize {
    self
      .schedule
      .data
      .iter()
      .map(|timeslot_class_hours| timeslot_class_hours.len())
      .max()
      .unwrap_or_default()
      .max(self.simulation_information.classes.len())
  }

  pub(crate) fn fill_classes(&mut self) {
    let mut schedule_hour_count = TimeslotClassHours::default();
    for day in DAY_RANGE {
      for timeslot in TIMESLOT_RANGE {
        let class_hours = self.schedule.get(day, timeslot).unwrap();
        for (class_id, count) in class_hours.iter().enumerate() {
          schedule_hour_count[class_id] += *count;
        }
      }
    }
    let mut classes_hour_count = TimeslotClassHours::default();
    for (class_id, class) in self.simulation_information.classes.iter().enumerate() {
      classes_hour_count[class_id] = class.class_hours;
    }

    let max_class_id = self.next_class_id();

    for class_id in 0..max_class_id {
      let schedule_hours = schedule_hour_count[class_id];
      let class_hours = classes_hour_count[class_id];
      match Ord::cmp(&schedule_hours, &class_hours) {
        std::cmp::Ordering::Less => {
          println!(
            "Deficit of {} classes with id {} in schedule",
            class_hours - schedule_hours,
            class_id
          );
          self.add_hours_to_schedule(class_id, class_hours - schedule_hours);
        }
        std::cmp::Ordering::Equal => (),
        std::cmp::Ordering::Greater => {
          println!(
            "Excess of {} classes with id {} in schedule",
            schedule_hours - class_hours,
            class_id
          );
          self.remove_hours_from_schedule(class_id, schedule_hours - class_hours);
        }
      }
    }
  }

  pub(crate) fn add_new_professor(&mut self) -> (&mut Professor, &mut ProfessorMetadata, usize) {
    let professor_metadata = &mut self.metadata.professors;
    let professors = &mut self.simulation_information.professors;

    professor_metadata.push(ProfessorMetadata {
      name: "New Professor".to_string(),
    });

    professors.push(Professor {
      availability: WeekCalendar::default(),
    });

    assert_eq!(professors.len(), professor_metadata.len());

    let professor_id = professors.len() - 1;

    (
      professors.get_mut(professor_id).unwrap(),
      professor_metadata.get_mut(professor_id).unwrap(),
      professor_id,
    )
  }

  pub(crate) fn add_new_class(&mut self) -> (&mut Class, &mut ClassMetadata) {
    let class_metadata: &mut Vec<ClassMetadata> = &mut self.metadata.classes;
    let classes = &mut self.simulation_information.classes;

    class_metadata.push(ClassMetadata {
      name: "New Class".to_string(),
      color: Color32::LIGHT_YELLOW,
    });

    classes.push(Class {
      professor_id: 0,
      classroom_type: ClassroomType::Single,
      class_hours: 1,
      semester: Semester::S1,
      group: Group::G1,
    });

    assert_eq!(classes.len(), class_metadata.len());
    let class_id = classes.len() - 1;

    (
      classes.get_mut(class_id).unwrap(),
      class_metadata.get_mut(class_id).unwrap(),
    )
  }

  pub(crate) fn get_professor_metadata_mut(&mut self, professor_id: usize) -> Option<&mut ProfessorMetadata> {
    self.metadata.professors.get_mut(professor_id)
  }

  pub(crate) fn get_professor_metadata(&mut self, professor_id: usize) -> Option<&ProfessorMetadata> {
    self.metadata.professors.get(professor_id)
  }
}
