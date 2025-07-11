use std::sync::Mutex;

use log::info;
use tauri::{
  Manager,
  Runtime,
  plugin::{Builder as PluginBuilder, TauriPlugin},
};

use crate::app::models::new_whisper_processor;

#[derive(Default)]
struct AppDataWhisperProcessor {
  whisper_processor: Option<crate::app::models::whisper::Processor>,
}

#[tauri::command]
pub async fn load_model_whisper<R: Runtime>(
  app: tauri::AppHandle<R>,
  window: tauri::WebviewWindow<R>,
) -> Result<(), String> {
  info!("Loading models...");

  // Load the traditional whisper models first
  match new_whisper_processor(window) {
    Ok(p) => {
      let data = app.state::<Mutex<AppDataWhisperProcessor>>();
      let mut data = data.lock().unwrap();
      data.whisper_processor = Some(p);
      info!("Whisper model loaded successfully");
    },
    Err(e) => {
      let error_message = format!("Failed to load Whisper model: {}", e);
      info!("{}", error_message);
      return Err(error_message);
    },
  }

  info!("All models loaded successfully");
  Ok(())
}

#[tauri::command]
pub async fn audio_transcription<R: Runtime>(
  app: tauri::AppHandle<R>,
  chunk: Vec<f32>,
) -> Result<String, String> {
  info!("Processing audio transcription...");

  let data = app.state::<Mutex<AppDataWhisperProcessor>>();

  // Check if processor exists first
  {
    let data = data.lock().unwrap();
    if data.whisper_processor.is_none() {
      return Err("Whisper model is not loaded".to_string());
    }
  }

  // Then mutable borrow
  let mut data = data.lock().unwrap();
  let processor = data.whisper_processor.as_mut().unwrap();

  let transcription = processor
    .transcribe(chunk.as_slice())
    .map_err(|e| e.to_string())?;

  Ok(transcription)
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  PluginBuilder::new("proj-airi-tauri-plugin-audio-transcription")
    .setup(|app, _| {
      info!("Initializing audio transcription plugin...");
      app.manage(Mutex::new(AppDataWhisperProcessor::default()));
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      load_model_whisper,
      audio_transcription,
    ])
    .build()
}
