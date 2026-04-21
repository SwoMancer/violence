use dioxus::prelude::*;

use views::{CreateAlarm, EditAlarm, Home, Settings};

pub type ThemeSignal = Signal<bool>;

/// Define a components module that contains all shared components for our app.
mod components;
mod database;
#[cfg(target_os = "android")]
mod scheduler;
mod views;

/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
///
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/create_alarm")]
    CreateAlarm { },
    #[route("/edit_alarm/:id")]
    EditAlarm { id: i64 },
    #[route("/settings")]
    Settings { },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    env_logger::init();
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    let light_mode = use_context_provider(|| Signal::new(database::get_light_mode()));
    let theme_class = if *light_mode.read() { "light" } else { "" };

    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div {
            class: "{theme_class}",
            Router::<Route> {}
        }
    }
}
