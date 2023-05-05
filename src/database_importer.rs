use crate::school_schedule::SchoolSchedule;
use anyhow::Context;
use itertools::Itertools;
use std::{fs, path::Path};
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
  name: String
}

#[derive(PartialEq, Eq, Hash)]
struct Professor {
  name: String
}

pub fn parse_database_data() -> anyhow::Result<SchoolSchedule> {
  let connection = sqlite::open(TEMP_DB_PATH)?;
  let mut schedule: SchoolSchedule = Default::default();
  let query = "SELECT * FROM Materias";
  
  let mut classes: Vec<Class> = Vec::new();
  connection.iterate(query, |rows| {
    for (a, b, c, (_, descripcion), d, e, f, g) in rows.iter().tuple_windows() {
      trace!("{:?} {:?} {:?} {:?} {:?} {:?} {:?}", a, b, c, d, e, f, g);
      trace!("{}", descripcion.unwrap());
      classes.push(Class {
        name: descripcion.unwrap().to_string(), 
      });
    }
    true
  })?;

  for my_class in classes.iter().unique() {
    let (_class, mut class_metadata) = schedule.add_new_class().context("no more space").unwrap();
    class_metadata.name = my_class.name.clone();
  }

  let mut professors: Vec<Professor> = Vec::new();

  let query = "SELECT * FROM Profesores";
  connection.iterate(query, |rows| {
    for ((_, nombre), _, _) in rows.iter().tuple_windows() {
      trace!("{}", nombre.unwrap());
      professors.push(Professor { name: nombre.unwrap().to_string() });
    }
    true
  })?;

  for my_professor in professors.iter().unique() {
    let (_professor, mut professor_metadata, _professor_id) = schedule.add_new_professor().context("no more space").unwrap();
    professor_metadata.name = my_professor.name.clone();
  }
  

  Ok(schedule)
}
