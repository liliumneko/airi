use std::sync::Mutex;

use log::info;
use tauri::{
  Manager,
  Runtime,
  plugin::{Builder as PluginBuilder, TauriPlugin},
};

use crate::app::models::new_silero_vad_processor;

#[derive(Default)]
struct AppDataSileroVadProcessor {
  silero_vad_processor: Option<crate::app::models::silero_vad::Processor>,
}

#[tauri::command]
pub async fn load_model_silero_vad<R: Runtime>(
  app: tauri::AppHandle<R>,
  window: tauri::WebviewWindow<R>,
) -> Result<(), String> {
  info!("Loading models...");

  match new_silero_vad_processor(window) {
    Ok(p) => {
      let data = app.state::<Mutex<AppDataSileroVadProcessor>>();
      let mut data = data.lock().unwrap();
      data.silero_vad_processor = Some(p);
      info!("Silero VAD model loaded successfully");
    },
    Err(e) => {
      let error_message = format!("Failed to load Silero VAD model: {}", e);
      info!("{}", error_message);
      return Err(error_message);
    },
  }

  info!("All models loaded successfully");
  Ok(())
}

#[tauri::command]
pub async fn audio_vad<R: Runtime>(
  app: tauri::AppHandle<R>,
  chunk: Vec<f32>,
) -> Result<f32, String> {
  let data = app.state::<Mutex<AppDataSileroVadProcessor>>();

  // Check if processor exists first
  {
    let data = data.lock().unwrap();
    if data.silero_vad_processor.is_none() {
      return Err("Silero VAD model is not loaded".to_string());
    }
  }

  // Then mutable borrow
  let mut data = data.lock().unwrap();
  let processor = data.silero_vad_processor.as_mut().unwrap();

  let speech_prob = processor
    .process_chunk(chunk.as_slice())
    .map_err(|e| e.to_string())?;

  Ok(speech_prob)
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  PluginBuilder::new("proj-airi-tauri-plugin-audio-vad")
    .setup(|app, _| {
      info!("Initializing audio VAD plugin...");
      app.manage(Mutex::new(AppDataSileroVadProcessor::default()));
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![load_model_silero_vad, audio_vad])
    .build()
}
