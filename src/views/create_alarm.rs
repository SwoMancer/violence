use crate::database;
use dioxus::prelude::*;

use crate::Route;

#[cfg(target_os = "android")]
const DEFAULT_CONFIRM_MS: i64 = 5 * 60_000;
#[cfg(target_os = "android")]
const DEFAULT_REFIRE_MS: i64 = 15 * 60_000;

fn parse_minutes_override(raw: &str) -> Option<i64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<i64>() {
        Ok(n) if n > 0 => Some(n * 60_000),
        _ => {
            log::error!("invalid minute override: {raw:?}");
            None
        }
    }
}

#[component]
pub fn CreateAlarm() -> Element {
    let mut time_str = use_signal(|| "00:00".to_string());
    let mut enabled = use_signal(|| true);
    let mut recurrence = use_signal(|| "one-off".to_string());
    let mut monday = use_signal(|| false);
    let mut tuesday = use_signal(|| false);
    let mut wednesday = use_signal(|| false);
    let mut thursday = use_signal(|| false);
    let mut friday = use_signal(|| false);
    let mut saturday = use_signal(|| false);
    let mut sunday = use_signal(|| false);
    let mut confirm_override = use_signal(String::new);
    let mut refire_override = use_signal(String::new);
    let mut error = use_signal(String::new);

    let navigator = use_navigator();

    let save = move |_| {
        let confirm_opt = parse_minutes_override(&confirm_override.read());
        let refire_opt = parse_minutes_override(&refire_override.read());
        if let (Some(c), Some(r)) = (confirm_opt, refire_opt) {
            if r <= c {
                error.set("Refire delay must be greater than confirm delay.".to_string());
                return;
            }
        }
        error.set(String::new());

        let time_str = {
            let v = time_str.read().clone();
            if v.is_empty() { "00:00".to_string() } else { v }
        };

        match database::insert_alarm(
            &time_str,
            *enabled.read(),
            &recurrence.read(),
            *monday.read(),
            *tuesday.read(),
            *wednesday.read(),
            *thursday.read(),
            *friday.read(),
            *saturday.read(),
            *sunday.read(),
            confirm_opt,
            refire_opt,
        ) {
            Ok(_id) => {
                #[cfg(target_os = "android")]
                if *enabled.read() {
                    let (global_confirm, global_refire) =
                        crate::scheduler::get_global_settings()
                            .unwrap_or((DEFAULT_CONFIRM_MS, DEFAULT_REFIRE_MS));
                    let effective_confirm = confirm_opt.unwrap_or(global_confirm);
                    let effective_refire = refire_opt.unwrap_or(global_refire);
                    if let Err(e) = crate::scheduler::schedule_alarm(
                        _id,
                        &time_str,
                        effective_confirm,
                        effective_refire,
                    ) {
                        log::error!("schedule_alarm failed: {e}");
                    }
                }
            }
            Err(e) => log::error!("failed to create alarm: {e}"),
        }
        navigator.push(Route::Home {});
    };

    rsx! {
        div {
            class: "bg-bg text-fg min-h-screen",
            div {
                class: "max-w-2xl mx-auto px-5 py-6",
                div {
                    class: "flex items-baseline gap-3 mb-6 pb-3 border-b border-faint",
                    Link { to: Route::Home {}, class: "text-dim hover:text-fg text-sm transition-colors", "←" }
                    h1 { class: "text-base tracking-wide", "new alarm" }
                }
                div {
                    class: "mb-5",
                    label {
                        class: "block text-xs uppercase tracking-widest text-dim mb-2",
                        "time"
                    }
                    input {
                        class: "w-full bg-transparent border-b border-faint focus:border-fg px-0 py-2 outline-none text-base tabular-nums",
                        r#type: "time",
                        value: "{time_str}",
                        oninput: move |evt| time_str.set(evt.value()),
                    }
                }
                div {
                    class: "mb-5",
                    label {
                        class: "flex items-center gap-2 text-sm",
                        input {
                            r#type: "checkbox",
                            checked: "{enabled}",
                            oninput: move |evt| enabled.set(evt.value() == "true"),
                        }
                        "enabled"
                    }
                }
                div {
                    class: "mb-5",
                    label {
                        class: "block text-xs uppercase tracking-widest text-dim mb-2",
                        "recurrence"
                    }
                    select {
                        class: "w-full bg-transparent border-b border-faint focus:border-fg px-0 py-2 outline-none text-base appearance-none",
                        onchange: move |evt| recurrence.set(evt.value().clone()),
                        option { value: "one-off", "one-off" }
                        option { value: "daily", "daily" }
                    }
                }
                if recurrence.read().clone() == "daily" {
                    div {
                        class: "mb-5",
                        label {
                            class: "block text-xs uppercase tracking-widest text-dim mb-2",
                            "repeat on"
                        }
                        div {
                            class: "grid grid-cols-4 gap-x-4 gap-y-2 text-sm",
                            label { class: "flex items-center gap-2", input { r#type: "checkbox", oninput: move |evt| monday.set(evt.value() == "true"), }, "mon" }
                            label { class: "flex items-center gap-2", input { r#type: "checkbox", oninput: move |evt| tuesday.set(evt.value() == "true"), }, "tue" }
                            label { class: "flex items-center gap-2", input { r#type: "checkbox", oninput: move |evt| wednesday.set(evt.value() == "true"), }, "wed" }
                            label { class: "flex items-center gap-2", input { r#type: "checkbox", oninput: move |evt| thursday.set(evt.value() == "true"), }, "thu" }
                            label { class: "flex items-center gap-2", input { r#type: "checkbox", oninput: move |evt| friday.set(evt.value() == "true"), }, "fri" }
                            label { class: "flex items-center gap-2", input { r#type: "checkbox", oninput: move |evt| saturday.set(evt.value() == "true"), }, "sat" }
                            label { class: "flex items-center gap-2", input { r#type: "checkbox", oninput: move |evt| sunday.set(evt.value() == "true"), }, "sun" }
                        }
                    }
                }
                div {
                    class: "mb-5",
                    label {
                        class: "block text-xs uppercase tracking-widest text-dim mb-2",
                        "confirm delay override (min)"
                    }
                    input {
                        class: "w-full bg-transparent border-b border-faint focus:border-fg px-0 py-2 outline-none text-base",
                        r#type: "number",
                        min: "1",
                        value: "{confirm_override}",
                        oninput: move |evt| confirm_override.set(evt.value().clone()),
                    }
                }
                div {
                    class: "mb-5",
                    label {
                        class: "block text-xs uppercase tracking-widest text-dim mb-2",
                        "refire delay override (min)"
                    }
                    input {
                        class: "w-full bg-transparent border-b border-faint focus:border-fg px-0 py-2 outline-none text-base",
                        r#type: "number",
                        min: "1",
                        value: "{refire_override}",
                        oninput: move |evt| refire_override.set(evt.value().clone()),
                    }
                }
                if !error.read().is_empty() {
                    div {
                        class: "border border-danger text-danger px-3 py-2 text-sm mb-4",
                        "{error}"
                    }
                }
                div {
                    class: "flex items-center gap-5 mt-6",
                    button {
                        class: "border border-faint hover:border-fg px-4 py-2 text-sm transition-colors",
                        onclick: save,
                        "save"
                    }
                    Link {
                        to: Route::Home {},
                        class: "text-dim hover:text-fg text-sm transition-colors",
                        "cancel"
                    }
                }
            }
        }
    }
}

