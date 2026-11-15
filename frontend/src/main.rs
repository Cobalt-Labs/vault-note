mod app;
mod models;

use crate::app::app;

fn main() {
    leptos::mount::mount_to_body(app);
}
