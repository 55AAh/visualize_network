pub mod scenario;

use super::network::node::NodeId;
use crate::network::Network;
use scenario::Scenario;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, TextureQuery, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use sdl2::{EventPump, TimerSubsystem};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use uuid::Uuid;

pub static SOURCE_NODE: AtomicUsize = AtomicUsize::new(0);
pub static DESTINATION_NODE: AtomicUsize = AtomicUsize::new(16);
pub static DIJKSTRA: AtomicBool = AtomicBool::new(true);
pub static BACK: AtomicBool = AtomicBool::new(false);
pub static SLOW: AtomicBool = AtomicBool::new(false);
pub static DELETE: AtomicBool = AtomicBool::new(false);

pub struct App {
    pub network: Network,
    nodes: Vec<NodeId>,
}

impl App {
    pub fn new() -> Result<App, String> {
        Ok(App {
            network: Network::new(),
            nodes: vec![],
        })
    }

    pub fn run(&mut self, scenario: Option<Scenario>) -> Result<(), String> {
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

        let mut renderer = Renderer::new(&ttf_context)?;

        let (use_scenario, nodes_positions, cable_connections, mut events) = match scenario {
            Some(scenario) => (
                true,
                scenario.nodes,
                scenario.cable_connections,
                scenario.transmissions,
            ),
            None => (
                false,
                Vec::from([
                    (80, 80, true),
                    (150, 130, false),
                    (290, 50, false),
                    (80, 300, true),
                    (120, 510, true),
                    (200, 360, false),
                    (400, 520, true),
                    (320, 280, false),
                    (480, 120, false),
                    (650, 150, true),
                    (300, 370, false),
                    (560, 40, true),
                    (520, 320, false),
                    (630, 360, false),
                    (640, 440, false),
                    (570, 560, true),
                    (760, 490, true),
                ]),
                Vec::from([
                    (0, 1),
                    (1, 3),
                    (3, 5),
                    (4, 5),
                    (5, 10),
                    (10, 7),
                    (1, 7),
                    (1, 2),
                    (2, 8),
                    (8, 11),
                    (11, 9),
                    (6, 10),
                    (10, 12),
                    (7, 12),
                    (8, 12),
                    (12, 13),
                    (13, 14),
                    (14, 15),
                    (14, 16),
                    (9, 12),
                ]),
                VecDeque::new(),
            ),
        };

        let nodes: Vec<NodeId> = nodes_positions
            .iter()
            .map(|(x, y, is_endpoint)| {
                if *is_endpoint {
                    self.network
                        .add_endpoint_node(&mut renderer, Point::new(*x, *y))
                } else {
                    self.network
                        .add_router_node(&mut renderer, Point::new(*x, *y))
                }
            })
            .collect();

        cable_connections.iter().for_each(|(ind1, ind2)| {
            let i1 = self
                .network
                .add_router_interface(nodes[*ind1], format!("{}-{}", ind1, ind2).to_string())
                .unwrap();
            let i2 = self
                .network
                .add_router_interface(nodes[*ind2], format!("{}-{}", ind2, ind1).to_string())
                .unwrap();
            self.network
                .connect_cable(((nodes[*ind1], i1), (nodes[*ind2], i2)));
        });

        self.nodes.extend(nodes);

        self.network.calculate_routes();

        let mut prev_mouse_buttons = HashSet::new();

        let mut _flag_texture_x = 40;
        let flags_textures = [
            ("DIJKSTRA", &DIJKSTRA),
            ("BACK", &BACK),
            ("SLOW", &SLOW),
            ("DELETE", &DELETE),
        ]
        .map(|(str, flag)| {
            let (texture, mut rect) = renderer
                .make_text(str, Point::new(_flag_texture_x, 5), Color::RED)
                .unwrap();
            rect.offset((rect.width() / 2) as i32 + 10, (rect.height() / 2) as i32);
            _flag_texture_x = rect.right() as i32;
            ((texture, rect), flag)
        });
        let mut source_destination_ids_texture = renderer.make_text(
            &format!(
                "{}-{}",
                SOURCE_NODE.load(Ordering::Relaxed),
                DESTINATION_NODE.load(Ordering::Relaxed)
            ),
            Point::new(25, 5),
            Color::RED,
        )?;
        source_destination_ids_texture
            .1
            .offset(0, (source_destination_ids_texture.1.height() / 2) as i32);

        let mut packets_count_texture =
            renderer.make_text("PACKETS:", Point::new(11, 25), Color::RED)?;
        packets_count_texture.1.offset(
            (packets_count_texture.1.width() / 2) as i32,
            (packets_count_texture.1.height() / 2) as i32,
        );

        let ascii_textures: Vec<Option<(Texture, Rect)>> = (0..=255)
            .map(|code: u8| {
                let c = code as char;
                if !c.is_ascii_control() {
                    renderer
                        .make_text(&format!("{}", code as char), Point::new(0, 0), Color::BLACK)
                        .ok()
                } else {
                    None
                }
            })
            .collect();

        let mut next_event_time = 0;

        'main: loop {
            if use_scenario {
                let time = renderer.timer_subsystem.ticks();
                if time >= next_event_time {
                    'process_events: loop {
                        if events.is_empty() {
                            break 'process_events;
                        }
                        let event = events.front().unwrap();
                        if event.0 > time {
                            next_event_time = event.0;
                            break 'process_events;
                        }
                        self.network.send(event.1, event.2, event.3);
                        events.pop_front();
                    }
                }
            } else {
                for event in renderer.event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'main,
                        Event::KeyDown {
                            keycode: Some(keycode),
                            ..
                        } => match keycode {
                            Keycode::Space => self.network.send(
                                Uuid::new_v4(),
                                SOURCE_NODE.load(Ordering::Relaxed),
                                DESTINATION_NODE.load(Ordering::Relaxed),
                            ),
                            Keycode::D => {
                                DIJKSTRA.fetch_xor(true, Ordering::Relaxed);
                            }
                            Keycode::Backspace => {
                                BACK.fetch_xor(true, Ordering::Relaxed);
                            }
                            Keycode::Minus => {
                                SLOW.fetch_xor(true, Ordering::Relaxed);
                            }
                            Keycode::Delete => {
                                DELETE.fetch_xor(true, Ordering::Relaxed);
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }

            let mouse_state = renderer.event_pump.mouse_state();
            let mouse_buttons = mouse_state.pressed_mouse_buttons().collect();

            let new_mouse_buttons = &mouse_buttons - &prev_mouse_buttons;
            let old_mouse_buttons = &prev_mouse_buttons - &mouse_buttons;

            if !new_mouse_buttons.is_empty() || !old_mouse_buttons.is_empty() {
                if new_mouse_buttons.contains(&MouseButton::Left) {
                    if let Some(node) = self
                        .network
                        .locate_node(Point::new(mouse_state.x(), mouse_state.y()))
                    {
                        SOURCE_NODE.store(node, Ordering::Relaxed);
                        source_destination_ids_texture = renderer.make_text(
                            &format!("{}-{}", node, DESTINATION_NODE.load(Ordering::Relaxed)),
                            source_destination_ids_texture.1.center(),
                            Color::RED,
                        )?;
                    }
                } else if new_mouse_buttons.contains(&MouseButton::Right) {
                    if let Some(node) = self
                        .network
                        .locate_node(Point::new(mouse_state.x(), mouse_state.y()))
                    {
                        DESTINATION_NODE.store(node, Ordering::Relaxed);
                        source_destination_ids_texture = renderer.make_text(
                            &format!("{}-{}", SOURCE_NODE.load(Ordering::Relaxed), node),
                            source_destination_ids_texture.1.center(),
                            Color::RED,
                        )?;
                    }
                }
            }

            prev_mouse_buttons = mouse_buttons;

            renderer.canvas.set_draw_color(Color::WHITE);
            renderer.canvas.clear();
            self.network.render(&mut renderer)?;

            renderer.canvas.copy(
                &source_destination_ids_texture.0,
                None,
                Some(source_destination_ids_texture.1),
            )?;

            for ((texture, rect), flag) in flags_textures.iter() {
                if flag.load(Ordering::Relaxed) {
                    renderer.canvas.copy(texture, None, Some(rect.clone()))?;
                }
            }

            renderer.canvas.copy(
                &packets_count_texture.0,
                None,
                Some(packets_count_texture.1),
            )?;

            let mut _cont_digit_texture_x = packets_count_texture.1.right() + 5;
            let mut packets_count_string = format!("{}", self.network.get_packets_count());
            if use_scenario {
                packets_count_string += &format!(" ({})", events.len());
            }
            for c in packets_count_string.chars() {
                if let Some((texture, texture_rect)) = &ascii_textures[c as usize] {
                    renderer.canvas.copy(
                        texture,
                        None,
                        Rect::new(
                            _cont_digit_texture_x,
                            packets_count_texture.1.top(),
                            texture_rect.width(),
                            texture_rect.height(),
                        ),
                    )?;
                    _cont_digit_texture_x += texture_rect.width() as i32;
                }
            }

            renderer.canvas.present();

            ::std::thread::sleep(Duration::new(
                0,
                if SLOW.load(Ordering::Relaxed) {
                    1_000_000_000u32 / 60
                } else {
                    1_000_000_000u32 / 60 / 10
                },
            ));

            if use_scenario && events.is_empty() && self.network.get_packets_count() == 0 {
                break 'main;
            }
        }

        Ok(())
    }
}

pub struct Renderer<'r> {
    event_pump: EventPump,
    pub(super) timer_subsystem: TimerSubsystem,
    pub canvas: WindowCanvas,
    pub font: Font<'r, 'r>,
    pub texture_creator: TextureCreator<WindowContext>,
    pub node_texture: Texture,
    pub endpoint_texture: Texture,
    pub packet_texture: Texture,
}

impl<'r> Renderer<'r> {
    fn new(ttf_context: &'r Sdl2TtfContext) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Network visualization", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();

        let node_texture = texture_creator.load_texture("node.png")?;
        let endpoint_texture = texture_creator.load_texture("endpoint.png")?;
        let packet_texture = texture_creator.load_texture("packet.png")?;

        let event_pump = sdl_context.event_pump()?;
        let timer_subsystem = sdl_context.timer()?;

        let font = ttf_context.load_font("sample.ttf", 15)?;

        Ok(Self {
            event_pump,
            timer_subsystem,
            canvas,
            font,
            texture_creator,
            node_texture,
            endpoint_texture,
            packet_texture,
        })
    }

    pub(crate) fn make_text(
        &self,
        text: &str,
        center: Point,
        color: Color,
    ) -> Result<(Texture, Rect), String> {
        let surface = self
            .font
            .render(text)
            .blended(color)
            .map_err(|e| e.to_string())?;
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();
        let rect = Rect::from_center(center, width, height);
        Ok((texture, rect))
    }
}
