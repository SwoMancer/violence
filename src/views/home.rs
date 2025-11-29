use crate::database::{self, get_connection};
use crate::Route;
use dioxus::prelude::*;

#[derive(Clone, Debug)]
struct Alarm {
    time: String,
    enabled: bool,
    recurrence: String,
    monday: bool,
    tuesday: bool,
    wednesday: bool,
    thursday: bool,
    friday: bool,
    saturday: bool,
    sunday: bool,
}

#[component]
pub fn Home() -> Element {
    let alarms = use_signal_sync(move || -> database::Result<Vec<Alarm>> {
        let conn = get_connection()
            .lock()
            .map_err(|e| database::Error::Mutex(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday FROM alarms")?;
        let alarms_iter = stmt.query_map([], |row| {
            Ok(Alarm {
                time: row.get(0)?,
                enabled: row.get(1)?,
                recurrence: row.get(2)?,
                monday: row.get(3)?,
                tuesday: row.get(4)?,
                wednesday: row.get(5)?,
                thursday: row.get(6)?,
                friday: row.get(7)?,
                saturday: row.get(8)?,
                sunday: row.get(9)?,
            })
        })?;

        let mut alarms = Vec::new();
        for alarm in alarms_iter {
            alarms.push(alarm?);
        }
        Ok(alarms)
    });

    rsx! {
        div {
            class: "bg-gray-800 text-gray-200 min-h-screen",
            div {
                class: "container mx-auto px-4 py-8",
                h1 {
                    class: "text-4xl font-bold mb-4",
                    "violence"
                },
                Link {
                    to: Route::CreateAlarm {},
                    class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                    "Create Alarm"
                },
                match alarms.read().as_ref() {
                    Ok(alarms) => {
                        rsx!{
                            div {
                                class: "mt-8",
                                for alarm in alarms.iter() {
                                    div {
                                        class: "bg-gray-700 p-4 rounded-lg mb-4",
                                        div {
                                            class: "flex justify-between items-center",
                                            p {
                                                class: "text-lg",
                                                "{alarm.time}"
                                            },
                                            p {
                                                class: if alarm.enabled { "text-green-500" } else { "text-red-500" },
                                                if alarm.enabled { "Enabled" } else { "Disabled" }
                                            }
                                        },
                                        div {
                                            class: "text-sm text-gray-400 mt-2",
                                            "Recurrence: {alarm.recurrence}",
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        rsx!{
                            div {
                                class: "bg-red-500 text-white p-4 rounded-lg mt-8",
                                "Error fetching alarms: {e}"
                            }
                        }
                    }
                }
            }
        }
    }
}
