use crate::week_calendar::WeekCalendar;

const MAX_CLASSES: usize = 128;
const MAX_PROFESSORS: usize = 128;

enum Availability {
    Prefered,
    Available,
    AvailableIfNeeded,
    NotAvailable
}

struct Professor {
    availability: WeekCalendar<Availability>,  
}

struct ProfessorMetadata {
    name: String,
}

enum ClassroomType {
    Single,
    Double,
    Lab
}

struct ClassMetadata {
    name: String,
}

struct Class {
    professor: usize,
    classroom_type: ClassroomType,
}

struct SimulationInformation {
    classes: [Option<Class>; MAX_CLASSES],
    professors: [Option<Professor>; MAX_PROFESSORS],
    class_hours: [u8; MAX_CLASSES],
}

pub struct SchoolSchedule {
    class_metadata: [Option<ClassMetadata>; MAX_CLASSES],
    professor_metadata: [Option<ProfessorMetadata>; MAX_PROFESSORS],
    simulation_information: SimulationInformation,
    schedule: WeekCalendar<[u8; MAX_CLASSES]>
}

fn generate_schedule(simulation_information: SimulationInformation) -> WeekCalendar<[u8; MAX_CLASSES]> {
    todo!()
}

fn calculate_energy(simulation_information: SimulationInformation) -> f32 {
    todo!()
}
