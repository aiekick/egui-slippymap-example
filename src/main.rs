mod map;
mod mercator;
mod tiles;
mod tokio;

use crate::tokio::TokioRuntimeThread;
use eframe::NativeOptions;
use egui::CentralPanel;
use mercator::Position;
use tiles::Tiles;

struct SlippyMapExample {
    my_position: Position,
    tiles: tiles::Tiles,

    #[allow(dead_code)] // Significant Drop
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.hyperlink("https://www.openstreetmap.org");
            map::ui(self.my_position, ctx, ui, "map", &mut self.tiles);
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eframe::run_native(
        "egui slippy map example",
        NativeOptions::default(),
        Box::new(|_cc| Box::new(SlippyMapExample::new())),
    )
    .map_err(|e| e.into())
}
