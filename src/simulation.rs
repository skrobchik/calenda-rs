use std::thread::{JoinHandle, self};

use crate::{week_calendar::WeekCalendar, school_schedule::{Classes, SimulationInformation}};

pub fn generate_schedule(simulation_information: SimulationInformation) -> JoinHandle<WeekCalendar<Classes>> {
    let h = thread::spawn(move || {
        let calendar: WeekCalendar<Classes> = Default::default();    
        calendar
    });
    h
}