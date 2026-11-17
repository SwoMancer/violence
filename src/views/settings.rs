use crate::{database, Route, Theme, ThemeSignal};
use dioxus::prelude::*;

const DEFAULT_CONFIRM_MINUTES: i64 = 5;
const DEFAULT_REFIRE_MINUTES: i64 = 15;

fn load_initial_minutes() -> (i64, i64) {
    #[cfg(target_os = "android")]
    {
        match crate::scheduler::get_global_settings() {
            Ok((c, r)) => (c / 60_000, r / 60_000),
            Err(e) => {
                log::error!("get_global_settings failed: {e}");
                (DEFAULT_CONFIRM_MINUTES, DEFAULT_REFIRE_MINUTES)
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        (DEFAULT_CONFIRM_MINUTES, DEFAULT_REFIRE_MINUTES)
    }
}

#[component]
pub fn Settings() -> Element {
    let (initial_confirm, initial_refire) = load_initial_minutes();
    let mut confirm_minutes = use_signal(|| initial_confirm.to_string());
    let mut refire_minutes = use_signal(|| initial_refire.to_string());
    let mut error = use_signal(String::new);
    let mut theme = use_context::<ThemeSignal>();
    let mut pending_theme = use_signal(|| theme());

    let navigator = use_navigator();

    let save = move |_| {
        let confirm = match confirm_minutes.read().trim().parse::<i64>() {
            Ok(n) if n > 0 => n,
            _ => {
                error.set("Confirm delay must be a positive integer.".to_string());
                return;
            }
        };
        let refire = match refire_minutes.read().trim().parse::<i64>() {
            Ok(n) if n > 0 => n,
            _ => {
                error.set("Refire delay must be a positive integer.".to_string());
                return;
            }
        };
        if refire <= confirm {
            error.set("Refire delay must be greater than confirm delay.".to_string());
            return;
        }
        error.set(String::new());

        let confirm_ms = confirm * 60_000;
        let refire_ms = refire * 60_000;

        #[cfg(target_os = "android")]
        {
            if let Err(e) = crate::scheduler::set_global_settings(confirm_ms, refire_ms) {
                log::error!("set_global_settings failed: {e}");
                error.set(format!("Failed to save: {e}"));
                return;
            }
        }
        #[cfg(not(target_os = "android"))]
        {
            log::info!(
                "set_global_settings (desktop no-op): confirm={confirm_ms}ms refire={refire_ms}ms"
            );
        }

        let selected = pending_theme();
        database::set_theme(selected);
        theme.set(selected);
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
                    h1 { class: "text-base tracking-wide", "settings" }
                }
                p {
                    class: "text-xs text-dim mb-6",
                    "global timer defaults. these apply to every alarm unless that alarm has its own overrides."
                }
                div {
                    class: "mb-5",
                    label {
                        class: "block text-xs uppercase tracking-widest text-dim mb-2",
                        "confirm delay (min)"
                    }
                    input {
                        class: "w-full bg-transparent border-b border-faint focus:border-fg px-0 py-2 outline-none text-base",
                        r#type: "number",
                        min: "1",
                        value: "{confirm_minutes}",
                        oninput: move |evt| confirm_minutes.set(evt.value().clone()),
                    }
                }
                div {
                    class: "mb-5",
                    label {
                        class: "block text-xs uppercase tracking-widest text-dim mb-2",
                        "refire delay (min)"
                    }
                    input {
                        class: "w-full bg-transparent border-b border-faint focus:border-fg px-0 py-2 outline-none text-base",
                        r#type: "number",
                        min: "1",
                        value: "{refire_minutes}",
                        oninput: move |evt| refire_minutes.set(evt.value().clone()),
                    }
                }
                div {
                    class: "mb-5",
                    label {
                        class: "block text-xs uppercase tracking-widest text-dim mb-2",
                        "theme"
                    }
                    div {
                        class: "flex items-center gap-4 text-sm py-2 border-b border-faint",
                        for option in Theme::ALL {
                            button {
                                class: if pending_theme() == option { "text-fg" } else { "text-dim hover:text-fg transition-colors" },
                                onclick: move |_| pending_theme.set(option),
                                "{option.as_str()}"
                            }
                        }
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
