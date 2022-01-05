use super::router_node::RouterNode;
use crate::app::Renderer;
use crate::network::ether::EtherId;
use crate::network::node::{Node, NodeId, NodeInterface, NodeInterfaceId};
use crate::network::packet::Packet;
use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;

pub struct EndpointNode(RouterNode);

impl Node for EndpointNode {
    fn get_id(&self) -> NodeId {
        self.0.get_id()
    }

    fn get_position(&self) -> Point {
        self.0.get_position()
    }

    fn corresponds_to_position(&self, position: Point) -> bool {
        self.0.corresponds_to_position(position)
    }

    fn draw(&self, renderer: &mut Renderer) -> Result<(), String> {
        let position = self.get_position();
        renderer.canvas.copy(
            &renderer.endpoint_texture,
            None,
            Some(Rect::from_center(position, 50, 50)),
        )?;
        let TextureQuery { width, height, .. } = self.0.texture.query();
        renderer.canvas.copy(
            &self.0.texture,
            None,
            Some(Rect::from_center(position.offset(0, -10), width, height)),
        )
    }

    fn will_receive(&self, interface: NodeInterfaceId, packet: &Packet) -> bool {
        self.0.will_receive(interface, packet)
    }

    fn receive(
        &mut self,
        interface: NodeInterfaceId,
        packet: Packet,
    ) -> Vec<(NodeInterfaceId, Packet)> {
        self.0.receive(interface, packet)
    }

    fn get_known_route_interface(&self, destination: NodeId) -> NodeInterfaceId {
        self.0.get_known_route_interface(destination)
    }

    fn set_known_route(&mut self, destination: NodeId, send_to: NodeId) {
        self.0.set_known_route(destination, send_to)
    }

    fn get_interface(&self, interface: NodeInterfaceId) -> &NodeInterface {
        self.0.get_interface(interface)
    }

    fn connect_interface(&self, interface: NodeInterfaceId, ether: EtherId) {
        self.0.connect_interface(interface, ether)
    }

    fn create_interface(&mut self, id: NodeInterfaceId) -> Result<NodeInterfaceId, String> {
        self.0.create_interface(id)
    }
}

impl EndpointNode {
    pub fn new(renderer: &mut Renderer, id: NodeId, position: Point) -> Self {
        Self {
            0: RouterNode::new(renderer, id, position),
        }
    }
}
