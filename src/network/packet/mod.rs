use super::node::NodeId;
use crate::app::Renderer;
use sdl2::rect::{Point, Rect};
use uuid::Uuid;

#[derive(Clone)]
pub struct Packet {
    pub uuid: Uuid,
    pub source: NodeId,
    pub current_sender: NodeId,
    pub destination: NodeId,
}

impl Packet {
    pub fn draw(&self, renderer: &mut Renderer, position: Point) -> Result<(), String> {
        renderer.canvas.copy(
            &renderer.packet_texture,
            None,
            Some(Rect::from_center(position, 30, 30)),
        )
    }
}
