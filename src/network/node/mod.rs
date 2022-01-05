pub mod endpoint_node;
pub mod router_node;

use super::ether::EtherId;
use super::packet::Packet;
use crate::app::Renderer;
use sdl2::rect::Point;
use std::cell::Cell;

pub type NodeId = usize;

pub trait Node {
    fn get_id(&self) -> NodeId;
    fn get_position(&self) -> Point;
    fn corresponds_to_position(&self, position: Point) -> bool;
    fn draw(&self, renderer: &mut Renderer) -> Result<(), String>;
    fn will_receive(&self, interface: NodeInterfaceId, packet: &Packet) -> bool;
    fn receive(
        &mut self,
        interface: NodeInterfaceId,
        packet: Packet,
    ) -> Vec<(NodeInterfaceId, Packet)>;
    fn get_known_route_interface(&self, destination: NodeId) -> NodeInterfaceId;
    fn set_known_route(&mut self, destination: NodeId, send_to: NodeId);
    fn get_interface(&self, interface: NodeInterfaceId) -> &NodeInterface;
    fn connect_interface(&self, interface: NodeInterfaceId, ether: EtherId);
    fn create_interface(&mut self, id: NodeInterfaceId) -> Result<NodeInterfaceId, String>;
}

pub type NodeInterfaceId = String;

#[derive(Clone)]
pub struct NodeInterface {
    owner_node: NodeId,
    id_in_owner: NodeInterfaceId,
    connected_ether: Cell<Option<EtherId>>,
}

impl NodeInterface {
    pub fn new(owner_node: NodeId, id_in_owner: NodeInterfaceId) -> NodeInterface {
        NodeInterface {
            owner_node,
            id_in_owner,
            connected_ether: Cell::new(None),
        }
    }

    pub fn connect(&self, ether: EtherId) {
        self.connected_ether.replace(Some(ether));
    }

    pub fn get_to_owner(&self) -> (NodeId, NodeInterfaceId) {
        (self.owner_node, self.id_in_owner.clone())
    }

    pub fn get_connected_ether(&self) -> Option<EtherId> {
        self.connected_ether.get()
    }
}
