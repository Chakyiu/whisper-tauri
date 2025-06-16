# whisper-tauri

## Introduction

whisper-tauri is an application that provides a simple user interface for transcribing video into various text formats using Whisper AI. The UI makes it easy to manage Whisper models and select output formats. Thanks to whisper-rs, this project integrates seamlessly with whisper.cpp for transcription. Please note that this project is still under maintenance and has only been tested on macOS so far.

## Background

This project serves as my personal learning experience with Rust and Leptos. I noticed there are already many Whisper apps available for Windows, web UIs, or CLI only, but I wanted to create one that is easy to run on macOS. I hope this project is helpful to you, and please feel free to contribute and leave comments.

## To do

- [ ] Whisper View
  - [ ] Drag and Drop
- [x] Whisper Model Selection View
- [ ] Settings View
  - [ ] Model Selection
  - [ ] Language Selection
  - [ ] Output format Selection
  - [ ] Output path Selection
- [ ] Add support to Windows App and Linux

## Will do

- [ ] Add video cutting features
- [ ] Add preprocessing function (e.g. using spleeter to extract vocals)

## How to dev

This project is relying on ffmpeg. For dev and building the project, you should also install cmake

```sh
# In Macos
brew install cmake
```

To run

```sh
# To install Tailwind dependency
npm  install
cargo tauri dev
```

## How to build

```sh
cargo tauri build --release
```
