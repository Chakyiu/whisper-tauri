use leptos::ev::MouseEvent;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_meta::provide_meta_context;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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

    let update_name = move |ev| {
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
        });
    };

    view! {
        <main class="container mx-auto p-8 min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
            <div class="max-w-4xl mx-auto">
                <h1 class="text-4xl font-bold text-center text-gray-800 mb-8">
                    "Welcome to Tauri + Leptos"
                </h1>

                <div class="flex justify-center items-center gap-8 mb-12">
                    <a
                        href="https://tauri.app"
                        target="_blank"
                        class="transition-transform hover:scale-110 hover:drop-shadow-lg"
                    >
                        <img
                            src="public/tauri.svg"
                            class="w-24 h-24"
                            alt="Tauri logo"
                        />
                    </a>
                    <a
                        href="https://docs.rs/leptos/"
                        target="_blank"
                        class="transition-transform hover:scale-110 hover:drop-shadow-lg"
                    >
                        <img
                            src="public/leptos.svg"
                            class="w-24 h-24"
                            alt="Leptos logo"
                        />
                    </a>
                </div>

                <p class="text-center text-gray-600 mb-8">
                    "Click on the Tauri and Leptos logos to learn more."
                </p>

                <div class="bg-white rounded-lg shadow-lg p-6 mb-6">
                    <form class="flex gap-4 items-end" on:submit=greet>
                        <div class="flex-1">
                            <label for="greet-input" class="block text-sm font-medium text-gray-700 mb-2">
                                "Enter your name"
                            </label>
                            <input
                                id="greet-input"
                                placeholder="Enter a name..."
                                on:input=update_name
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            />
                        </div>
                        <button
                            type="submit"
                            class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-md shadow-sm transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                        >
                            "Greet"
                        </button>
                    </form>

                    <div class="mt-4">
                        <p class="text-lg font-medium text-green-700">{move || greet_msg.get()}</p>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6">
                    <h2 class="text-xl font-semibold text-gray-800 mb-4">"Whisper Models"</h2>
                    <button
                        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white font-medium rounded-md shadow-sm transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 mb-4"
                        on:click=get_available_models
                    >
                        "Get Available Models"
                    </button>

                    <div class="space-y-2">
                        <For
                            each=move || available_models.get()
                            key=|model| model.name.clone()
                            let:model
                        >
                            <div class="flex items-center justify-between p-3 bg-gray-50 rounded-md border">
                                <div>
                                    <span class="font-medium text-gray-800">{model.name.clone()}</span>
                                    <span class="ml-2 text-sm text-gray-500">{format!("({})", model.size)}</span>
                                </div>
                                <div class="flex items-center">
                                    {if model.downloaded {
                                        view! {
                                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                                                "Downloaded"
                                            </span>
                                        }
                                    } else {
                                        view! {
                                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                                                "Not Downloaded"
                                            </span>
                                        }
                                    }}
                                </div>
                            </div>
                        </For>
                    </div>
                </div>
            </div>
        </main>
    }
}
