#![allow(deprecated)]

use leptos::{
    IntoView, ev,
    prelude::*,
    reactive::{
        signal::create_signal,
        traits::{Get, Set},
    },
    view,
};
use leptos_use::{use_event_listener, use_window};
use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
// use web_sys::js_sys::futures::spawn_local;

use crate::models::Note;

pub fn app() -> impl IntoView {
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());
    let (status, set_status) = create_signal(String::new());

    let save_note = Rc::new(move || {
        let note = Note {
            id: Some(Uuid::new_v4().to_string()),
            title: Some(title.get()),
            content: Some(content.get()),
        };

        set_status.set("Saving..".to_string());

        spawn_local(async move {
            let req = gloo_net::http::Request::post("http://127.0.0.1:3000/api/notes").json(&note);
            match req {
                Ok(req) => match req.send().await {
                    Ok(response) if response.ok() => {
                        set_status.set("Saved!".to_string());
                    }
                    Ok(response) => {
                        set_status.set(format!("Failed: HTTP {}", response.status()));
                    }
                    Err(err) => {
                        set_status.set(format!("Request error: {err}"));
                    }
                },
                Err(err) => {
                    set_status.set(format!("JSON error: {err}"));
                }
            }
        });
    });

    let save_button = {
        let save_note = Rc::clone(&save_note);
        move |_| save_note()
    };

    let save_keyboard = Rc::clone(&save_note);

    let _listener = use_event_listener(use_window(), ev::keydown, move |evt| {
        if evt.ctrl_key() && evt.key() == "s" {
            evt.prevent_default();
            save_keyboard();
        }
    });

    view! {
        <div class="p-8 max-w-2xl mx-auto font-sans">

            <div class="flex justify-between items-center mb-4">
                <h1 class="text-2xl font-bold text-gray-800">
                    "VaultNote"
                </h1>

                <span class="text-sm text-gray-500">
                    {move || status.get()} //
                </span>
            </div>

            <input
                type="text"
                placeholder="Note Title..."
                class="w-full p-3 mb-4 border border-gray-300 rounded-lg shadow-sm"
                prop:value=title
                on:input=move |ev| {
                    set_title.set(event_target_value(&ev));
                }
            />

            <textarea
                placeholder="Write your thoughts here..."
                class="w-full p-3 h-64 border border-gray-300 rounded-lg shadow-sm resize-none"
                prop:value=content
                on:input=move |ev| {
                    set_content.set(event_target_value(&ev));
                }
            ></textarea>

            <button
                on:click=save_button
                class="mt-4 bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg"
            >
                "Save (Ctrl+S)"
            </button>

        </div>
    }
}
