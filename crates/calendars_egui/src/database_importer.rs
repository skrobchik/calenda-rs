use calendars_core::{ClassroomType, Group, ProfessorKey, SchoolSchedule, Semester};

use anyhow::Context;
use egui::Color32;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::iter;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct ClassRow {
  descripcion: String,
  rfc1: String,
  rfc2: String,
  ciclo: String,
  grupo: String,
  asign: String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct ProfessorRow {
  nombre: String,
  rfc: String,
}

fn preprocess_sql(contents: &str) -> String {
  contents
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .filter(|line| {
      !["ALTER", "ADD", "MODIFY", "SET", "--", "/*"]
        .iter()
        .any(|pat| line.starts_with(pat))
    })
    .map(|line| {
      if line.contains("ENGINE=InnoDB") {
        ");"
      } else {
        line
      }
    })
    .interleave_shortest(iter::repeat("\n"))
    .collect()
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

fn get_professor_rows(
  connection: &rusqlite::Connection,
) -> Result<Vec<ProfessorRow>, rusqlite::Error> {
  let mut statement = connection.prepare("SELECT * FROM `Profesores`")?;
  let professor_rows = statement.query_map([], |row| {
    let nombre: String = row.get("nombre")?;
    let rfc: String = row.get("rfc")?;
    let _usuario: String = row.get("usuario")?;
    Ok(ProfessorRow { nombre, rfc })
  })?;
  professor_rows.collect()
}

fn get_class_rows(connection: &rusqlite::Connection) -> Result<Vec<ClassRow>, rusqlite::Error> {
  let mut statement = connection.prepare("SELECT * FROM Materias")?;
  let class_rows = statement.query_map([], |row| {
    let _id: i64 = row.get("id")?;
    let grupo: String = row.get("grupo")?;
    let asignatura: String = row.get("asign")?;
    let descripcion: String = row.get("descripcion")?;
    let rfc1: String = row.get("rfc1")?;
    let rfc2: String = row.get("rfc2")?;
    let ciclo: String = row.get("ciclo")?;
    let _especial: i64 = row.get("especial")?;
    Ok(ClassRow {
      descripcion,
      asign: asignatura,
      rfc1,
      rfc2,
      ciclo,
      grupo,
    })
  })?;
  class_rows.collect()
}

fn create_schedule(
  professor_rows: &[ProfessorRow],
  class_rows: &[ClassRow],
) -> anyhow::Result<SchoolSchedule> {
  let mut schedule = SchoolSchedule::default();
  let mut professors: BTreeMap<&str, ProfessorKey> = BTreeMap::new();
  for professor_row in professor_rows {
    let professor_id = schedule.add_new_professor();
    let professor_metadata = schedule.get_professor_metadata_mut(professor_id).unwrap();
    professor_metadata.name.clone_from(&professor_row.nombre);
    if professors
      .insert(professor_row.rfc.as_str(), professor_id)
      .is_some()
    {
      return Err(anyhow::format_err!("Duplicate RFC `{}`", professor_row.rfc));
    }
  }
  let colors_iterator = crate::color_list::COLOR_LIST.iter().cycle().map(|s| {
    let color = csscolorparser::parse(s).unwrap();
    let color = color.to_rgba8();

    Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
  });
  for (class_row, color) in class_rows.iter().zip(colors_iterator) {
    let has_lab = !class_row.rfc2.trim().is_empty();
    let theory_professor_key = *professors.get(class_row.rfc1.as_str()).context(format!(
      "Professor with RFC `{}` not found. Required by class `{} {}`",
      class_row.rfc1, class_row.asign, class_row.descripcion
    ))?;
    let lab_professor_id = if has_lab {
      Some(*professors.get(class_row.rfc2.as_str()).context(format!(
        "Lab Professor with RFC `{}` not found. Required by class `{} {}`",
        class_row.rfc2, class_row.asign, class_row.descripcion
      ))?)
    } else {
      None
    };
    let (semester, group) = parse_semester_group(&class_row.grupo).context(format!(
      "Couldn't parse semester and group: `{}` from `{}`",
      class_row.grupo, class_row.descripcion
    ))?;
    let is_optative = is_optative(&class_row.asign);
    let theory_class_key = schedule.add_new_class(theory_professor_key);
    let mut theory_class = schedule.get_class_entry(theory_class_key).unwrap();
    theory_class.set_semester(semester);
    theory_class.set_group(group);
    theory_class.set_optative(is_optative);
    theory_class.set_professor_id(theory_professor_key);
    theory_class.set_hours(2);
    theory_class.set_allowed_classroom_types(ClassroomType::AulaSimple | ClassroomType::AulaDoble);
    let theory_class_metadata = schedule.get_class_metadata_mut(theory_class_key).unwrap();
    theory_class_metadata.rgba = color.to_array();
    theory_class_metadata
      .name
      .clone_from(&class_row.descripcion);
    theory_class_metadata
      .class_code
      .clone_from(&class_row.asign);
    if let Some(lab_professor_key) = lab_professor_id {
      assert!(has_lab);
      let lab_class_key = schedule.add_new_class(lab_professor_key);
      let mut lab_class = schedule.get_class_entry(lab_class_key).unwrap();
      lab_class.set_semester(semester);
      lab_class.set_group(group);
      lab_class.set_optative(is_optative);
      lab_class.set_professor_id(lab_professor_key);
      lab_class.set_hours(3);
      lab_class.set_allowed_classroom_types(ClassroomType::LabFisica | ClassroomType::LabQuimica);
      let lab_class_metadata = schedule.get_class_metadata_mut(lab_class_key).unwrap();
      lab_class_metadata.rgba = color.to_array();
      lab_class_metadata.name = format!("{} (Lab)", class_row.descripcion);
      lab_class_metadata.class_code.clone_from(&class_row.asign);
    }
  }
  Ok(schedule)
}

fn filter_unused_professors(
  class_rows: &[ClassRow],
  professor_rows: Vec<ProfessorRow>,
) -> Vec<ProfessorRow> {
  let rfcs_used: BTreeSet<&str> = class_rows
    .iter()
    .flat_map(|c| [c.rfc1.as_str(), c.rfc2.as_str()])
    .collect();
  professor_rows
    .into_iter()
    .filter(|professor_row| rfcs_used.contains(professor_row.rfc.as_str()))
    .collect()
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
  let connection = rusqlite::Connection::open_in_memory()?;
  connection.execute_batch(&preprocess_sql(&fs::read_to_string(
    paths.profesores_sql_path,
  )?))?;
  connection.execute_batch(&preprocess_sql(&fs::read_to_string(
    paths.materias_sql_path,
  )?))?;
  let classes = get_class_rows(&connection)?;
  let classes: Vec<ClassRow> = classes
    .into_iter()
    .filter(|class_row| class_row.ciclo == "2022-2")
    .filter(|class_row| class_row.grupo.starts_with("02"))
    .filter(|class_row| class_row.descripcion != "CURSO TALLER DE DIDÁCTICA II")
    .collect();

  let professors = filter_unused_professors(&classes, get_professor_rows(&connection)?);

  create_schedule(&professors, &classes)
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_preprocess_sql() {
    let input = r###"
-- phpMyAdmin SQL Dump
-- version 4.5.2
-- http://www.phpmyadmin.net
--

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
SET time_zone = "+00:00";

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `nanoDB`
--

-- --------------------------------------------------------

--
-- Table structure for table `Profesores`
--

CREATE TABLE `Profesores` (
  `nombre` varchar(50) NOT NULL,
  `rfc` varchar(14) NOT NULL,
  `usuario` varchar(30) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

--
-- Dumping data for table `Profesores`
--

INSERT INTO `Profesores` (`nombre`, `rfc`, `usuario`) VALUES
('AAAA', 'ABC1111', 'aaaa'),
('BBBB', 'ABC2222', 'bbbb'),
('CCCC', 'ABC3333', 'cccc');

--
-- Indexes for dumped tables
--

--
-- Indexes for table `Profesores`
--
ALTER TABLE `Profesores`
  ADD UNIQUE KEY `rfc` (`rfc`);

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
"###;
    let output = r###"CREATE TABLE `Profesores` (
`nombre` varchar(50) NOT NULL,
`rfc` varchar(14) NOT NULL,
`usuario` varchar(30) NOT NULL
);
INSERT INTO `Profesores` (`nombre`, `rfc`, `usuario`) VALUES
('AAAA', 'ABC1111', 'aaaa'),
('BBBB', 'ABC2222', 'bbbb'),
('CCCC', 'ABC3333', 'cccc');
"###;
    assert_eq!(&preprocess_sql(input), &output)
  }

  #[test]
  fn test_get_professors() {
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    connection
      .execute_batch(
        r###"CREATE TABLE `Profesores` (
`nombre` varchar(50) NOT NULL,
`rfc` varchar(14) NOT NULL,
`usuario` varchar(30) NOT NULL
);
INSERT INTO `Profesores` (`nombre`, `rfc`, `usuario`) VALUES
('AAAA', 'ABC1111', 'aaaa'),
('BBBB', 'ABC2222', 'bbbb'),
('CCCC', 'ABC3333', 'cccc');
"###,
      )
      .unwrap();
    let output = vec![
      ProfessorRow {
        nombre: "AAAA".to_string(),
        rfc: "ABC1111".to_string(),
      },
      ProfessorRow {
        nombre: "BBBB".to_string(),
        rfc: "ABC2222".to_string(),
      },
      ProfessorRow {
        nombre: "CCCC".to_string(),
        rfc: "ABC3333".to_string(),
      },
    ];
    assert_eq!(get_professor_rows(&connection).unwrap(), output);
  }
}
