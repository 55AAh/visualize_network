pub mod cable;

use super::node::{Node, NodeId, NodeInterface, NodeInterfaceId};
use crate::app::Renderer;
use sdl2::rect::Point;
use std::cell::Cell;

pub type EtherId = usize;

pub trait Ether {
    fn get_id(&self) -> EtherId;
    fn draw(&self, renderer: &mut Renderer) -> Result<(), String>;
    fn connect_internal(&mut self, interfaces: Vec<EtherInterface>);
    fn connect(&mut self, interfaces: Vec<(&Box<dyn Node>, NodeInterfaceId)>);
    fn get_interfaces(&self) -> Vec<EtherInterface>;
    fn get_distance_multipliers(&self) -> Vec<(NodeId, NodeId, f64)>;
}

impl dyn Ether {
    /*pub fn connect(&mut self, interfaces: Vec<(&Box<dyn Node>, NodeInterfaceId)>) {
        self.connect_internal(
            interfaces
                .iter()
                .map(|(node, interface)| {
                    node.connect_interface(interface.clone(), self.get_id());
                    EtherInterface::from_node_interface(node.get_interface(interface.clone()))
                })
                .collect(),
        )
    }*/
}

#[derive(Clone)]
pub struct EtherInterface {
    owner_node: NodeId,
    id_in_owner: NodeInterfaceId,
    _connected_ether: Cell<Option<EtherId>>,
}

impl EtherInterface {
    pub fn from_node_interface(interface: &NodeInterface) -> EtherInterface {
        let (owner_node, id_in_owner) = interface.get_to_owner();
        let _connected_ether = Cell::new(interface.get_connected_ether());
        EtherInterface {
            owner_node,
            id_in_owner,
            _connected_ether,
        }
    }

    pub fn get_to_owner(&self) -> (NodeId, NodeInterfaceId) {
        (self.owner_node, self.id_in_owner.clone())
    }
}

pub fn distance_between(points: (Point, Point)) -> f64 {
    let vector = points.0 - points.1;
    ((vector.x().pow(2) + vector.y().pow(2)) as f64).sqrt()
}
