use futures_util::StreamExt;
use obws::events::Event;
use obws::requests::inputs::{InputId, Volume};
use obws::requests::scene_items::SetEnabled;
use obws::Client;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

pub struct ObsState {
    client: Arc<Mutex<Option<Arc<Client>>>>,
    rt: Runtime,
    pub event_tx: broadcast::Sender<Event>,
}

impl ObsState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            client: Arc::new(Mutex::new(None)),
            rt: Runtime::new().unwrap(),
            event_tx: tx,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.lock().unwrap().is_some()
    }

    pub fn connect(&self, host: &str, port: u16, password: Option<String>) -> Result<(), String> {
        let host = host.to_string();
        let client_clone = self.client.clone();
        let event_tx = self.event_tx.clone();

        self.rt.block_on(async move {
            let connect_future = Client::connect(host, port, password);
            match tokio::time::timeout(std::time::Duration::from_secs(3), connect_future).await {
                Ok(Ok(c)) => {
                    if let Ok(events) = c.events() {
                        tokio::spawn(async move {
                            let mut events = events;
                            while let Some(event) = events.next().await {
                                let _ = event_tx.send(event);
                            }
                        });
                    }

                    let mut guard = client_clone.lock().unwrap();
                    *guard = Some(Arc::new(c));
                    Ok(())
                }
                Ok(Err(e)) => Err(format!("OBS: {}", e)),
                Err(_) => Err("OBS Timeout: Server nicht erreichbar".to_string()),
            }
        })
    }

    pub fn execute(&self, action: &str, target: Option<String>) -> Result<(), String> {
        let client_opt = self.client.lock().unwrap().clone();
        let client = client_opt.ok_or("OBS Offline")?;
        let target_str = target.unwrap_or_default();
        let action = action.to_string();

        self.rt.spawn(async move {
            match action.as_str() {
                "SetScene" => {
                    let _ = client
                        .scenes()
                        .set_current_program_scene(target_str.as_str())
                        .await;
                }
                "SetPreviewScene" => {
                    let _ = client
                        .scenes()
                        .set_current_preview_scene(target_str.as_str())
                        .await;
                }
                "ToggleStudioMode" => {
                    if let Ok(enabled) = client.ui().studio_mode_enabled().await {
                        let _ = client.ui().set_studio_mode_enabled(!enabled).await;
                    }
                }
                "Transition" => {
                    let _ = client.transitions().trigger().await;
                }
                "ToggleMute" => {
                    let _ = client
                        .inputs()
                        .toggle_mute(obws::requests::inputs::InputId::Name(target_str.as_str()))
                        .await;
                }
                "StartStopStream" => {
                    let _ = client.streaming().toggle().await;
                }
                "StartStopRecord" => {
                    let _ = client.recording().toggle().await;
                }
                "ToggleSource" => {
                    // target_str format: "SceneName|SourceName"
                    let parts: Vec<&str> = target_str.split('|').collect();
                    if parts.len() == 2 {
                        if let Ok(items) = client
                            .scene_items()
                            .list(obws::requests::canvases::SceneId::Name(parts[0]))
                            .await
                        {
                            if let Some(item) =
                                items.into_iter().find(|i| i.source_name == parts[1])
                            {
                                if let Ok(enabled) = client
                                    .scene_items()
                                    .enabled(
                                        obws::requests::canvases::SceneId::Name(parts[0]),
                                        item.id,
                                    )
                                    .await
                                {
                                    let _ = client
                                        .scene_items()
                                        .set_enabled(SetEnabled {
                                            scene: obws::requests::canvases::SceneId::Name(
                                                parts[0],
                                            ),
                                            item_id: item.id,
                                            enabled: !enabled,
                                        })
                                        .await;
                                }
                            }
                        }
                    }
                }
                "SetVolume" => {
                    // target_str format: "InputName|VolumePct"
                    let parts: Vec<&str> = target_str.split('|').collect();
                    if parts.len() == 2 {
                        if let Ok(vol) = parts[1].parse::<f32>() {
                            let pct = vol.clamp(0.1, 100.0) / 100.0;
                            let _ = client
                                .inputs()
                                .set_volume(InputId::Name(parts[0]), Volume::Mul(pct))
                                .await;
                        }
                    }
                }
                "ToggleFilter" => {
                    // target_str format: "SourceName|FilterName"
                    let parts: Vec<&str> = target_str.split('|').collect();
                    if parts.len() == 2 {
                        if let Ok(filter) = client
                            .filters()
                            .get(obws::requests::sources::SourceId::Name(parts[0]), parts[1])
                            .await
                        {
                            let _ = client
                                .filters()
                                .set_enabled(obws::requests::filters::SetEnabled {
                                    source: obws::requests::sources::SourceId::Name(parts[0]),
                                    filter: parts[1],
                                    enabled: !filter.enabled,
                                })
                                .await;
                        }
                    }
                }
                "SetSourceVisible" => {
                    // target_str format: "SceneName|SourceName|1/0"
                    let parts: Vec<&str> = target_str.split('|').collect();
                    if parts.len() == 3 {
                        if let Ok(items) = client
                            .scene_items()
                            .list(obws::requests::canvases::SceneId::Name(parts[0]))
                            .await
                        {
                            if let Some(item) =
                                items.into_iter().find(|i| i.source_name == parts[1])
                            {
                                let _ = client
                                    .scene_items()
                                    .set_enabled(SetEnabled {
                                        scene: obws::requests::canvases::SceneId::Name(parts[0]),
                                        item_id: item.id,
                                        enabled: parts[2] == "1",
                                    })
                                    .await;
                            }
                        }
                    }
                }
                "ReplayBuffer" => match target_str.as_str() {
                    "toggle" => {
                        let _ = client.replay_buffer().toggle().await;
                    }
                    "save" => {
                        let _ = client.replay_buffer().save().await;
                    }
                    _ => {}
                },
                _ => {}
            }
        });
        Ok(())
    }

    pub fn get_scenes(&self) -> Result<Vec<String>, String> {
        let client_lock = self.client.lock().unwrap();
        let client = client_lock.as_ref().ok_or("OBS Offline")?;

        self.rt.block_on(async {
            let scenes = client.scenes().list().await.map_err(|e| e.to_string())?;
            Ok(scenes.scenes.into_iter().map(|s| s.id.name).collect())
        })
    }

    pub fn get_inputs(&self) -> Result<Vec<String>, String> {
        let client_lock = self.client.lock().unwrap();
        let client = client_lock.as_ref().ok_or("OBS Offline")?;

        self.rt.block_on(async {
            let inputs = client
                .inputs()
                .list(None)
                .await
                .map_err(|e| e.to_string())?;
            Ok(inputs.into_iter().map(|i| i.id.name).collect())
        })
    }

    pub fn get_sources(&self, scene: &str) -> Result<Vec<String>, String> {
        let client_lock = self.client.lock().unwrap();
        let client = client_lock.as_ref().ok_or("OBS Offline")?;

        self.rt.block_on(async {
            let items = client
                .scene_items()
                .list(obws::requests::canvases::SceneId::Name(scene))
                .await
                .map_err(|e| e.to_string())?;
            Ok(items.into_iter().map(|i| i.source_name).collect())
        })
    }

    pub fn get_filters(&self, source: &str) -> Result<Vec<String>, String> {
        let client_lock = self.client.lock().unwrap();
        let client = client_lock.as_ref().ok_or("OBS Offline")?;

        self.rt.block_on(async {
            let filters = client
                .filters()
                .list(obws::requests::sources::SourceId::Name(source))
                .await
                .map_err(|e| e.to_string())?;
            Ok(filters.into_iter().map(|f| f.name).collect())
        })
    }

    pub async fn resolve_scene_item_name(
        &self,
        scene: &str,
        item_id: i64,
    ) -> Result<String, String> {
        let client_opt = self.client.lock().unwrap().clone();
        let client = client_opt.ok_or("OBS Offline")?;

        let items = client
            .scene_items()
            .list(obws::requests::canvases::SceneId::Name(scene))
            .await
            .map_err(|e| e.to_string())?;
        if let Some(item) = items.into_iter().find(|i| i.id == item_id) {
            Ok(item.source_name)
        } else {
            Err("Item not found".to_string())
        }
    }
}
