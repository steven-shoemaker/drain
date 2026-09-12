mod commands;
mod db;
mod process;
mod runner;

use commands::AppState;
use db::Db;
use process::ProcessHub;
use runner::QueueCtl;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let conn = db::open(&dir.join("drain.db"))?;
            app.manage(AppState {
                db: Db(Mutex::new(conn)),
                hub: ProcessHub::new(),
                queue: Arc::new(QueueCtl::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_ideas,
            commands::create_idea,
            commands::get_idea,
            commands::update_idea_drafts,
            commands::update_idea_body,
            commands::refine_idea,
            commands::enqueue_idea,
            commands::list_tasks,
            commands::get_task,
            commands::reorder_tasks,
            commands::start_queue,
            commands::pause_queue,
            commands::queue_paused,
            commands::cancel_run,
            commands::approve_task,
            commands::reject_task,
            commands::retry_task,
            commands::skip_failed,
            commands::get_settings,
            commands::save_settings,
            commands::get_run_log,
            commands::binary_status,
            commands::pick_folder,
            commands::open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Drain");
}
