use crate::database;
use crate::Route;
use dioxus::prelude::*;

#[cfg(target_os = "android")]
const DEFAULT_CONFIRM_MS: i64 = 5 * 60_000;
#[cfg(target_os = "android")]
const DEFAULT_REFIRE_MS: i64 = 15 * 60_000;

#[component]
pub fn Home() -> Element {
    let mut alarms = use_signal(database::load_alarms);

    // Re-register all enabled alarms with AlarmManager on every app open.
    // Covers alarms lost after reboot or process death.
    #[cfg(target_os = "android")]
    use_hook(|| crate::scheduler::reschedule_all_enabled());

    let mut delete = move |id: i64| {
        if let Err(e) = database::delete_alarm(id) {
            log::error!("delete_alarm failed: {e}");
            return;
        }
        if let Ok(list) = alarms.write().as_mut() {
            list.retain(|a| a.id != id);
        }
    };

    let mut toggle = move |id: i64| {
        #[cfg_attr(not(target_os = "android"), allow(unused_variables))]
        let (new_enabled, time, confirm_override, refire_override) = match alarms.read().as_ref() {
            Ok(list) => match list.iter().find(|a| a.id == id) {
                Some(a) => (
                    !a.enabled,
                    a.time.clone(),
                    a.confirm_delay_ms,
                    a.refire_delay_ms,
                ),
                None => return,
            },
            Err(_) => return,
        };

        if let Err(e) = database::set_enabled(id, new_enabled) {
            log::error!("set_enabled failed: {e}");
            return;
        }

        #[cfg(target_os = "android")]
        {
            if new_enabled {
                let (global_confirm, global_refire) = crate::scheduler::get_global_settings()
                    .unwrap_or((DEFAULT_CONFIRM_MS, DEFAULT_REFIRE_MS));
                let effective_confirm = confirm_override.unwrap_or(global_confirm);
                let effective_refire = refire_override.unwrap_or(global_refire);
                if let Err(e) = crate::scheduler::schedule_alarm(
                    id,
                    &time,
                    effective_confirm,
                    effective_refire,
                ) {
                    log::error!("schedule_alarm failed: {e}");
                }
            } else if let Err(e) = crate::scheduler::cancel_alarm(id) {
                log::error!("cancel_alarm failed: {e}");
            }
        }

        if let Ok(list) = alarms.write().as_mut() {
            if let Some(a) = list.iter_mut().find(|a| a.id == id) {
                a.enabled = new_enabled;
            }
        }
    };

    rsx! {
        div {
            class: "bg-bg text-fg min-h-screen",
            div {
                class: "max-w-2xl mx-auto px-5 py-6",
                div {
                    class: "flex items-baseline justify-between mb-6 pb-3 border-b border-faint",
                    h1 { class: "text-base tracking-wide", "violence" }
                    match alarms.read().as_ref() {
                        Ok(list) => rsx!{ span { class: "text-xs text-dim", "{list.len()} alarms" } },
                        Err(_) => rsx!{ span { class: "text-xs text-danger", "error" } },
                    }
                }
                div {
                    class: "flex gap-5 mb-8 text-sm",
                    Link {
                        to: Route::CreateAlarm {},
                        class: "text-dim hover:text-fg transition-colors",
                        "new alarm"
                    }
                    Link {
                        to: Route::Settings {},
                        class: "text-dim hover:text-fg transition-colors",
                        "settings"
                    }
                }
                match alarms.read().as_ref() {
                    Ok(alarms) => rsx!{
                        div {
                            class: "border-t border-faint",
                            for alarm in alarms.iter() {
                                div {
                                    class: "flex items-center gap-4 py-3 border-b border-faint",
                                    Link {
                                        to: Route::EditAlarm { id: alarm.id },
                                        class: "text-2xl tabular-nums hover:text-fg flex-1",
                                        "{alarm.time}"
                                    }
                                    span { class: "text-xs text-dim w-16 text-right", "{alarm.recurrence}" }
                                    button {
                                        class: if alarm.enabled { "text-fg w-6 text-center transition-colors" } else { "text-dim w-6 text-center transition-colors" },
                                        onclick: {
                                            let id = alarm.id;
                                            move |_| toggle(id)
                                        },
                                        if alarm.enabled { "●" } else { "○" }
                                    }
                                    button {
                                        class: "text-xs text-dim hover:text-danger transition-colors",
                                        onclick: {
                                            let id = alarm.id;
                                            move |_| delete(id)
                                        },
                                        "delete"
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => rsx!{
                        div {
                            class: "border border-danger text-danger px-3 py-2 text-sm",
                            "error fetching alarms: {e}"
                        }
                    }
                }
            }
        }
    }
}
