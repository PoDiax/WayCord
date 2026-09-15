use std::{
    collections::HashMap,
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

#[derive(Clone)]
pub struct AvatarCache {
    images: Arc<RwLock<HashMap<String, Arc<egui::ColorImage>>>>,
    in_flight: Arc<Mutex<HashMap<String, String>>>,
    cache_dir: PathBuf,
}

impl Default for AvatarCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarCache {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dir = PathBuf::from(home).join(".cache/waycord/avatars");
        let _ = std::fs::create_dir_all(&cache_dir);

        Self {
            images: Arc::new(RwLock::new(HashMap::new())),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            cache_dir,
        }
    }

    pub fn get(&self, user_id: &str) -> Option<Arc<egui::ColorImage>> {
        if let Ok(map) = self.images.read() {
            if let Some(img) = map.get(user_id) {
                return Some(img.clone());
            }
        }

        let disk_path = self.cache_dir.join(format!("{user_id}.png"));
        if disk_path.exists() {
            if let Ok(bytes) = std::fs::read(&disk_path) {
                if let Ok(color_image) = Self::decode_bytes(&bytes) {
                    let arc_img = Arc::new(color_image);
                    if let Ok(mut map) = self.images.write() {
                        map.insert(user_id.to_string(), arc_img.clone());
                    }
                    return Some(arc_img);
                }
            }
        }

        None
    }

    pub fn queue_download(&self, user_id: String, url: String) {
        if self.get(&user_id).is_some() {
            return;
        }

        let mut in_flight = self.in_flight.lock().unwrap();
        if in_flight.contains_key(&user_id) {
            return;
        }
        in_flight.insert(user_id.clone(), url.clone());
        drop(in_flight);

        let images = self.images.clone();
        let in_flight = self.in_flight.clone();
        let disk_path = self.cache_dir.join(format!("{user_id}.png"));

        thread::spawn(move || {
            let result = ureq::get(&url)
                .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko)")
                .timeout(Duration::from_secs(6))
                .call();

            match result {
                Ok(resp) => {
                    let mut bytes = Vec::new();
                    if let Ok(_) = resp.into_reader().read_to_end(&mut bytes) {
                        let _ = std::fs::write(&disk_path, &bytes);
                        match Self::decode_bytes(&bytes) {
                            Ok(color_image) => {
                                if let Ok(mut map) = images.write() {
                                    map.insert(user_id.clone(), Arc::new(color_image));
                                }
                                println!("Avatar cached for user: {} ({})", user_id, url);
                            }
                            Err(e) => {
                                eprintln!("Failed to decode avatar image for {user_id}: {e}");
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch avatar from {url}: {e}");
                }
            }

            if let Ok(mut inflight) = in_flight.lock() {
                inflight.remove(&user_id);
            }
        });
    }

    pub fn decode_bytes(bytes: &[u8]) -> Result<egui::ColorImage, Box<dyn std::error::Error + Send + Sync>> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            rgba.as_raw(),
        );
        Ok(color_image)
    }
}
