use calendars_core::{
  Availability, Day, Group, SchoolSchedule, Semester, TIMESLOT_09_00, TIMESLOT_17_00,
};

use egui::Color32;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::fmt::Write;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tracing::trace;

fn preprocess_sql_file(contents: String) -> String {
  contents
    .lines()
    .filter(|line| {
      let ignore = ["ALTER", "ADD", "MODIFY", "SET", "--", "/*"];
      for pattern in ignore {
        if line.trim().starts_with(pattern) || line.trim().is_empty() {
          return false;
        }
      }
      true
    })
    .fold(String::new(), |mut output, line| {
      if line.contains("ENGINE=InnoDB") {
        let _ = writeln!(output, ");");
      } else {
        let _ = writeln!(output, "{line}");
      }
      output
    })
}

fn import_temporary_database<P1: AsRef<Path>, P2: AsRef<Path>>(
  materias_sql_path: P1,
  profesores_sql_path: P2,
) -> anyhow::Result<sqlite::Connection> {
  let temp_db_file = tempfile::NamedTempFile::new()?;
  let temp_db_path = temp_db_file.into_temp_path();
  let temp_db_path = temp_db_path.keep()?;
  let connection = sqlite::open(temp_db_path)?;
  let query_profesores = preprocess_sql_file(fs::read_to_string(profesores_sql_path)?);
  let query_materias = preprocess_sql_file(fs::read_to_string(materias_sql_path)?);
  connection.execute(query_materias)?;
  connection.execute(query_profesores)?;
  Ok(connection)
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Class {
  name: String,
  rfc1: String,
  rfc2: String,
  ciclo: String,
  grupo: String,
  asignatura: String,
}

#[derive(PartialEq, Eq, Hash)]
struct Professor {
  name: String,
  rfc: String,
}

fn is_optative(class_code: &str) -> bool {
  class_code.starts_with("00")
}

pub fn parse_semester_group(s: &str) -> Option<(Semester, Group)> {
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

fn parse_database_data(connection: sqlite::Connection) -> anyhow::Result<SchoolSchedule> {
  let mut schedule: SchoolSchedule = Default::default();
  let query = "SELECT * FROM Materias";

  let mut classes: Vec<Class> = Vec::new();
  connection.iterate(query, |rows| {
    for row in rows.iter().tuple_windows() {
      trace!("{:?}", row);
      let (
        (_, _id),
        (_, grupo),
        (_, asignatura),
        (_, descripcion),
        (_, rfc1),
        (_, rfc2),
        (_, ciclo),
        (_, _especial),
      ) = row;
      classes.push(Class {
        name: descripcion.unwrap().to_string(),
        rfc1: rfc1.unwrap().to_string(),
        rfc2: rfc2.unwrap().to_string(),
        ciclo: ciclo.unwrap().to_string(),
        grupo: grupo.unwrap().to_string(),
        asignatura: asignatura.unwrap().to_string(),
      });
    }
    true
  })?;

  let classes: Vec<Class> = classes
  .iter()
  .filter(|c| c.ciclo == "2022-2")
  .filter(|c| {
    // c.grupo.starts_with("01")
    //   || c.grupo.starts_with("03")
    //   || c.grupo.starts_with("05")
    //   || c.grupo.starts_with("07")
    // c.grupo.starts_with("02")
         c.grupo.starts_with("04")
      // || c.grupo.starts_with("06")
      // || c.grupo.starts_with("08")
  })
  .cloned()
  .collect();

  let mut rfc_used: BTreeSet<String> = BTreeSet::new();
  for class in classes.iter() {
    rfc_used.insert(class.rfc1.clone());
    rfc_used.insert(class.rfc2.clone());
  }

  let mut professors: Vec<Professor> = Vec::new();

  let query = "SELECT * FROM Profesores";
  connection.iterate(query, |rows| {
    for ((_, nombre), (_, rfc), (_, _usuario)) in rows.iter().tuple_windows() {
      trace!("{}", nombre.unwrap());
      professors.push(Professor {
        name: nombre.unwrap().to_string(),
        rfc: rfc.unwrap().to_string(),
      });
    }
    true
  })?;

  let mut professor_ids: BTreeMap<String, usize> = BTreeMap::new();

  for my_professor in professors.iter().unique() {
    let rfc = my_professor.rfc.clone();
    if !rfc_used.contains(&rfc) {
      continue;
    }
    let professor_id = schedule.add_new_professor();
    let professor_metadata = schedule.get_professor_metadata_mut(professor_id).unwrap();
    professor_metadata.name = my_professor.name.clone();
    let professor = schedule.get_professor_mut(professor_id).unwrap();
    professor_ids.insert(rfc, professor_id);
    for day in Day::all() {
      for timeslot in TIMESLOT_09_00..TIMESLOT_17_00 {
        *professor
          .availability
          .get_mut(day, timeslot.try_into().unwrap()) = Availability::AvailableIfNeeded;
      }
    }
  }


  println!("Imported {} classes", classes.len());

  let num_classes = classes.len();
  // let colors_iterator = itertools_num::linspace(0.0, 1.0, num_classes).map(|x| {
  //   let color = ecolor::Hsva::new(x, 1.0, 1.0, 1.0);
  //   let color = Color32::from(color);
  //   color
  // });
  let colors_iterator = crate::color_list::COLOR_LIST.iter().cycle().map(|s| {
    let color = csscolorparser::parse(s).unwrap();
    let color = color.to_rgba8();

    Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
  });

  for (my_class, color) in classes.iter().take(num_classes).zip(colors_iterator) {
    let class_id = schedule.add_new_class();
    schedule.get_class_metadata_mut(class_id).unwrap().color = color;
    schedule.get_class_metadata_mut(class_id).unwrap().name = my_class.name.to_string();
    let professor_id = professor_ids.get(&my_class.rfc1).unwrap();
    let mut class_entry = schedule.get_class_entry_mut(class_id).unwrap();
    class_entry.set_professor_id(*professor_id);
    if let Some((semester, group)) = parse_semester_group(&my_class.grupo) {
      class_entry.set_group(group);
      class_entry.set_semester(semester);
      class_entry.set_optative(is_optative(&my_class.asignatura));
      schedule
        .get_class_metadata_mut(class_id)
        .unwrap()
        .class_code = my_class.asignatura.clone();
    } else {
      println!("ERRRRRROOOOOR");
    }

    if my_class.rfc2.trim().is_empty() {
      continue;
    }

    let class_id = schedule.add_new_class();
    schedule.get_class_metadata_mut(class_id).unwrap().color = color;
    schedule.get_class_metadata_mut(class_id).unwrap().name = format!("{} (Lab)", my_class.name);
    let professor_id = professor_ids.get(&my_class.rfc2).unwrap();
    let mut class_entry = schedule.get_class_entry_mut(class_id).unwrap();
    class_entry.set_professor_id(*professor_id);
    if let Some((semester, group)) = parse_semester_group(&my_class.grupo) {
      class_entry.set_group(group);
      class_entry.set_semester(semester);
      class_entry.set_optative(is_optative(&my_class.asignatura));
      schedule
        .get_class_metadata_mut(class_id)
        .unwrap()
        .class_code = my_class.asignatura.clone();
    }
  }

  Ok(schedule)
}

pub struct ImportSchedulePaths<P1: AsRef<Path>, P2: AsRef<Path>> {
  pub materias_sql_path: P1,
  pub profesores_sql_path: P2,
}

impl Default for ImportSchedulePaths<PathBuf, PathBuf> {
  fn default() -> Self {
    Self {
      materias_sql_path: PathBuf::from_iter(&["Archivos SQL", "Materias.sql"]),
      profesores_sql_path: PathBuf::from_iter(&["Archivos SQL", "Profesores.sql"]),
    }
  }
}

pub fn import_schedule<P1: AsRef<Path>, P2: AsRef<Path>>(
  paths: ImportSchedulePaths<P1, P2>,
) -> anyhow::Result<SchoolSchedule> {
  let connection = import_temporary_database(paths.materias_sql_path, paths.profesores_sql_path)?;
  parse_database_data(connection)
}
