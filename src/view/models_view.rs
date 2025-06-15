use leptos::task::spawn_local;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}


#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct WhisperModel {
    name: String,
    size: String,
    url: String,
    downloaded: bool,
    file_path: Option<String>,
    progress: Option<i32>
}

#[derive(Serialize, Deserialize)]
struct DownloadModelArgs<'a> {
    model_name: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadModelClosure {
    model: String,
    progress: i32
}

#[derive(Serialize, Deserialize, Debug)]
struct TauriEvent {
    payload: DownloadModelClosure,
}

#[derive(Serialize, Deserialize, Debug)]
struct TauriEventString {
    payload: String,
}

#[component]
pub fn ModelsView() -> impl IntoView {
    let (available_models, set_available_models): (
        ReadSignal<Vec<WhisperModel>>,
        WriteSignal<Vec<WhisperModel>>,
    ) = signal(Vec::new());

    let get_available_models = move || {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let result = invoke("get_available_models", args).await;
            let models: Vec<WhisperModel> = serde_wasm_bindgen::from_value(result).unwrap();
            set_available_models.set(models);
        });
    };

    let download_models = move |model_name: String| {
        spawn_local(async move {
            let download_model_args = DownloadModelArgs { model_name: &model_name };
            let args = serde_wasm_bindgen::to_value(&download_model_args).unwrap();
            invoke("download_model", args).await;
        })
    };

    get_available_models();

    spawn_local(async move {
        let closure = Closure::<dyn FnMut(_)>::new(move |s: JsValue| {
            log::info!("Raw event data: {:?}", s);
            
            match serde_wasm_bindgen::from_value::<TauriEvent>(s.clone()) {
                Ok(event) => {
                    log::info!("Progress parsed as Tauri event: {:?}", event.payload);
                    let progress = event.payload;
                    
                    // Update the model progress
                    set_available_models.update(|models| {
                        if let Some(model) = models.iter_mut().find(|m| m.name == progress.model) {
                            model.progress = Some(progress.progress); 
                            if progress.progress >= 100{
                                model.downloaded = true;
                            }
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to parse progress event: {:?}", e);
                }
            }
        });
        listen("model-download-progress", closure.as_ref().unchecked_ref()).await;
        closure.forget();
    });

    spawn_local(async move {
        let completion_closure = Closure::<dyn FnMut(_)>::new(move |s: JsValue| {
            log::info!("Download complete event: {:?}", s);
            
            match serde_wasm_bindgen::from_value::<TauriEventString>(s.clone()) {
                Ok(event) => {
                    log::info!("Model download completed (wrapped): {}", event.payload);
                    let model_name = event.payload;
                    
                    // Update the model state to mark as downloaded
                    set_available_models.update(|models| {
                        if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                            model.downloaded = true;
                            model.progress = Some(100);
                        }
                    });

                    log::info!("updated model: {:?}", available_models.get());
                }
                Err(e) => {
                    log::error!("Failed to parse completion event: {:?}", e);
                }
            }
        });
        listen("model-download-complete", completion_closure.as_ref().unchecked_ref()).await;
        completion_closure.forget();
    });

    view! {
        <div class="p-6">
            <div class="bg-white rounded-lg divide-y divide-gray-100 shadow-sm dark:bg-gray-700 w-100">
                <For
                    each=move || available_models.get()
                    key=|model| format!("{:?}-{:?}", model.name.clone(), model.progress)
                    children=move |model| {
                        view! {
                            <ul class="py-2 text-sm text-gray-700 dark:text-gray-200">
                                <li class="flex justify-between">
                                    <div class="mx-6">
                                        <a class="block py-2 text-lg">
                                            <strong>{model.name.clone()}</strong>
                                        </a>
                                        <a class="block py-2 text-xs">Size: {model.size.clone()}</a>
                                        {move || {
                                            if let Some(progress) = model.progress {
                                                if progress < 100 {
                                                    view! {
                                                        <div class="mt-2 w-full h-2.5 bg-gray-200 rounded-full dark:bg-gray-700">
                                                            <div
                                                                class="h-2.5 bg-blue-600 rounded-full"
                                                                style:width=format!("{}%", progress)
                                                            ></div>
                                                        </div>
                                                        <span class="text-xs text-gray-500 dark:text-gray-400">
                                                            {format!("{}%", progress)}
                                                        </span>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }
                                        }}
                                    </div>
                                    <div class="content-center mx-4">
                                        <button
                                            type="button"
                                            class="py-3 px-5 mb-2 text-sm font-medium text-white rounded-lg dark:bg-blue-600 focus:ring-4 focus:outline-none me-2"
                                            class=(
                                                [
                                                    "bg-blue-700",
                                                    "hover:bg-blue-800",
                                                    "focus:ring-blue-300",
                                                    "dark:hover:bg-blue-700",
                                                    "dark:focus:ring-blue-800",
                                                ],
                                                move || !model.downloaded && model.progress.is_none(),
                                            )
                                            class=(
                                                [
                                                    "bg-yellow-600",
                                                    "hover:bg-yellow-700",
                                                    "focus:ring-yellow-300",
                                                    "dark:hover:bg-yellow-700",
                                                    "dark:focus:ring-yellow-800",
                                                ],
                                                move || model.progress.is_some() && model.progress.unwrap_or(0) < 100,
                                            )
                                            class=(["bg-green-700", "focus:ring-green-300"], move || model.downloaded)
                                            disabled=move || {
                                                model.progress.is_some() && model.progress.unwrap_or(0) < 100
                                            }
                                            on:click=move |_| { download_models(model.name.clone()) }
                                        >
                                            {move || {
                                                if model.downloaded {
                                                    "Downloaded".to_string()
                                                } else if let Some(progress) = model.progress {
                                                    if progress < 100 {
                                                        "Downloading...".to_string()
                                                    } else {
                                                        "Download".to_string()
                                                    }
                                                } else {
                                                    "Download".to_string()
                                                }
                                            }}
                                        </button>
                                    </div>
                                </li>
                            </ul>
                        }
                    }
                />
            </div>
        </div>
    }
}
