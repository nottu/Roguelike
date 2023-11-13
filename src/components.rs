use rltk::prelude::*;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Debug, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Into<Point> for &Position {
    fn into(self) -> Point {
        Point::new(self.x, self.y)
    }
}

#[derive(Debug, Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Debug, Component)]
pub struct LeftMover;

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}
