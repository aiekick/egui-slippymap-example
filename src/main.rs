mod map;
mod mercator;
mod tiles;
mod tokio;

use crate::tokio::TokioRuntimeThread;
use eframe::NativeOptions;
use mercator::Position;
use tiles::Tiles;

struct SlippyMapExample {
    my_position: Position,
    tiles: tiles::Tiles,
    tokio_thread: tokio::TokioRuntimeThread,
}

impl SlippyMapExample {
    fn new() -> Self {
        let tokio_thread = TokioRuntimeThread::new();

        Self {
            my_position: Position {
                lat: 52.23181,
                lon: 21.00625,
            },
            tiles: Tiles::new(tokio_thread.runtime.clone()),
            tokio_thread,
        }
    }
}

impl eframe::App for SlippyMapExample {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //map::ui(self.my_position, ctx, ui, id_source, follow)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eframe::run_native(
        "egui slippy map example",
        NativeOptions::default(),
        Box::new(|cc| Box::new(SlippyMapExample::new())),
    )
    .map_err(|e| e.into())
}
