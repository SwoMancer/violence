//! The views module contains the components for all routes in our app.

mod home;
pub use home::Home;

mod create_alarm;
pub use create_alarm::CreateAlarm;

mod edit_alarm;
pub use edit_alarm::EditAlarm;

mod settings;
pub use settings::Settings;
