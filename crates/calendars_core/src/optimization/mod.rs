mod class_calendar;
mod heuristics;
mod methods;
mod optimization_constraints;
mod stats_tracker;

pub use class_calendar::ClassCalendar;
pub use class_calendar::SingleClassEntry;
pub use optimization_constraints::AllowedClassroomTypes;
pub use optimization_constraints::Availability;
pub use optimization_constraints::Class;
pub use optimization_constraints::ClassKey;
pub use optimization_constraints::Classroom;
pub use optimization_constraints::ClassroomType;
pub use optimization_constraints::Group;
pub use optimization_constraints::OptimizationConstraints;
pub use optimization_constraints::Professor;
pub use optimization_constraints::ProfessorKey;
pub use optimization_constraints::Semester;

pub use methods::simulated_annealing::SimulatedAnnealingOptimizer;

#[deprecated]
pub use methods::simulated_annealing::AdvancedSimulationOptions;
#[deprecated]
pub use methods::simulated_annealing::LiveUpdate;
#[deprecated]
pub use methods::simulated_annealing::ProgressOption;
#[deprecated]
pub use methods::simulated_annealing::SimulationOptions;
#[deprecated]
pub use methods::simulated_annealing::SimulationOutput;
#[deprecated]
pub use methods::simulated_annealing::TemperatureFunction;

pub use methods::simulated_annealing::StopCondition;
pub type CostFunction = fn(&ClassCalendar, &OptimizationConstraints) -> f64;

pub trait ClassCalendarOptimizer {
  type OptimizerOptions;

  fn generate_class_calendar(
    &mut self,
    constraints: OptimizationConstraints,
    options: Self::OptimizerOptions,
    cost_function: Option<CostFunction>,
  ) -> ClassCalendar;
}
