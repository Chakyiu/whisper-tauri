use leptos::ev::MouseEvent;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_meta::provide_meta_context;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::debug_view::DebugView;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct WhisperModel {
    name: String,
    size: String,
    url: String,
    downloaded: bool,
    file_path: Option<String>,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let (name, set_name) = signal(String::new());
    let (greet_msg, set_greet_msg) = signal(String::new());

    let (tab, set_tab) = signal(String::new());
    let change_tab = move |ev: SubmitEvent, tab: String| {
        ev.prevent_default();

        spawn_local(async move {
            set_tab.set(tab);
        });
    };
    let get_tab = move || {
        match tab.get().as_str() {
            "Debug" => DebugView,
            _ => DebugView
        }
    };

    let update_name = move |ev: SubmitEvent| {
        let v = event_target_value(&ev);
        set_name.set(v);
    };

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let name = name.get_untracked();
            if name.is_empty() {
                return;
            }

            let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name }).unwrap();
            // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
            let new_msg = invoke("greet", args).await.as_string().unwrap();
            set_greet_msg.set(new_msg);
        });
    };

    let (available_models, set_available_models): (
        ReadSignal<Vec<WhisperModel>>,
        WriteSignal<Vec<WhisperModel>>,
    ) = signal(Vec::new());
    let get_available_models = move |ev: MouseEvent| {
        ev.prevent_default();

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let result = invoke("get_available_models", args).await;
            let model: Vec<WhisperModel> = serde_wasm_bindgen::from_value(result).unwrap();
            set_available_models.set(model);

            let updated: Vec<String> = available_models
                .read_untracked()
                .iter()
                .map(|item| item.name.clone())
                .collect();
            log::info!("{:?}", updated);
        });
    };

    view! {
        <main class="container p-8 mx-auto min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
            {get_tab}
        // <div class="mx-auto max-w-4xl">
        // <h1 class="mb-8 text-4xl font-bold text-center text-gray-800">"Welcome to Tauri + Leptos"</h1>

        // <div class="flex gap-8 justify-center items-center mb-12">
        // <a
        // href="https://tauri.app"
        // target="_blank"
        // class="transition-transform hover:scale-110 hover:drop-shadow-lg"
        // >
        // <img src="public/tauri.svg" class="w-24 h-24" alt="Tauri logo" />
        // </a>
        // <a
        // href="https://docs.rs/leptos/"
        // target="_blank"
        // class="transition-transform hover:scale-110 hover:drop-shadow-lg"
        // >
        // <img src="public/leptos.svg" class="w-24 h-24" alt="Leptos logo" />
        // </a>
        // </div>

        // <p class="mb-8 text-center text-gray-600">"Click on the Tauri and Leptos logos to learn more."</p>

        // <div class="p-6 mb-6 bg-white rounded-lg shadow-lg">
        // <form class="flex gap-4 items-end" on:submit=greet>
        // <div class="flex-1">
        // <label for="greet-input" class="block mb-2 text-sm font-medium text-gray-700">
        // "Enter your name"
        // </label>
        // <input
        // id="greet-input"
        // placeholder="Enter a name..."
        // on:input=update_name
        // class="py-2 px-3 w-full rounded-md border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:outline-none"
        // />
        // </div>
        // <button
        // type="submit"
        // class="py-2 px-6 font-semibold text-white bg-blue-600 rounded-md shadow-sm transition-colors hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none"
        // >
        // "Greet"
        // </button>
        // </form>

        // <div class="mt-4">
        // <p class="text-lg font-medium text-green-700">{move || greet_msg.get()}</p>
        // </div>
        // </div>

        // <div class="p-6 bg-white rounded-lg shadow-lg">
        // <h2 class="mb-4 text-xl font-semibold text-gray-800">"Whisper Models"</h2>
        // <button
        // class="py-2 px-4 mb-4 font-medium text-white bg-indigo-600 rounded-md shadow-sm transition-colors hover:bg-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none"
        // on:click=get_available_models
        // >
        // "Get Available Models"
        // </button>

        // <div class="space-y-2">
        // <For each=move || available_models.get() key=|model| model.name.clone() let:model>
        // <div class="flex justify-between items-center p-3 bg-gray-50 rounded-md border">
        // <div>
        // <span class="font-medium text-gray-800">{model.name.clone()}</span>
        // <span class="ml-2 text-sm text-gray-500">{format!("({})", model.size)}</span>
        // </div>
        // <div class="flex items-center">
        // {if model.downloaded {
        // view! {
        // <span class="inline-flex items-center py-0.5 px-2.5 text-xs font-medium text-green-800 bg-green-100 rounded-full">
        // "Downloaded"
        // </span>
        // }
        // } else {
        // view! {
        // <span class="inline-flex items-center py-0.5 px-2.5 text-xs font-medium text-yellow-800 bg-yellow-100 rounded-full">
        // "Not Downloaded"
        // </span>
        // }
        // }}
        // </div>
        // </div>
        // </For>
        // </div>
        // </div>
        // </div>
        </main>
    }
}
