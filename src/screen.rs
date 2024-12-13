pub mod dashboard;

pub use dashboard::Dashboard;

pub enum Screen {
    Dashboard(Dashboard),
}
