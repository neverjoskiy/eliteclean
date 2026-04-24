//! Noxum Launcher - Rust Core Library
//! Миграция с Python/FastAPI на Rust/Tauri v2

pub mod commands;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

// Windows-specific memory manipulation module
#[cfg(windows)]
pub mod memory;
