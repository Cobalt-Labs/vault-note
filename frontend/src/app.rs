#![allow(deprecated)]

use leptos::{ev, prelude::*, reactive::signal::create_signal, view, IntoView};
use leptos_use::{use_event_listener, use_window};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

use crate::models::{CreateNoteRequest, Note, SaveNoteResponse};

const API: &str = "http://127.0.0.1:3000/api/notes";

#[component]
fn NoteCard(
    note: Note,
    is_selected: bool,
    on_select: Callback<String>,
) -> impl IntoView {
    let id = note.id.clone();
    let title = if note.title.is_empty() {
        "Untitled".to_string()
    } else {
        note.title.clone()
    };
    let preview = if note.content.len() > 50 {
        format!("{}…", &note.content[..50])
    } else {
        note.content.clone()
    };

    let date = note.updated_at.get(..10).unwrap_or("").to_string();

    view! {
        <div
            class=move || {
                if is_selected {
                    "flex flex-col gap-1 px-3 py-2.5 mx-2 rounded-lg cursor-pointer \
                     bg-gray-700 border border-gray-600"
                } else {
                    "flex flex-col gap-1 px-3 py-2.5 mx-2 rounded-lg cursor-pointer \
                     hover:bg-gray-700/60 border border-transparent transition-colors"
                }
            }
            on:click=move |_| on_select.run(id.clone())
        >
            <span class="text-sm font-semibold text-gray-100 truncate">{title}</span>
            <span class="text-xs text-gray-400 truncate">{preview}</span>
            <span class="text-xs text-gray-500">{date}</span>
        </div>
    }
}


pub fn app() -> impl IntoView {
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());
    let (status, set_status) = create_signal::<Option<(bool, String)>>(None);

    let (active_id, set_active_id) = create_signal::<Option<String>>(None);

    let (notes, set_notes) = create_signal::<Vec<Note>>(vec![]);

    {
        let set_notes = set_notes;
        spawn_local(async move {
            match gloo_net::http::Request::get(API).send().await {
                Ok(resp) if resp.ok() => {
                    if let Ok(fetched) = resp.json::<Vec<Note>>().await {
                        set_notes.set(fetched);
                    }
                }
                _ => {} // ignore load errors silently on startup
            }
        });
    }

    let save_note = Rc::new(move || {
        let current_title = title.get();
        if current_title.trim().is_empty() {
            set_status.set(Some((false, "⚠ Title cannot be empty.".into())));
            return;
        }

        let payload = CreateNoteRequest {
            id: active_id.get(),
            title: current_title,
            content: content.get(),
        };

        set_status.set(Some((true, "Saving…".into())));

        spawn_local(async move {
            let req = gloo_net::http::Request::post(API).json(&payload);
            match req {
                Err(e) => {
                    set_status.set(Some((false, format!("Serialisation error: {e}"))));
                }
                Ok(req) => match req.send().await {
                    Err(e) => {
                        set_status.set(Some((false, format!("Network error: {e}"))));
                    }
                    Ok(resp) => {
                        if resp.ok() {
                            match resp.json::<SaveNoteResponse>().await {
                                Ok(body) => {
                                    let saved = body.note.clone();
                                    set_active_id.set(Some(saved.id.clone()));
                                    set_status.set(Some((true, format!("✓ {}", body.message))));

                                    // Upsert the note in the local sidebar list
                                    set_notes.update(|list| {
                                        if let Some(pos) = list.iter().position(|n| n.id == saved.id) {
                                            list[pos] = saved.clone();
                                        } else {
                                            list.insert(0, saved.clone());
                                        }
                                        // Keep sorted by updated_at desc
                                        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                                    });
                                }
                                Err(e) => {
                                    set_status.set(Some((false, format!("Parse error: {e}"))));
                                }
                            }
                        } else {
                            let status_code = resp.status();
                            let body = resp.text().await.unwrap_or_default();
                            set_status.set(Some((
                                false,
                                format!("Server error {status_code}: {body}"),
                            )));
                        }
                    }
                },
            }
        });
    });

    // ── Ctrl+S keyboard shortcut ──────────────────────────────────────────
    let save_keyboard = Rc::clone(&save_note);
    let _listener = use_event_listener(use_window(), ev::keydown, move |evt| {
        if evt.ctrl_key() && evt.key() == "s" {
            evt.prevent_default();
            save_keyboard();
        }
    });

    // ── new note button ───────────────────────────────────────────────────
    let on_new_note = move |_| {
        set_active_id.set(None);
        set_title.set(String::new());
        set_content.set(String::new());
        set_status.set(None);
    };

    // ── select a note from the sidebar ───────────────────────────────────
    let on_select = Callback::new(move |id: String| {
        let list = notes.get();
        if let Some(note) = list.iter().find(|n| n.id == id) {
            set_active_id.set(Some(note.id.clone()));
            set_title.set(note.title.clone());
            set_content.set(note.content.clone());
            set_status.set(None);
        }
    });

    // ── save button click ─────────────────────────────────────────────────
    let save_btn = {
        let save_note = Rc::clone(&save_note);
        move |_| save_note()
    };

    // ─── view ─────────────────────────────────────────────────────────────
    view! {
        <div class="flex h-screen bg-gray-900 text-gray-100 font-sans overflow-hidden">

            // ══ SIDEBAR ════════════════════════════════════════════════════
            <aside class="flex flex-col w-64 min-w-56 bg-gray-800 border-r border-gray-700 select-none shrink-0">

                // Brand header
                <div class="flex items-center gap-2 px-4 py-4 border-b border-gray-700">
                    <span class="text-lg font-bold text-white tracking-tight">"VaultNote"</span>
                </div>

                // New note button
                <div class="px-3 py-3">
                    <button
                        on:click=on_new_note
                        class="w-full flex items-center gap-2 px-3 py-2 rounded-lg \
                               bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium \
                               transition-colors"
                    >
                        <span class="text-base">"＋"</span>
                        "New Note"
                    </button>
                </div>

                // Notes section header
                <div class="px-4 pb-1 pt-2">
                    <span class="text-xs font-semibold uppercase tracking-widest text-gray-500">
                        "Recents"
                    </span>
                </div>

                // Scrollable note list
                <div class="flex-1 overflow-y-auto py-1 space-y-0.5">
                    {move || {
                        let list = notes.get();
                        let current_id = active_id.get();
                        if list.is_empty() {
                            view! {
                                <p class="px-5 py-4 text-xs text-gray-500 italic">
                                    "No notes yet — create one!"
                                </p>
                            }.into_any()
                        } else {
                            list.into_iter().map(|note| {
                                let is_sel = current_id.as_deref() == Some(&note.id);
                                view! {
                                    <NoteCard
                                        note=note
                                        is_selected=is_sel
                                        on_select=on_select
                                    />
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>

                // Sidebar footer
                <div class="px-4 py-3 border-t border-gray-700 text-xs text-gray-500">
                    "In-memory cache · SQLite coming soon"
                </div>
            </aside>

            // ══ EDITOR PANE ════════════════════════════════════════════════
            <main class="flex flex-col flex-1 overflow-hidden">

                // Top bar
                <header class="flex items-center justify-between px-6 py-3 border-b border-gray-700 bg-gray-800 shrink-0">
                    <span class="text-sm font-semibold text-gray-300">
                        {move || {
                            if active_id.get().is_some() {
                                "Edit Note"
                            } else {
                                "New Note"
                            }
                        }}
                    </span>

                    // Status indicator
                    {move || {
                        match status.get() {
                            None => view! { <span /> }.into_any(),
                            Some((ok, msg)) => view! {
                                <span class=move || {
                                    if ok {
                                        "text-xs px-2 py-1 rounded-full bg-green-800/50 text-green-300"
                                    } else {
                                        "text-xs px-2 py-1 rounded-full bg-red-800/50 text-red-300"
                                    }
                                }>
                                    {msg}
                                </span>
                            }.into_any(),
                        }
                    }}
                </header>

                // Editor body
                <div class="flex flex-col flex-1 p-6 gap-4 overflow-y-auto">

                    // Title input
                    <input
                        type="text"
                        placeholder="Note title…"
                        class="w-full px-4 py-2.5 text-lg font-semibold \
                               bg-gray-800 border border-gray-600 rounded-lg \
                               text-gray-100 placeholder-gray-500 \
                               focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 \
                               transition-colors"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                    />

                    // Content textarea
                    <textarea
                        placeholder="Write your thoughts here…"
                        class="flex-1 w-full px-4 py-3 text-sm \
                               bg-gray-800 border border-gray-600 rounded-lg \
                               text-gray-100 placeholder-gray-500 \
                               focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 \
                               resize-none transition-colors"
                        prop:value=content
                        on:input=move |ev| set_content.set(event_target_value(&ev))
                    />
                </div>

                // Action bar / footer
                <footer class="flex items-center justify-between px-6 py-3 border-t border-gray-700 bg-gray-800 shrink-0">
                    <span class="text-xs text-gray-500">"Ctrl + S to save"</span>
                    <button
                        on:click=save_btn
                        class="px-5 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 \
                               text-white text-sm font-semibold tracking-wide \
                               transition-colors shadow"
                    >
                        "Save"
                    </button>
                </footer>
            </main>

        </div>
    }
}
