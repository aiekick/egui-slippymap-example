use std::collections::{hash_map::Entry, HashMap};

use crate::mercator::Position;
use eframe::epaint::CircleShape;
use egui::{
    Align2, Color32, Context, FontId, Id, Mesh, Painter, Pos2, Response, RichText, Sense, Shape,
    Stroke, Ui, Vec2, Window,
};

use crate::{
    mercator::{screen_to_position, PositionExt, TileCoordinates},
    tiles::Tiles,
};

#[derive(Clone, PartialEq)]
pub enum MapCenterMode {
    MyPosition,
    Exact(Position),
}

impl MapCenterMode {
    fn screen_drag(&mut self, response: Response, my_position: Position, zoom: u8) {
        if response.dragged_by(egui::PointerButton::Primary) {
            *self = match *self {
                MapCenterMode::MyPosition => MapCenterMode::Exact(my_position),
                MapCenterMode::Exact(position) => MapCenterMode::Exact({
                    let position_delta = screen_to_position(response.drag_delta(), zoom);
                    Position {
                        lat: position.lat - position_delta.lat,
                        lon: position.lon - position_delta.lon,
                    }
                }),
            };
        }
    }

    fn position(&self, my_position: Position) -> Position {
        match self {
            MapCenterMode::MyPosition => my_position,
            MapCenterMode::Exact(position) => *position,
        }
    }
}

#[derive(Clone)]
struct MapMemory {
    center_mode: MapCenterMode,
    osm: bool,
    zoom: u8,
}

impl Default for MapMemory {
    fn default() -> Self {
        Self {
            center_mode: MapCenterMode::MyPosition,
            osm: false,
            zoom: 16,
        }
    }
}

fn draw_tiles(
    painter: &Painter,
    tile_coordinates: TileCoordinates,
    map_center_projected_position: Pos2,
    zoom: u8,
    screen_center: Pos2,
    tiles: &mut Tiles,
    ui: &mut Ui,
    meshes: &mut HashMap<TileCoordinates, Mesh>,
) {
    let tile_projected = tile_coordinates.position_on_world_bitmap();
    let tile_screen_position = screen_center.to_vec2() + tile_projected.to_vec2()
        - map_center_projected_position.to_vec2();

    let image = tiles.at(tile_coordinates.x, tile_coordinates.y, zoom);

    let image = if let Some(image) = image {
        image
    } else {
        return;
    };

    if painter
        .clip_rect()
        .intersects(image.rect(tile_screen_position))
    {
        if let Entry::Vacant(vacant) = meshes.entry(tile_coordinates) {
            vacant.insert(image.mesh(tile_screen_position, ui.ctx()));

            for coordinates in [
                tile_coordinates.north(),
                tile_coordinates.east(),
                tile_coordinates.south(),
                tile_coordinates.west(),
            ] {
                draw_tiles(
                    painter,
                    coordinates,
                    map_center_projected_position,
                    zoom,
                    screen_center,
                    tiles,
                    ui,
                    meshes,
                );
            }
        }
    }
}

pub(crate) fn ui(
    my_position: Position,
    ctx: &Context,
    ui: &mut Ui,
    id_source: impl std::hash::Hash,
    tiles: &mut Tiles,
) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

    let id = ui.make_persistent_id(Id::new(id_source));

    let mut memory = ui
        .ctx()
        .data_mut(|data| data.get_persisted::<MapMemory>(id).unwrap_or_default());

    Window::new("Map")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [10., -10.])
        .show(ctx, |ui| {
            ui.checkbox(&mut memory.osm, "OSM");

            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("➕").font(FontId::proportional(20.)))
                    .clicked()
                {
                    memory.zoom += 1;
                }

                if ui
                    .button(RichText::new("➖").font(FontId::proportional(20.)))
                    .clicked()
                {
                    memory.zoom -= 1;
                }
            });
        });

    memory
        .center_mode
        .screen_drag(response, my_position, memory.zoom);

    let map_center = memory.center_mode.position(my_position);

    // Center of the visible screen, in pixels, e.g. 400, 300 for 800x600 window.
    let screen_center = ui.clip_rect().center();

    // Projected means layed on a huge world bitmap using Mercator.
    let map_center_projected_position: Pos2 = map_center.project_with_zoom(memory.zoom).into();

    let painter = ui.painter().with_clip_rect(rect);

    if memory.osm {
        let mut shapes = Default::default();
        draw_tiles(
            &painter,
            map_center.tile(memory.zoom),
            map_center_projected_position,
            memory.zoom,
            screen_center,
            tiles,
            ui,
            &mut shapes,
        );

        for (_, shape) in shapes {
            painter.add(shape);
        }
    }

    ui.ctx().data_mut(|data| data.insert_persisted(id, memory));
}
