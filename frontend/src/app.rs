#![allow(deprecated)]

use leptos::{ev, prelude::*, reactive::signal::create_signal, view, IntoView};
use leptos_use::{use_event_listener, use_window};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

use crate::models::{CreateNoteRequest, Note, SaveNoteResponse};

const API: &str = "http://127.0.0.1:3000/api/notes";

#[component]
fn TrashIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14" height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
            <path d="M10 11v6" />
            <path d="M14 11v6" />
            <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
        </svg>
    }
}

#[component]
fn NoteCard(
    note: Note,
    is_selected: bool,
    on_select: Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    let select_id = note.id.clone();
    let delete_id = note.id.clone();

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
        // `group` lets children react to the card's hover state
        <div
            class=move || {
                if is_selected {
                    "group relative flex items-start gap-2 px-3 py-2.5 mx-2 rounded-lg \
                     cursor-pointer bg-gray-700 border border-gray-600"
                } else {
                    "group relative flex items-start gap-2 px-3 py-2.5 mx-2 rounded-lg \
                     cursor-pointer hover:bg-gray-700/60 border border-transparent transition-colors"
                }
            }
        >
            // Text block — takes all space except trash icon
            <div
                class="flex flex-col gap-0.5 flex-1 min-w-0"
                on:click=move |_| on_select.run(select_id.clone())
            >
                <span class="text-sm font-semibold text-gray-100 truncate">{title}</span>
                <span class="text-xs text-gray-400 truncate">{preview}</span>
                <span class="text-xs text-gray-500">{date}</span>
            </div>

            // Trash button — hidden until card is hovered
            <button
                class="opacity-0 group-hover:opacity-100 transition-opacity \
                       flex items-center justify-center shrink-0 mt-0.5 p-1 rounded \
                       text-red-400 hover:text-red-300 hover:bg-red-900/30"
                title="Delete note"
                on:click=move |evt| {
                    // Stop propagation so the card's select handler doesn't fire
                    evt.stop_propagation();
                    on_delete.run(delete_id.clone());
                }
            >
                <TrashIcon />
            </button>
        </div>
    }
}


pub fn app() -> impl IntoView {
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());

    let (active_id, set_active_id) = create_signal::<Option<String>>(None);

    let (notes, set_notes) = create_signal::<Vec<Note>>(vec![]);

    let (saved, set_saved) = create_signal::<Option<bool>>(None);

    let (status, set_status) = create_signal::<Option<(bool, String)>>(None);

    {
        let set_notes = set_notes;
        spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get(API).send().await {
                if resp.ok() {
                    if let Ok(fetched) = resp.json::<Vec<Note>>().await {
                        set_notes.set(fetched);
                    }
                }
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
            match gloo_net::http::Request::post(API).json(&payload) {
                Err(e) => {
                    set_status.set(Some((false, format!("Serialisation error: {e}"))));
                    set_saved.set(Some(false));
                }
                Ok(req) => match req.send().await {
                    Err(e) => {
                        set_status.set(Some((false, format!("Network error: {e}"))));
                        set_saved.set(Some(false));
                    }
                    Ok(resp) => {
                        if resp.ok() {
                            match resp.json::<SaveNoteResponse>().await {
                                Ok(body) => {
                                    let saved_note = body.note.clone();
                                    set_active_id.set(Some(saved_note.id.clone()));
                                    set_status.set(Some((true, format!("✓ {}", body.message))));
                                    set_saved.set(Some(true));

                                    set_notes.update(|list| {
                                        if let Some(pos) =
                                            list.iter().position(|n| n.id == saved_note.id)
                                        {
                                            list[pos] = saved_note.clone();
                                        } else {
                                            list.insert(0, saved_note.clone());
                                        }
                                        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                                    });
                                }
                                Err(e) => {
                                    set_status.set(Some((false, format!("Parse error: {e}"))));
                                    set_saved.set(Some(false));
                                }
                            }
                        } else {
                            let code = resp.status();
                            let body = resp.text().await.unwrap_or_default();
                            set_status.set(Some((
                                false,
                                format!("Server error {code}: {body}"),
                            )));
                            set_saved.set(Some(false));
                        }
                    }
                },
            }
        });
    });

    let save_keyboard = Rc::clone(&save_note);
    let _listener = use_event_listener(use_window(), ev::keydown, move |evt| {
        if evt.ctrl_key() && evt.key() == "s" {
            evt.prevent_default();
            save_keyboard();
        }
    });

    let on_new_note = move |_| {
        set_active_id.set(None);
        set_title.set(String::new());
        set_content.set(String::new());
        set_status.set(None);
        set_saved.set(None);
    };

    let on_select = Callback::new(move |id: String| {
        let list = notes.get();
        if let Some(note) = list.iter().find(|n| n.id == id) {
            set_active_id.set(Some(note.id.clone()));
            set_title.set(note.title.clone());
            set_content.set(note.content.clone());
            set_status.set(None);
            set_saved.set(Some(true));
        }
    });

    let on_delete = Callback::new(move |id: String| {
        let delete_url = format!("{API}/{id}");
        let current_active = active_id.get();

        spawn_local(async move {
            match gloo_net::http::Request::delete(&delete_url).send().await {
                Ok(resp) if resp.ok() => {
                    set_notes.update(|list| list.retain(|n| n.id != id));

                    if current_active.as_deref() == Some(&id) {
                        set_active_id.set(None);
                        set_title.set(String::new());
                        set_content.set(String::new());
                        set_status.set(None);
                        set_saved.set(None);
                    }
                }
                Ok(resp) => {
                    let code = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    set_status.set(Some((false, format!("Delete failed {code}: {body}"))));
                }
                Err(e) => {
                    set_status.set(Some((false, format!("Delete network error: {e}"))));
                }
            }
        });
    });

    let save_btn = {
        let save_note = Rc::clone(&save_note);
        move |_| save_note()
    };

    let word_count = move || {
        let c = content.get();
        if c.trim().is_empty() {
            0usize
        } else {
            c.split_whitespace().count()
        }
    };
    let char_count = move || content.get().len();

    view! {
        <div class="flex h-screen bg-gray-900 text-gray-100 font-sans overflow-hidden">

            // ══ SIDEBAR ════════════════════════════════════════════════════
            <aside class="flex flex-col w-64 min-w-56 bg-gray-800 border-r border-gray-700 select-none shrink-0">

                // Brand
                <div class="flex items-center gap-2 px-4 py-4 border-b border-gray-700">
                    <span class="text-lg font-bold text-white tracking-tight">"🔐 VaultNote"</span>
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

                // Section label
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
                                        on_delete=on_delete
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

                // ── Top bar ─────────────────────────────────────────────
                <header class="flex items-center justify-between px-6 py-3 border-b border-gray-700 bg-gray-800 shrink-0">
                    <span class="text-sm font-semibold text-gray-300">
                        {move || if active_id.get().is_some() { "Edit Note" } else { "New Note" }}
                    </span>

                    // Ephemeral status chip (errors, "Saving…", "✓ Saved")
                    {move || match status.get() {
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
                    }}
                </header>

                // ── Editor body ──────────────────────────────────────────
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
                        on:input=move |ev| {
                            set_title.set(event_target_value(&ev));
                            set_saved.set(Some(false));
                        }
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
                        on:input=move |ev| {
                            set_content.set(event_target_value(&ev));
                            set_saved.set(Some(false));
                        }
                    />

                    // ── Stats + save-state bar ───────────────────────────
                    <div class="flex justify-between items-center px-1 text-xs text-gray-500">
                        // Left: live counts
                        <span>
                            {move || format!("{} words · {} chars", word_count(), char_count())}
                        </span>

                        // Right: unsaved / saved indicator
                        {move || match saved.get() {
                            None => view! { <span /> }.into_any(),
                            Some(true) => view! {
                                <span class="text-green-400 font-medium">"✓ Saved"</span>
                            }.into_any(),
                            Some(false) => view! {
                                <span class="text-yellow-400 font-medium">"● Unsaved changes"</span>
                            }.into_any(),
                        }}
                    </div>
                </div>

                // ── Action bar / footer ──────────────────────────────────
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
