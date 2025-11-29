use crate::database::{self, get_connection};
use dioxus::prelude::*;

use crate::Route;

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn CreateAlarm() -> Element {
    let mut time = use_signal(String::new);
    let mut enabled = use_signal(|| true);
    let mut recurrence = use_signal(|| "one-off".to_string());
    let mut monday = use_signal(|| false);
    let mut tuesday = use_signal(|| false);
    let mut wednesday = use_signal(|| false);
    let mut thursday = use_signal(|| false);
    let mut friday = use_signal(|| false);
    let mut saturday = use_signal(|| false);
    let mut sunday = use_signal(|| false);

    let navigator = use_navigator();

    let save = move |_| {
        if let Err(e) = create_alarm(
            time.read().clone(),
            *enabled.read(),
            recurrence.read().clone(),
            *monday.read(),
            *tuesday.read(),
            *wednesday.read(),
            *thursday.read(),
            *friday.read(),
            *saturday.read(),
            *sunday.read(),
        ) {
            log::error!("failed to create alarm: {}", e);
        }
        navigator.push(Route::Home {});
    };

    rsx! {
        div {
            class: "bg-gray-800 text-gray-200 min-h-screen",
            div {
                class: "container mx-auto px-4 py-8",
                h1 {
                    class: "text-4xl font-bold mb-4",
                    "Create Alarm"
                },
                div {
                    class: "mb-4",
                    label {
                        class: "block text-gray-400 text-sm font-bold mb-2",
                        "Time"
                    },
                    input {
                        class: "bg-gray-700 text-white rounded py-2 px-3 w-full",
                        r#type: "time",
                        value: "{time}",
                        oninput: move |evt| time.set(evt.value().clone()),
                    }
                },
                div {
                    class: "mb-4",
                    label {
                        class: "flex items-center",
                        input {
                            class: "mr-2",
                            r#type: "checkbox",
                            checked: "{enabled}",
                            oninput: move |evt| enabled.set(evt.value() == "true"),
                        },
                        "Enabled"
                    }
                },
                div {
                    class: "mb-4",
                    label {
                        class: "block text-gray-400 text-sm font-bold mb-2",
                        "Recurrence"
                    },
                    select {
                        class: "bg-gray-700 text-white rounded py-2 px-3 w-full",
                        onchange: move |evt| recurrence.set(evt.value().clone()),
                        option { value: "one-off", "One-off" },
                        option { value: "daily", "Daily" },
                    }
                },
                if recurrence.read().clone() == "daily" {
                    div {
                        class: "mb-4 grid grid-cols-4 gap-2",
                        label { class: "flex items-center", input { class: "mr-2", r#type: "checkbox", oninput: move |evt| monday.set(evt.value() == "true"), }, "Mon" },
                        label { class: "flex items-center", input { class: "mr-2", r#type: "checkbox", oninput: move |evt| tuesday.set(evt.value() == "true"), }, "Tue" },
                        label { class: "flex items-center", input { class: "mr-2", r#type: "checkbox", oninput: move |evt| wednesday.set(evt.value() == "true"), }, "Wed" },
                        label { class: "flex items-center", input { class: "mr-2", r#type: "checkbox", oninput: move |evt| thursday.set(evt.value() == "true"), }, "Thu" },
                        label { class: "flex items-center", input { class: "mr-2", r#type: "checkbox", oninput: move |evt| friday.set(evt.value() == "true"), }, "Fri" },
                        label { class: "flex items-center", input { class: "mr-2", r#type: "checkbox", oninput: move |evt| saturday.set(evt.value() == "true"), }, "Sat" },
                        label { class: "flex items-center", input { class: "mr-2", r#type: "checkbox", oninput: move |evt| sunday.set(evt.value() == "true"), }, "Sun" },
                    }
                }
                div {
                    class: "flex justify-between",
                    button {
                        class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                        onclick: save,
                        "Save"
                    },
                    Link {
                        to: Route::Home {},
                        class: "text-gray-400 hover:text-white",
                        "Back"
                    }
                }
            }
        }
    }
}

fn create_alarm(
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
) -> database::Result<()> {
    let conn = get_connection()
        .lock()
        .map_err(|e| database::Error::Mutex(e.to_string()))?;
    conn.execute(
        "INSERT INTO alarms (time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (time, enabled, recurrence, monday, tuesday, wednesday, thursday, friday, saturday, sunday),
    )?;
    Ok(())
}
