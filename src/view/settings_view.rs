use leptos::ev::MouseEvent;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;
use std::path::PathBuf;

use crate::constants::LANGUAGES;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn open(obj: JsValue) -> JsValue;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    Srt,
    Txt,
    Json,
    Vtt,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Srt => "srt",
            OutputFormat::Txt => "txt",
            OutputFormat::Json => "json",
            OutputFormat::Vtt => "vtt",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            OutputFormat::Srt => "SRT (SubRip)",
            OutputFormat::Txt => "Plain Text",
            OutputFormat::Json => "JSON",
            OutputFormat::Vtt => "VTT (WebVTT)",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranscriptionSettings {
    pub language: Option<String>,
    pub model: String,
    pub output_format: OutputFormat,
    pub keep_wav: bool,
    pub output_dir: Option<PathBuf>,
    pub parallel_jobs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub name: String,
    pub size: String,
    pub url: String,
    pub downloaded: bool,
    pub file_path: Option<PathBuf>,
    pub progress: Option<f32>,
}

// Common languages for Whisper


#[component]
pub fn SettingsView() -> impl IntoView {
    let (settings, set_settings) = signal(None::<TranscriptionSettings>);
    let (models, set_models) = signal(Vec::<WhisperModel>::new());
    let (loading, set_loading) = signal(true);
    let (saving, set_saving) = signal(false);
    let (error_message, set_error_message) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);

    // Load settings and models on component mount
    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            
            // Load settings
            match load_settings_from_backend().await {
                Ok(loaded_settings) => {
                    set_settings.set(Some(loaded_settings));
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Failed to load settings: {}", e)));
                }
            }
            
            // Load available models
            match get_available_models_from_backend().await {
                Ok(available_models) => {
                    set_models.set(available_models);
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Failed to load models: {}", e)));
                }
            }
            
            set_loading.set(false);
        });
    });

    let on_save = move |ev: SubmitEvent| {
        ev.prevent_default();
        
        if let Some(current_settings) = settings.get() {
            set_saving.set(true);
            set_error_message.set(None);
            set_success_message.set(None);
            
            spawn_local(async move {
                match save_settings_to_backend(current_settings).await {
                    Ok(_) => {
                        set_success_message.set(Some("Settings saved successfully!".to_string()));
                    }
                    Err(e) => {
                        set_error_message.set(Some(format!("Failed to save settings: {}", e)));
                    }
                }
                set_saving.set(false);
            });
        }
    };

    let on_select_output_dir = move |_: MouseEvent| {
        spawn_local(async move {
            match select_directory().await {
                Ok(Some(dir)) => {
                    if let Some(mut current_settings) = settings.get() {
                        current_settings.output_dir = Some(PathBuf::from(dir));
                        set_settings.set(Some(current_settings));
                    }
                }
                Ok(None) => {
                    // User cancelled
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Failed to select directory: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="p-6 mx-auto max-w-4xl bg-white">
            <h1 class="mb-8 text-3xl font-bold text-gray-900">Settings</h1>

            <Show
                when=move || loading.get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || settings.get().is_some()
                            fallback=|| {
                                view! {
                                    <div class="py-8 text-center">
                                        <p class="text-gray-500">Failed to load settings</p>
                                    </div>
                                }
                            }
                        >
                            {move || {
                                let current_settings = settings.get().unwrap();
                                let settings_clone = current_settings.clone();
                                let lang_value = settings_clone.language.as_deref().unwrap_or("auto").to_owned();
                                let output_dir_clone = settings_clone.output_dir.clone();
                                let has_output_dir = output_dir_clone.is_some();
                                view! {
                                    <form on:submit=on_save class="space-y-8">
                                        // Model Selection
                                        <div class="p-6 bg-gray-50 rounded-lg">
                                            <h2 class="mb-4 text-xl font-semibold text-gray-900">Model Settings</h2>

                                            <div class="space-y-4">
                                                <div>
                                                    <label
                                                        for="model"
                                                        class="block mb-2 text-sm font-medium text-gray-700"
                                                    >
                                                        Whisper Model
                                                    </label>
                                                    <select
                                                        id="model"
                                                        class="py-2 px-3 w-full rounded-md border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 focus:outline-none"
                                                        prop:value=move || current_settings.model.clone()
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            if let Some(mut settings) = settings.get() {
                                                                settings.model = value;
                                                                set_settings.set(Some(settings));
                                                            }
                                                        }
                                                    >
                                                        <For
                                                            each=move || models.get()
                                                            key=|model| model.name.clone()
                                                            children=move |model| {
                                                                let model_name = model.name.clone();
                                                                let display_name = if model.downloaded {
                                                                    format!("{} ({}) ✓", model.name, model.size)
                                                                } else {
                                                                    format!("{} ({}) - Not Downloaded", model.name, model.size)
                                                                };

                                                                view! { <option value=model_name>{display_name}</option> }
                                                            }
                                                        />
                                                    </select>
                                                    <p class="mt-1 text-xs text-gray-500">
                                                        Choose the Whisper model. Larger models are more accurate but slower.
                                                    </p>
                                                </div>
                                            </div>
                                        </div>

                                        // Language and Output Settings
                                        <div class="p-6 bg-gray-50 rounded-lg">
                                            <h2 class="mb-4 text-xl font-semibold text-gray-900">Language & Output</h2>

                                            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                                                <div>
                                                    <label
                                                        for="language"
                                                        class="block mb-2 text-sm font-medium text-gray-700"
                                                    >
                                                        Language
                                                    </label>
                                                    <select
                                                        id="language"
                                                        class="py-2 px-3 w-full rounded-md border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 focus:outline-none"
                                                        prop:value=lang_value
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            if let Some(mut settings) = settings.get() {
                                                                settings.language = if value == "auto" {
                                                                    None
                                                                } else {
                                                                    Some(value)
                                                                };
                                                                set_settings.set(Some(settings));
                                                            }
                                                        }
                                                    >
                                                        <For
                                                            each=|| LANGUAGES.iter().cloned()
                                                            key=|(code, _)| code.to_string()
                                                            children=|(code, name)| {
                                                                view! { <option value=code>{name}</option> }
                                                            }
                                                        />
                                                    </select>
                                                </div>

                                                <div>
                                                    <label
                                                        for="output_format"
                                                        class="block mb-2 text-sm font-medium text-gray-700"
                                                    >
                                                        Output Format
                                                    </label>
                                                    <select
                                                        id="output_format"
                                                        class="py-2 px-3 w-full rounded-md border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 focus:outline-none"
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            if let Some(mut settings) = settings.get() {
                                                                settings.output_format = match value.as_str() {
                                                                    "Srt" => OutputFormat::Srt,
                                                                    "Txt" => OutputFormat::Txt,
                                                                    "Json" => OutputFormat::Json,
                                                                    "Vtt" => OutputFormat::Vtt,
                                                                    _ => OutputFormat::Srt,
                                                                };
                                                                set_settings.set(Some(settings));
                                                            }
                                                        }
                                                    >
                                                        <option
                                                            value="Srt"
                                                            selected=move || {
                                                                current_settings.output_format == OutputFormat::Srt
                                                            }
                                                        >
                                                            {OutputFormat::Srt.display_name()}
                                                        </option>
                                                        <option
                                                            value="Txt"
                                                            selected=move || {
                                                                current_settings.output_format == OutputFormat::Txt
                                                            }
                                                        >
                                                            {OutputFormat::Txt.display_name()}
                                                        </option>
                                                        <option
                                                            value="Json"
                                                            selected=move || {
                                                                current_settings.output_format == OutputFormat::Json
                                                            }
                                                        >
                                                            {OutputFormat::Json.display_name()}
                                                        </option>
                                                        <option
                                                            value="Vtt"
                                                            selected=move || {
                                                                current_settings.output_format == OutputFormat::Vtt
                                                            }
                                                        >
                                                            {OutputFormat::Vtt.display_name()}
                                                        </option>
                                                    </select>
                                                </div>
                                            </div>
                                        </div>

                                        // Processing Settings
                                        <div class="p-6 bg-gray-50 rounded-lg">
                                            <h2 class="mb-4 text-xl font-semibold text-gray-900">
                                                Processing Settings
                                            </h2>

                                            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                                                <div>
                                                    <label
                                                        for="parallel_jobs"
                                                        class="block mb-2 text-sm font-medium text-gray-700"
                                                    >
                                                        Parallel Jobs
                                                    </label>
                                                    <input
                                                        type="number"
                                                        id="parallel_jobs"
                                                        min="1"
                                                        max="8"
                                                        class="py-2 px-3 w-full rounded-md border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 focus:outline-none"
                                                        prop:value=move || current_settings.parallel_jobs.to_string()
                                                        on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            if let Ok(num) = value.parse::<usize>() {
                                                                if let Some(mut settings) = settings.get() {
                                                                    settings.parallel_jobs = num.max(1).min(8);
                                                                    set_settings.set(Some(settings));
                                                                }
                                                            }
                                                        }
                                                    />
                                                    <p class="mt-1 text-xs text-gray-500">
                                                        Number of files to process simultaneously (1-8)
                                                    </p>
                                                </div>

                                                <div class="flex items-center">
                                                    <input
                                                        type="checkbox"
                                                        id="keep_wav"
                                                        class="w-4 h-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                                                        prop:checked=move || current_settings.keep_wav
                                                        on:change=move |ev| {
                                                            let checked = event_target_checked(&ev);
                                                            if let Some(mut settings) = settings.get() {
                                                                settings.keep_wav = checked;
                                                                set_settings.set(Some(settings));
                                                            }
                                                        }
                                                    />
                                                    <label for="keep_wav" class="block ml-2 text-sm text-gray-900">
                                                        Keep WAV files after transcription
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // Output Directory
                                        <div class="p-6 bg-gray-50 rounded-lg">
                                            <h2 class="mb-4 text-xl font-semibold text-gray-900">Output Directory</h2>

                                            <div class="flex items-center space-x-4">
                                                <div class="flex-1">
                                                    <input
                                                        type="text"
                                                        class="py-2 px-3 w-full bg-gray-100 rounded-md border border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 focus:outline-none"
                                                        prop:value=move || {
                                                            current_settings
                                                                .output_dir
                                                                .as_ref()
                                                                .map(|p| p.to_string_lossy().to_string())
                                                                .unwrap_or_else(|| "Same as input file".to_string())
                                                        }
                                                        readonly
                                                        placeholder="Same as input file"
                                                    />
                                                </div>
                                                <button
                                                    type="button"
                                                    class="py-2 px-4 font-medium text-white bg-blue-600 rounded-md shadow-sm hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none"
                                                    on:click=on_select_output_dir
                                                >
                                                    Browse
                                                </button>
                                                <button
                                                    type="button"
                                                    class="py-2 px-4 font-medium text-white bg-gray-600 rounded-md shadow-sm hover:bg-gray-700 focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 focus:outline-none"
                                                    on:click=move |_| {
                                                        if let Some(mut settings) = settings.get() {
                                                            settings.output_dir = None;
                                                            set_settings.set(Some(settings));
                                                        }
                                                    }
                                                >
                                                    Clear
                                                </button>
                                            </div>
                                            <p class="mt-2 text-xs text-gray-500">
                                                If not specified, transcription files will be saved in the same directory as the input files.
                                            </p>
                                        </div>

                                        // Messages
                                        <Show when=move || error_message.get().is_some()>
                                            <div class="p-4 bg-red-50 rounded-md border border-red-200">
                                                <div class="flex">
                                                    <div class="ml-3">
                                                        <h3 class="text-sm font-medium text-red-800">Error</h3>
                                                        <div class="mt-2 text-sm text-red-700">
                                                            {move || error_message.get().unwrap_or_default()}
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </Show>

                                        <Show when=move || success_message.get().is_some()>
                                            <div class="p-4 bg-green-50 rounded-md border border-green-200">
                                                <div class="flex">
                                                    <div class="ml-3">
                                                        <h3 class="text-sm font-medium text-green-800">Success</h3>
                                                        <div class="mt-2 text-sm text-green-700">
                                                            {move || success_message.get().unwrap_or_default()}
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </Show>

                                        // Save Button
                                        <div class="flex justify-end pt-6 border-t border-gray-200">
                                            <button
                                                type="submit"
                                                class="py-3 px-6 font-medium text-white bg-blue-600 rounded-md shadow-sm hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                                                disabled=move || saving.get()
                                            >
                                                <Show when=move || saving.get() fallback=|| "Save Settings">
                                                    "Saving..."
                                                </Show>
                                            </button>
                                        </div>
                                    </form>
                                }
                            }}
                        </Show>
                    }
                }
            >
                <div class="flex justify-center items-center py-12">
                    <div class="w-8 h-8 rounded-full border-b-2 border-blue-600 animate-spin"></div>
                    <span class="ml-3 text-gray-600">Loading settings...</span>
                </div>
            </Show>
        </div>
    }
}

async fn load_settings_from_backend() -> Result<TranscriptionSettings, String> {
    let result = invoke("load_settings", JsValue::NULL).await;
    
    if let Ok(settings) = serde_wasm_bindgen::from_value(result) {
        Ok(settings)
    } else {
        Err("Failed to deserialize settings".to_string())
    }
}

async fn save_settings_to_backend(settings: TranscriptionSettings) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    let result = invoke("save_settings", args).await;
    
    if result.is_truthy() {
        Ok(())
    } else {
        match serde_wasm_bindgen::from_value(result) {
            Ok(error_msg) => Err(error_msg),
            Err(_) => Err("Failed to save settings".to_string())
        }
    }
}

async fn get_available_models_from_backend() -> Result<Vec<WhisperModel>, String> {
    let result = invoke("get_available_models", JsValue::NULL).await;
    
    if let Ok(models) = serde_wasm_bindgen::from_value(result) {
        Ok(models)
    } else {
        Err("Failed to load available models".to_string())
    }
}

#[derive(Serialize)]
struct DialogOptions {
    directory: bool,
    multiple: bool,
}

async fn select_directory() -> Result<Option<String>, String> {
    let options = DialogOptions {
        directory: true,
        multiple: false,
    };
    
    let options_js = serde_wasm_bindgen::to_value(&options)
        .map_err(|e| format!("Failed to create dialog options: {}", e))?;
    
    let result = open(options_js).await;
    
    if result.is_null() || result.is_undefined() {
        Ok(None)
    } else {
        // result.into_serde::<String>()
        //     .map(Some)
        //     .map_err(|e| format!("Failed to parse selected directory: {}", e))
        serde_wasm_bindgen::from_value(result).unwrap()
    }
}
