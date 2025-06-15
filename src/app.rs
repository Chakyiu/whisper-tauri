use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use wasm_bindgen::prelude::*;

use crate::view::debug_view::DebugView;
use crate::view::models_view::ModelsView;
use crate::view::whisper_view::WhisperView;
use crate::view::settings_view::SettingsView;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let tab_names = vec!["Whisper", "Models", "Settings", "Debug"];
    let (tab, set_tab) = signal(String::from("Models"));
    let change_tab = move |tab: String| {
        set_tab.set(tab);
    };

    view! {
        <main class="p-8 min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 w-100">
            <div class="border-b border-gray-200 dark:border-gray-700">
                <ul class="flex flex-wrap -mb-px text-sm font-medium text-center text-gray-500 dark:text-gray-400">
                    <For
                        each=move || tab_names.clone()
                        key=|tab| tab.to_string()
                        children=move |t| {
                            view! {
                                <li class="me-2" on:click=move |_| change_tab(t.to_string())>
                                    <a
                                        href="#"
                                        class="inline-flex justify-center items-center p-4 rounded-t-lg border-b-2 group"
                                        class=(
                                            [
                                                "border-transparent",
                                                "hover:text-gray-600",
                                                "dark:hover:text-gray-300",
                                                "hover:border-gray-300",
                                            ],
                                            move || !tab.get().eq(t),
                                        )
                                        class=(
                                            [
                                                "text-blue-600",
                                                "border-blue-600",
                                                "hover:text-blue-600",
                                                "dark:text-blue-500",
                                                "dark:border-blue-500",
                                                "dark:hover:text-blue-600",
                                                "active",
                                            ],
                                            move || { tab.get().eq(t) },
                                        )
                                    >
                                        {t}
                                    </a>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
            {move || match tab.get().as_str() {
                "Whisper" => view! { <WhisperView /> }.into_any(),
                "Models" => view! { <ModelsView /> }.into_any(),
                "Settings" => view! { <SettingsView /> }.into_any(),
                "Debug" => view! { <DebugView /> }.into_any(),
                _ => view! { <DebugView /> }.into_any(),
            }}
        </main>
    }
}
