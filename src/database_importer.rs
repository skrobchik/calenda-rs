use crate::{school_schedule::{SchoolSchedule, parse_semester_group, Availability}, timeslot};
use anyhow::Context;
use itertools::Itertools;
use std::{collections::BTreeMap, fs, path::Path};
use tracing::trace;

const TEMP_DB_PATH: &str = "temp_db.sqlite";

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
    .map(|line| {
      if line.contains("ENGINE=InnoDB") {
        return ");\n".into();
      }
      format!("{line}\n")
    })
    .collect()
}

pub fn import_temporary_database() -> anyhow::Result<()> {
  let materias_sql_export_path = "Archivos SQL/Materias.sql";
  let profesores_sql_export_path = "Archivos SQL/Profesores.sql";
  if Path::new(TEMP_DB_PATH).exists() {
    fs::remove_file(TEMP_DB_PATH)?;
  }
  let connection = sqlite::open(TEMP_DB_PATH)?;
  let query_profesores = preprocess_sql_file(fs::read_to_string(profesores_sql_export_path)?);
  let query_materias = preprocess_sql_file(fs::read_to_string(materias_sql_export_path)?);
  connection.execute(query_materias)?;
  connection.execute(query_profesores)?;
  Ok(())
}

#[derive(PartialEq, Eq, Hash)]
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

pub fn parse_database_data() -> anyhow::Result<SchoolSchedule> {
  let connection = sqlite::open(TEMP_DB_PATH)?;
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
    let (professor, mut professor_metadata, professor_id) = schedule
      .add_new_professor()
      .context("no more space")
      .unwrap();
    professor_metadata.name = my_professor.name.clone();
    professor_ids.insert(my_professor.rfc.clone(), professor_id);
    for day in timeslot::DAY_RANGE {
      for timeslot in timeslot::TIMESLOT_09_00..timeslot::TIMESLOT_17_00 {
        *professor.availability.get_mut(day, timeslot).unwrap() = Availability::AvailableIfNeeded;
      }
    }
  }

  for my_class in classes.iter().filter(|c| c.ciclo == "2023-2").take(10) {
    let (class, mut class_metadata) = schedule.add_new_class().context("no more space").unwrap();
    class_metadata.name = format!("{} {}", my_class.asignatura, my_class.name);
    let professor_id = professor_ids.get(&my_class.rfc1).unwrap_or(&0);
    class.professor = *professor_id;
    if let Some((semester, group)) = parse_semester_group(&my_class.grupo) {
      class.group = group;
      class.semester = semester;
    }

    if my_class.rfc2.trim().is_empty() { continue; }

    let (class, mut class_metadata) = schedule.add_new_class().context("no more space").unwrap();
    class_metadata.name = format!("{} {} (Lab)", my_class.asignatura, my_class.name);
    let professor_id = professor_ids.get(&my_class.rfc2).unwrap_or(&0);
    class.professor = *professor_id;
    if let Some((semester, group)) = parse_semester_group(&my_class.grupo) {
      class.group = group;
      class.semester = semester;
    }
  }

  Ok(schedule)
}
