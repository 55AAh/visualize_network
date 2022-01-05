use super::super::ether::{Ether, EtherId, EtherInterface};
use crate::app::Renderer;
use crate::network::node::{Node, NodeId, NodeInterfaceId};
use sdl2::rect::Point;

pub struct Cable {
    id: EtherId,
    sides: Option<[EtherInterface; 2]>,
    cached_positions: Option<[Point; 2]>,
}

impl Ether for Cable {
    fn get_id(&self) -> EtherId {
        self.id
    }

    fn draw(&self, renderer: &mut Renderer) -> Result<(), String> {
        match self.cached_positions {
            Some(cached_positions) => renderer
                .canvas
                .draw_line(cached_positions[0], cached_positions[1]),
            None => Ok(()),
        }
    }

    fn connect_internal(&mut self, mut interfaces: Vec<EtherInterface>) {
        assert_eq!(interfaces.len(), 2);

        let mut interfaces = interfaces.drain(..);
        let interface_1 = interfaces.next().unwrap();
        let interface_2 = interfaces.next().unwrap();
        self.sides = Some([interface_1, interface_2]);
    }

    fn connect(&mut self, interfaces: Vec<(&Box<dyn Node>, NodeInterfaceId)>) {
        let mut cached_positions = vec![];

        self.connect_internal(
            interfaces
                .iter()
                .map(|(node, interface)| {
                    cached_positions.push(node.get_position());
                    node.connect_interface(interface.clone(), self.get_id());
                    EtherInterface::from_node_interface(node.get_interface(interface.clone()))
                })
                .collect(),
        );

        let mut cached_positions = cached_positions.drain(..);
        self.cached_positions = Some([
            cached_positions.next().unwrap(),
            cached_positions.next().unwrap(),
        ]);
    }

    fn get_interfaces(&self) -> Vec<EtherInterface> {
        self.sides.as_ref().unwrap().to_vec()
    }

    fn get_distance_multipliers(&self) -> Vec<(NodeId, NodeId, f64)> {
        let mut distances = vec![];
        if let Some(sides) = self.sides.as_ref() {
            let nodes = (
                sides.get(0).unwrap().owner_node,
                sides.get(1).unwrap().owner_node,
            );
            distances.push((nodes.0, nodes.1, 1.0));
            distances.push((nodes.1, nodes.0, 1.0));
        }
        distances
    }
}

impl Cable {
    pub fn new(id: EtherId) -> Cable {
        Cable {
            id,
            sides: None,
            cached_positions: None,
        }
    }
}
