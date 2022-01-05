use super::super::ether::{distance_between, EtherId};
use super::super::node::{Node, NodeId, NodeInterface, NodeInterfaceId};
use super::super::packet::Packet;
use crate::app::{Renderer, BACK, DELETE, DIJKSTRA};
use rand::seq::SliceRandom;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureQuery};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use uuid::Uuid;

pub struct RouterNode {
    id: NodeId,
    position: Point,
    interfaces: HashMap<NodeInterfaceId, NodeInterface>,
    known_routes: HashMap<NodeId, NodeInterfaceId>,
    pub(super) texture: Texture,
}

impl Node for RouterNode {
    fn get_id(&self) -> NodeId {
        self.id
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn corresponds_to_position(&self, position: Point) -> bool {
        distance_between((position, self.get_position())) < 35.0
    }

    fn draw(&self, renderer: &mut Renderer) -> Result<(), String> {
        let position = self.get_position();
        renderer.canvas.copy(
            &renderer.node_texture,
            None,
            Some(Rect::from_center(position, 50, 50)),
        )?;
        let TextureQuery { width, height, .. } = self.texture.query();
        renderer.canvas.copy(
            &self.texture,
            None,
            Some(Rect::from_center(position.offset(0, -10), width, height)),
        )
    }

    fn will_receive(&self, _interface: NodeInterfaceId, packet: &Packet) -> bool {
        packet.current_sender != self.id
    }

    fn receive(
        &mut self,
        _interface: NodeInterfaceId,
        packet: Packet,
    ) -> Vec<(NodeInterfaceId, Packet)> {
        // println!(
        //     "{} {:3} > {:3} : RX {:3} | {}",
        //     packet.uuid, packet.source, packet.destination, self.id, interface,
        // );
        if DELETE.load(Ordering::Relaxed) {
            return vec![];
        }
        if packet.destination != self.get_id() {
            let out_interface = self.get_known_route_interface(packet.destination);
            vec![(
                out_interface,
                Packet {
                    uuid: packet.uuid,
                    source: packet.source,
                    current_sender: self.get_id(),
                    destination: packet.destination,
                },
            )]
        } else {
            if BACK.load(Ordering::Relaxed) {
                vec![(
                    self.get_known_route_interface(packet.source),
                    Packet {
                        uuid: Uuid::new_v4(),
                        source: packet.destination,
                        current_sender: self.get_id(),
                        destination: packet.source,
                    },
                )]
            } else {
                vec![]
            }
        }
    }

    fn get_known_route_interface(&self, destination: NodeId) -> NodeInterfaceId {
        let known_route_interface = if DIJKSTRA.load(Ordering::Relaxed) {
            self.known_routes.get(&destination)
        } else {
            None
        };

        known_route_interface
            .unwrap_or_else(|| {
                let keys: Vec<&String> = self.interfaces.keys().collect();
                *keys.choose(&mut rand::thread_rng()).unwrap()
            })
            .to_string()
    }

    fn set_known_route(&mut self, destination: NodeId, send_to: NodeId) {
        /*let keys: Vec<&String> = self.interfaces.keys().collect();
        let mut keys: Vec<usize> = keys
            .iter()
            .map(|s| (*s).rsplit_once('-').unwrap().1.parse::<usize>().unwrap())
            .collect();
        keys.sort();
        let key = if destination == 16 {
            keys.last()
        } else {
            keys.first()
        };
        let interface = self.id.to_string() + "-" + &key.unwrap().to_string();*/
        let interface = format!("{}-{}", self.get_id(), send_to);
        self.known_routes.insert(destination, interface);
    }

    fn get_interface(&self, interface: NodeInterfaceId) -> &NodeInterface {
        self.interfaces.get(&interface).unwrap()
    }

    fn connect_interface(&self, interface: NodeInterfaceId, ether: EtherId) {
        self.interfaces
            .get(&interface)
            .expect(&*format!(
                "No interface '{}' in node '{}'!",
                interface,
                self.get_id()
            ))
            .connect(ether)
    }

    fn create_interface(&mut self, id: NodeInterfaceId) -> Result<NodeInterfaceId, String> {
        if self.interfaces.contains_key(&id) {
            Err("Interface already created!".to_string())
        } else {
            self.interfaces
                .insert(id.clone(), NodeInterface::new(self.get_id(), id.clone()));
            Ok(id)
        }
    }
}

impl RouterNode {
    pub fn new(renderer: &mut Renderer, id: NodeId, position: Point) -> Self {
        let texture = renderer
            .make_text(&id.to_string(), position, Color::RED)
            .unwrap()
            .0;

        Self {
            id,
            position,
            interfaces: HashMap::new(),
            known_routes: HashMap::new(),
            texture,
        }
    }
}
