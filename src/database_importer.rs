use crate::school_schedule::SchoolSchedule;
use std::fs;
use std::path::Path;
use itertools::Itertools;
use anyhow::Context;

const temp_db_path: &str = "temp_db.sqlite";

pub fn import_temporary_database() -> anyhow::Result<()> {
    let materias_sql_export_path = "Archivos SQL/Materias.sql";
    let profesores_sql_export_path = "Archivos SQL/Profesores.sql";
    let connection = sqlite::open(temp_db_path)?;
    let query_profesores = fs::read_to_string(profesores_sql_export_path)?;
    let query_materias = fs::read_to_string(materias_sql_export_path)?;
    connection.execute(query_materias)?;
    connection.execute(query_profesores)?;
    Ok(())
}

pub fn import_database() -> anyhow::Result<SchoolSchedule> {
    let connection = sqlite::open(temp_db_path)?;
    let mut schedule: SchoolSchedule = Default::default();
    let query = "SELECT * FROM Materias";
    let mut counter = 0;
    connection.iterate(query, |rows| {
        for (_, _, _, (_, descripcion), _, _, _, _) in rows.iter().tuple_windows() {
            println!("{}", descripcion.unwrap());
            let (mut class, mut class_metadata) = schedule.add_new_class().context("no more space").unwrap();
            class_metadata.name = String::from(descripcion.unwrap());
            println!("{}", counter);
            counter += 1;
        }
        true
    })?;
    Ok(schedule)
}