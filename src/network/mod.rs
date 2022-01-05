pub mod ether;
pub mod node;
pub mod packet;

use crate::app::Renderer;
use crate::network::node::endpoint_node::EndpointNode;
use ether::cable::Cable;
use ether::{distance_between, Ether, EtherId, EtherInterface};
use indexmap::IndexMap;
use node::{router_node::RouterNode, Node, NodeId, NodeInterfaceId};
use packet::Packet;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use slab::Slab;
use std::collections::HashMap;
use uuid::Uuid;

pub struct Network {
    nodes: Slab<Box<dyn Node>>,
    ethers: Slab<Box<dyn Ether>>,
    transmissions: Vec<Transmission>,
    incoming: Vec<(NodeId, NodeInterfaceId, Packet)>,
    outgoing: Vec<(EtherId, Point, Packet)>,
}

impl Network {
    pub fn new() -> Network {
        Network {
            nodes: Slab::new(),
            ethers: Slab::new(),
            transmissions: vec![],
            incoming: vec![],
            outgoing: vec![],
        }
    }

    pub fn add_router_node(&mut self, renderer: &mut Renderer, position: Point) -> NodeId {
        let entry = self.nodes.vacant_entry();
        let id = entry.key();
        entry.insert(Box::new(RouterNode::new(renderer, id, position)) as Box<dyn Node>);
        id
    }

    pub fn add_endpoint_node(&mut self, renderer: &mut Renderer, position: Point) -> NodeId {
        let entry = self.nodes.vacant_entry();
        let id = entry.key();
        entry.insert(Box::new(EndpointNode::new(renderer, id, position)) as Box<dyn Node>);
        id
    }

    pub fn add_router_interface(
        &mut self,
        node: NodeId,
        interface: NodeInterfaceId,
    ) -> Result<NodeInterfaceId, String> {
        self.nodes
            .get_mut(node)
            .expect(&*format!("Node '{}' not found!", node))
            .create_interface(interface)
    }

    pub fn add_cable(&mut self) -> EtherId {
        let entry = self.ethers.vacant_entry();
        let id = entry.key();
        entry.insert(Box::new(Cable::new(id)) as Box<dyn Ether>);
        id
    }

    pub fn connect_cable(&mut self, sides: ((NodeId, NodeInterfaceId), (NodeId, NodeInterfaceId))) {
        let cable = self.add_cable();
        let (s1, s2) = sides;
        self.ethers.get_mut(cable).unwrap().connect(vec![
            (self.nodes.get(s1.0).unwrap(), s1.1),
            (self.nodes.get(s2.0).unwrap(), s2.1),
        ])
    }

    pub fn locate_node(&self, position: Point) -> Option<NodeId> {
        for (id, node) in self.nodes.iter() {
            if node.corresponds_to_position(position) {
                return Some(id);
            }
        }
        None
    }

    pub fn send(&mut self, uuid: Uuid, source: NodeId, destination: NodeId) {
        self.incoming.push((
            source,
            "localhost".to_string(),
            Packet {
                uuid,
                source,
                current_sender: source,
                destination,
            },
        ))
        /*let node = self.nodes.get(node).expect("Node not found!");
        self.outgoing.push((
            node.get_interface(_interface)
                .get_connected_ether()
                .expect("Interface not connected!"),
            node.get_position(),
            packet,
        ))*/
    }

    pub fn get_packets_count(&self) -> usize {
        self.transmissions.len()
    }

    /*pub fn receive(&mut self, node: NodeId, interface: NodeInterfaceId, packet: Packet) {
        self.incoming.push((node, interface, packet))
    }*/

    fn calculate_preferred_path(
        &self,
        source: NodeId,
        destination: NodeId,
        distances: &Vec<(NodeId, NodeId, i64)>,
    ) -> Option<Vec<NodeId>> {
        if source == destination {
            return Some(vec![source, destination]);
        }

        let mut visited = HashMap::new();
        let mut shortest: IndexMap<NodeId, (NodeId, i64)> = IndexMap::new();
        shortest.insert(source, (source, 0));

        while !shortest.is_empty() {
            let current_id = shortest.first().unwrap().0.clone();
            let (previous_id, current_distance) = shortest.remove(&current_id).unwrap();
            if current_id != destination {
                for (from, to, distance) in distances.iter() {
                    if *from == current_id {
                        let distance_to_next = current_distance + distance;
                        if !visited.contains_key(to)
                            && if let Some((_, known_distance_to_next)) = shortest.get(to) {
                                distance_to_next < *known_distance_to_next
                            } else {
                                true
                            }
                        {
                            shortest.insert(*to, (current_id, distance_to_next));
                        }
                    }
                }

                shortest.sort_by(|_, a, _, b| a.1.cmp(&b.1));
            }

            visited.insert(current_id, previous_id);
        }

        let mut path = vec![];
        let mut current = destination;
        while current != source {
            path.push(current);
            if let Some(previous) = visited.get(&current) {
                current = *previous;
            } else {
                return None;
            }
        }
        path.push(source);
        path.reverse();
        Some(path)
    }

    pub fn calculate_routes(&mut self) {
        let mut distances = vec![];
        for (_, ether) in self.ethers.iter() {
            for (from, to, multiplier) in ether.get_distance_multipliers() {
                let distance = distance_between((
                    self.nodes.get(to).unwrap().get_position(),
                    self.nodes.get(from).unwrap().get_position(),
                ));
                distances.push((from, to, (distance * multiplier) as i64));
            }
        }

        let mut routes = vec![];

        for (_, source) in self.nodes.iter() {
            for (_, destination) in self.nodes.iter() {
                let (source_id, destination_id) = (source.get_id(), destination.get_id());
                let path = self
                    .calculate_preferred_path(source_id, destination_id, &distances)
                    .expect("Cannot calculate path: unreachable!");
                routes.push((source_id, destination_id, path[1]));
            }
        }

        for (source, destination, send_to) in routes.drain(..) {
            self.nodes
                .get_mut(source)
                .unwrap()
                .set_known_route(destination, send_to);
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) -> Result<(), String> {
        renderer.canvas.set_draw_color(Color::BLACK);
        for (_, ether) in self.ethers.iter() {
            ether.draw(renderer)?;
        }

        for (_, node) in self.nodes.iter() {
            node.draw(renderer)?;
        }

        for transmission in self.transmissions.iter_mut() {
            transmission.travelled += 1;
            let c =
                (100f64 * (transmission.travelled as f64) / (transmission.distance as f64)) as i32;
            transmission.packet.draw(
                renderer,
                (transmission.from * (100 - c) + transmission.to * c) / 100,
            )?;
            if transmission.travelled >= transmission.distance {
                let (owner, owner_interface) = transmission.recipient.get_to_owner();
                self.incoming
                    .push((owner, owner_interface.clone(), transmission.packet.clone()));
                println!(
                    "{:6} # {} {:3} > {:3} : RX {:3} | {}",
                    renderer.timer_subsystem.ticks(),
                    transmission.packet.uuid,
                    transmission.packet.source,
                    transmission.packet.destination,
                    owner,
                    owner_interface,
                );
            }
        }

        self.transmissions
            .retain(|transmission| transmission.travelled < transmission.distance);

        for (node, interface, packet) in self.incoming.drain(..) {
            let node = self.nodes.get_mut(node).expect("Node not found!");
            self.outgoing
                .extend(node.receive(interface, packet).drain(..).map(
                    |(outgoing_interface, outgoing_packet)| {
                        (
                            node.get_interface(outgoing_interface.clone())
                                .get_connected_ether()
                                .expect("Interface not connected!"),
                            node.get_position(),
                            outgoing_packet,
                        )
                    },
                ));
        }

        for (ether, from_position, packet) in self.outgoing.drain(..) {
            for interface in self.ethers.get(ether).unwrap().get_interfaces() {
                let (owner, owner_interface) = interface.get_to_owner();
                let owner = self.nodes.get(owner).unwrap();
                if owner.will_receive(owner_interface, &packet) {
                    self.transmissions.push(Transmission::new(
                        from_position,
                        owner.get_position(),
                        interface,
                        packet.clone(),
                    ));
                }
            }
        }

        Ok(())
    }
}

struct Transmission {
    from: Point,
    to: Point,
    distance: i32,
    travelled: i32,
    recipient: EtherInterface,
    packet: Packet,
}

impl Transmission {
    pub fn new(from: Point, to: Point, recipient: EtherInterface, packet: Packet) -> Transmission {
        Transmission {
            from,
            to,
            distance: distance_between((from, to)) as i32,
            travelled: 0,
            recipient,
            packet,
        }
    }
}
