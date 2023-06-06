use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use egui::{pos2, Color32, Context, Mesh, Rect, Vec2};
use egui_extras::RetainedImage;
use reqwest::header::USER_AGENT;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Id {
    x: u32,
    y: u32,
    zoom: u8,
}

#[derive(Clone)]
pub struct Tile {
    image: Arc<RetainedImage>,
}

impl Tile {
    fn new(image: &[u8]) -> Self {
        Self {
            image: Arc::new(RetainedImage::from_image_bytes("debug_name", &image).unwrap()),
        }
    }

    pub fn rect(&self, screen_position: Vec2) -> Rect {
        let tile_size = pos2(self.image.width() as f32, self.image.height() as f32);
        Rect::from_two_pos(
            screen_position.to_pos2(),
            (screen_position + tile_size.to_vec2()).to_pos2(),
        )
    }

    pub fn mesh(&self, screen_position: Vec2, ctx: &Context) -> Mesh {
        let tile_size = pos2(self.image.width() as f32, self.image.height() as f32);
        let mut mesh = Mesh::with_texture(self.image.texture_id(ctx));
        mesh.add_rect_with_uv(
            Rect::from_two_pos(
                screen_position.to_pos2(),
                (screen_position + tile_size.to_vec2()).to_pos2(),
            ),
            Rect::from_min_max(pos2(0., 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        mesh
    }
}

#[derive(Clone)]
pub struct Tiles {
    cache: Arc<Mutex<HashMap<Id, Tile>>>,
    requests: tokio::sync::mpsc::Sender<Id>,
}

async fn download(
    mut requests: tokio::sync::mpsc::Receiver<Id>,
    cache: Arc<Mutex<HashMap<Id, Tile>>>,
) {
    loop {
        if let Some(requested) = requests.recv().await {
            log::debug!("Tile requested: {:?}.", requested);

            // Might have been downloaded before this request was received from
            // the requests queue.
            if cache.lock().unwrap().contains_key(&requested) {
                continue;
            }

            let url = format!(
                "https://tile.openstreetmap.org/{}/{}/{}.png",
                requested.zoom, requested.x, requested.y
            );
            let image = reqwest::Client::new()
                .get(url)
                .header(USER_AGENT, "Widnet/1.0")
                .send()
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap();

            cache.lock().unwrap().insert(requested, Tile::new(&image));
        }
    }
}

impl Tiles {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(5);
        let cache = Arc::new(Mutex::new(HashMap::<Id, Tile>::new()));
        runtime.spawn(download(rx, cache.clone()));
        Self {
            cache,
            requests: tx,
        }
    }

    pub fn at(&self, x: u32, y: u32, zoom: u8) -> Option<Tile> {
        let tile = Id { x, y, zoom };

        self.cache.lock().unwrap().get(&tile).cloned().or_else(|| {
            if let Err(error) = self.requests.try_send(tile) {
                log::debug!("Could not request a tile: {:?}, reason: {}", tile, error);
            };

            // Tile was requested, but we don't have it yet.
            None
        })
    }
}
