// ============================================================================
// COSMIC Colorpicker Applet
//
// Author: Fred Silveyra (@fredsilveyra)
// Co-author & Technical Assistance: Claude (Anthropic)
// License: GPL-3.0-or-later
// Repository: https://github.com/FredSilveyra/cosmic-ext-applet-colorpicker
// ============================================================================

// SPDX-License-Identifier: GPL-3.0-or-later
//
// Selector de color con lupa.
//
// Este módulo corre en un proceso aparte (`cosmic-ext-applet-colorpicker --pick`).
// El applet vive dentro del compositor anidado de cosmic-panel, que solo
// reenvía popups; por eso no puede crear superficies layer-shell. Este proceso
// se conecta directo a cosmic-comp (WAYLAND_DISPLAY), así que sí puede abrir
// una superficie en la capa Overlay que cubre toda la pantalla.
//
// Flujo:
//   1. Captura la pantalla completa en memoria (portal de COSMIC; grim como respaldo).
//   2. Abre una superficie Overlay transparente por monitor.
//   3. Dibuja la lupa (canvas) siguiendo el cursor.
//   4. Clic izquierdo: imprime "#RRGGBB" en stdout y termina. Escape/clic derecho: cancela.

use cosmic::iced::event::{self, PlatformSpecific, wayland};
use cosmic::iced::keyboard::{self, key::Named};
use cosmic::iced::mouse;
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::{
    IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced::platform_specific::shell::wayland::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer, get_layer_surface,
};
use cosmic::iced::widget::canvas::{self, Frame, Geometry, Path, Stroke, Text};
use cosmic::iced::{
    Color, Event, Length, Pixels, Point, Rectangle, Size, Subscription, Task, Vector,
    alignment, window::Id,
};
use cosmic::cctk::sctk::reexports::client::protocol::wl_output::WlOutput;
use cosmic::prelude::*;
use std::io::Write;
use std::sync::Arc;

/// Píxeles de pantalla visibles por lado (impar para que haya un centro exacto).
const GRID: i32 = 15;
/// Radio de la lupa en píxeles lógicos.
const RADIUS: f32 = 75.0;
/// Espera antes de capturar, para que el popup del applet termine de cerrarse.
const CAPTURE_DELAY_MS: u64 = 180;

pub fn run() -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default()
        .no_main_window(true)
        .exit_on_close(false)
        .transparent(true);
    cosmic::app::run::<Picker>(settings, ())
}

/// Captura de pantalla en RGBA.
#[derive(Debug)]
pub struct Shot {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Shot {
    fn pixel(&self, x: i32, y: i32) -> Option<[u8; 3]> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        let i = ((y as u32 * self.width + x as u32) * 4) as usize;
        Some([self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]])
    }
}

#[derive(Debug, Clone)]
pub struct OutputGeo {
    wl: WlOutput,
    /// Posición y tamaño lógicos del monitor en el escritorio.
    rect: Rectangle,
}

struct Surface {
    id: Id,
    /// Rectángulo lógico del monitor; None si no lo conocemos (modo Active).
    rect: Option<Rectangle>,
}

pub struct Picker {
    core: cosmic::Core,
    shot: Option<Arc<Shot>>,
    outputs: Vec<OutputGeo>,
    surfaces: Vec<Surface>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Captured(Option<Arc<Shot>>),
    Output(OutputGeo),
    Pick(String),
    Cancel,
}

impl cosmic::Application for Picker {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.fredsilveyra.CosmicExtAppletColorpicker.Picker";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: ()) -> (Self, Task<cosmic::Action<Message>>) {
        let app = Picker {
            core,
            shot: None,
            outputs: Vec::new(),
            surfaces: Vec::new(),
        };
        let capture = Task::perform(capture_screen(), |shot| {
            cosmic::Action::App(Message::Captured(shot.map(Arc::new)))
        });
        (app, capture)
    }

    fn view(&self) -> Element<'_, Message> {
        // Sin ventana principal (no_main_window); nunca se llama.
        cosmic::widget::Space::new().into()
    }

    fn view_window(&self, id: Id) -> Element<'_, Message> {
        let Some(shot) = self.shot.clone() else {
            return cosmic::widget::Space::new().into();
        };
        let rect = self.surfaces.iter().find(|s| s.id == id).and_then(|s| s.rect);
        let bbox = self.bbox();

        canvas::Canvas::new(Loupe { shot, rect, bbox })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::Output(geo) => {
                self.outputs.retain(|o| o.wl != geo.wl);
                self.outputs.push(geo);
            }
            Message::Captured(None) => {
                eprintln!("colorpicker: no se pudo capturar la pantalla");
                std::process::exit(2);
            }
            Message::Captured(Some(shot)) => {
                self.shot = Some(shot);
                return self.open_surfaces();
            }
            Message::Pick(hex) => {
                let mut out = std::io::stdout();
                let _ = writeln!(out, "{hex}");
                let _ = out.flush();
                std::process::exit(0);
            }
            Message::Cancel => std::process::exit(1),
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::Output(
                evt,
                wl,
            ))) => {
                let info = match evt {
                    wayland::OutputEvent::Created(Some(info)) => info,
                           wayland::OutputEvent::InfoUpdate(info) => info,
                           _ => return None,
                };
                let (x, y) = info.logical_position?;
                let (w, h) = info.logical_size?;
                Some(Message::Output(OutputGeo {
                    wl,
                    rect: Rectangle::new(
                        Point::new(x as f32, y as f32),
                                         Size::new(w as f32, h as f32),
                    ),
                }))
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Escape),
                            ..
            }) => Some(Message::Cancel),
                           _ => None,
        })
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::iced::theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
            icon_color: Color::WHITE,
        })
    }
}

impl Picker {
    /// Caja lógica que envuelve todos los monitores (lo que abarca la captura).
    fn bbox(&self) -> Option<Rectangle> {
        let mut it = self.outputs.iter().map(|o| o.rect);
        let first = it.next()?;
        Some(it.fold(first, |a, b| a.union(&b)))
    }

    fn open_surfaces(&mut self) -> Task<cosmic::Action<Message>> {
        let targets: Vec<(IcedOutput, Option<Rectangle>)> = if self.outputs.is_empty() {
            vec![(IcedOutput::Active, None)]
        } else {
            self.outputs
            .iter()
            .map(|o| (IcedOutput::Output(o.wl.clone()), Some(o.rect)))
            .collect()
        };

        let tasks = targets.into_iter().map(|(output, rect)| {
            let id = Id::unique();
            self.surfaces.push(Surface { id, rect });
            get_layer_surface(SctkLayerSurfaceSettings {
                id,
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: -1,
                output,
                namespace: "colorpicker-loupe".into(),
                              size: Some((None, None)),
                              size_limits: cosmic::iced::Limits::NONE,
                              ..Default::default()
            })
        });
        Task::batch(tasks)
    }
}

// ---------------------------------------------------------------------------
// Captura de pantalla
// ---------------------------------------------------------------------------

async fn capture_screen() -> Option<Shot> {
    tokio::time::sleep(std::time::Duration::from_millis(CAPTURE_DELAY_MS)).await;
    match capture_portal().await {
        Some(s) => Some(s),
        None => capture_grim().await,
    }
}

/// Portal xdg-desktop-portal (xdg-desktop-portal-cosmic en COSMIC), sin diálogo.
async fn capture_portal() -> Option<Shot> {
    use ashpd::desktop::screenshot::Screenshot;
    let response = Screenshot::request()
    .interactive(false)
    .modal(false)
    .send()
    .await
    .ok()?
    .response()
    .ok()?;
    let path = response.uri().to_file_path().ok()?;
    let img = image::open(&path).ok()?.to_rgba8();
    // El archivo solo lo necesitábamos para leerlo en memoria.
    let _ = std::fs::remove_file(&path);
    Some(Shot {
        width: img.width(),
         height: img.height(),
         rgba: img.into_raw(),
    })
}

/// Respaldo opcional: grim, si está instalado.
async fn capture_grim() -> Option<Shot> {
    let out = tokio::process::Command::new("grim")
    .args(["-t", "ppm", "-"])
    .output()
    .await
    .ok()?;
    if !out.status.success() {
        return None;
    }
    let img = image::load_from_memory(&out.stdout).ok()?.to_rgba8();
    Some(Shot {
        width: img.width(),
         height: img.height(),
         rgba: img.into_raw(),
    })
}

// ---------------------------------------------------------------------------
// Lupa
// ---------------------------------------------------------------------------

struct Loupe {
    shot: Arc<Shot>,
    /// Rectángulo lógico del monitor de esta superficie.
    rect: Option<Rectangle>,
    /// Caja lógica de todos los monitores (lo que abarca la captura).
    bbox: Option<Rectangle>,
}

impl Loupe {
    /// Convierte la posición del cursor (lógica, local a la superficie) a
    /// coordenadas de píxel en la captura.
    fn to_image(&self, local: Point, bounds: Rectangle) -> (i32, i32) {
        let (origin, area) = match (self.rect, self.bbox) {
            (Some(r), Some(b)) => (Point::new(r.x - b.x, r.y - b.y), b.size()),
            _ => (Point::ORIGIN, bounds.size()),
        };
        let sx = self.shot.width as f32 / area.width;
        let sy = self.shot.height as f32 / area.height;
        (
            ((origin.x + local.x) * sx).floor() as i32,
         ((origin.y + local.y) * sy).floor() as i32,
        )
    }

    fn hex_at(&self, local: Point, bounds: Rectangle) -> Option<String> {
        let (x, y) = self.to_image(local, bounds);
        self.shot
        .pixel(x, y)
        .map(|[r, g, b]| format!("#{r:02X}{g:02X}{b:02X}"))
    }
}

impl canvas::Program<Message, cosmic::Theme, cosmic::Renderer> for Loupe {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
              event: &Event,
              bounds: Rectangle,
              cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                Some(canvas::Action::request_redraw())
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let pos = cursor.position_in(bounds)?;
                let hex = self.hex_at(pos, bounds)?;
                Some(canvas::Action::publish(Message::Pick(hex)).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                Some(canvas::Action::publish(Message::Cancel).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &(),
            renderer: &cosmic::Renderer,
            _theme: &cosmic::Theme,
            bounds: Rectangle,
            cursor: mouse::Cursor,
    ) -> Vec<Geometry<cosmic::Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let Some(pos) = cursor.position_in(bounds) else {
            return vec![frame.into_geometry()];
        };

        let (cx, cy) = self.to_image(pos, bounds);
        let half = GRID / 2;
        let cell = (RADIUS * 2.0) / GRID as f32;

        // Posición de la etiqueta (abajo; arriba si no cabe).
        let label_size = Size::new(118.0, 30.0);
        let below = pos.y + RADIUS + 12.0 + label_size.height < bounds.height;
        let label_y = if below {
            pos.y + RADIUS + 12.0
        } else {
            pos.y - RADIUS - 12.0 - label_size.height
        };
        let label_origin = Point::new(pos.x - label_size.width / 2.0, label_y);

        // Fondo casi invisible que envuelve lupa + etiqueta. El renderer por
        // software calcula el área a repintar con el contorno de los trazos sin
        // contar su grosor; este relleno amplía esa área para que no queden
        // rastros del aro al mover el cursor.
        let pad = 8.0;
        let loupe_box = Rectangle::new(
            Point::new(pos.x - RADIUS - pad, pos.y - RADIUS - pad),
                                       Size::new((RADIUS + pad) * 2.0, (RADIUS + pad) * 2.0),
        );
        let label_box = Rectangle::new(
            label_origin - Vector::new(pad, pad),
                                       Size::new(label_size.width + pad * 2.0, label_size.height + pad * 2.0),
        );
        let damage_box = loupe_box.union(&label_box);
        frame.fill_rectangle(
            damage_box.position(),
                             damage_box.size(),
                             Color::from_rgba(0.0, 0.0, 0.0, 1.0 / 255.0),
        );

        let circle = circle_polygon(pos, RADIUS, 48);
        let r2 = RADIUS * RADIUS;
        let inside = |p: Point| {
            let (dx, dy) = (p.x - pos.x, p.y - pos.y);
            dx * dx + dy * dy <= r2
        };

        // Píxeles ampliados. Las celdas completamente dentro del círculo se
        // pintan como rectángulos simples; solo las del borde se recortan.
        for gy in 0..GRID {
            for gx in 0..GRID {
                let tl = Point::new(
                    pos.x - RADIUS + gx as f32 * cell,
                    pos.y - RADIUS + gy as f32 * cell,
                );
                let quad = [
                    tl,
                    Point::new(tl.x + cell, tl.y),
                    Point::new(tl.x + cell, tl.y + cell),
                    Point::new(tl.x, tl.y + cell),
                ];
                let color = match self.shot.pixel(cx + gx - half, cy + gy - half) {
                    Some([r, g, b]) => Color::from_rgb8(r, g, b),
                    None => Color::from_rgb8(40, 40, 40),
                };
                if quad.iter().all(|p| inside(*p)) {
                    // +0.5 evita costuras finas entre celdas vecinas.
                    frame.fill_rectangle(tl, Size::new(cell + 0.5, cell + 0.5), color);
                    continue;
                }
                let clipped = clip_convex(&quad, &circle);
                if clipped.len() >= 3 {
                    frame.fill(&polygon_path(&clipped), color);
                }
            }
        }

        // Cuadrícula tenue.
        let grid_stroke = Stroke::default()
        .with_width(1.0)
        .with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.18));
        for i in 1..GRID {
            let off = -RADIUS + i as f32 * cell;
            let d = (RADIUS * RADIUS - off * off).max(0.0).sqrt();
            frame.stroke(
                &Path::line(Point::new(pos.x + off, pos.y - d), Point::new(pos.x + off, pos.y + d)),
                         grid_stroke,
            );
            frame.stroke(
                &Path::line(Point::new(pos.x - d, pos.y + off), Point::new(pos.x + d, pos.y + off)),
                         grid_stroke,
            );
        }

        // Cruz: marco del píxel central (negro + blanco para contraste en cualquier color).
        let c = Rectangle::new(
            Point::new(pos.x - cell / 2.0, pos.y - cell / 2.0),
                               Size::new(cell, cell),
        );
        let center = Path::rectangle(c.position(), c.size());
        frame.stroke(&center, Stroke::default().with_width(3.0).with_color(Color::BLACK));
        frame.stroke(&center, Stroke::default().with_width(1.5).with_color(Color::WHITE));
        let arm = Stroke::default()
        .with_width(1.5)
        .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.85));
        for (a, b) in [
            (Point::new(pos.x, c.y - 2.0), Point::new(pos.x, c.y - cell * 1.6)),
            (Point::new(pos.x, c.y + cell + 2.0), Point::new(pos.x, c.y + cell * 2.6)),
            (Point::new(c.x - 2.0, pos.y), Point::new(c.x - cell * 1.6, pos.y)),
            (Point::new(c.x + cell + 2.0, pos.y), Point::new(c.x + cell * 2.6, pos.y)),
        ] {
            frame.stroke(&Path::line(a, b), arm);
        }

        // Aro exterior.
        let ring = Path::circle(pos, RADIUS);
        frame.stroke(&ring, Stroke::default().with_width(4.0).with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.55)));
        frame.stroke(&ring, Stroke::default().with_width(2.0).with_color(Color::WHITE));

        // Etiqueta con el HEX.
        if let Some([r, g, b]) = self.shot.pixel(cx, cy) {
            let hex = format!("#{r:02X}{g:02X}{b:02X}");
            let (origin, size) = (label_origin, label_size);
            frame.fill(
                &Path::rounded_rectangle(origin, size, 8.0.into()),
                       Color::from_rgba(0.08, 0.08, 0.08, 0.88),
            );
            frame.fill(
                &Path::rounded_rectangle(origin + Vector::new(8.0, 7.0), Size::new(16.0, 16.0), 4.0.into()),
                       Color::from_rgb8(r, g, b),
            );
            frame.fill_text(Text {
                content: hex,
                position: origin + Vector::new(32.0, size.height / 2.0),
                            color: Color::WHITE,
                            size: Pixels(15.0),
                            font: cosmic::iced::Font::MONOSPACE,
                            align_y: alignment::Vertical::Center.into(),
                            ..Text::default()
            });
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, _state: &(), bounds: Rectangle, cursor: mouse::Cursor) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Hidden
        } else {
            mouse::Interaction::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Geometría: recorte de cada celda al círculo (Sutherland–Hodgman)
// ---------------------------------------------------------------------------

fn circle_polygon(c: Point, r: f32, n: usize) -> Vec<Point> {
    (0..n)
    .map(|i| {
        let t = i as f32 / n as f32 * std::f32::consts::TAU;
        Point::new(c.x + r * t.cos(), c.y + r * t.sin())
    })
    .collect()
}

/// Recorta `subject` contra el polígono convexo `clip` (sentido horario en pantalla).
fn clip_convex(subject: &[Point], clip: &[Point]) -> Vec<Point> {
    let mut out: Vec<Point> = subject.to_vec();
    for i in 0..clip.len() {
        if out.is_empty() {
            break;
        }
        let a = clip[i];
        let b = clip[(i + 1) % clip.len()];
        let inside = |p: Point| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x) >= 0.0;
        let input = std::mem::take(&mut out);
        for j in 0..input.len() {
            let cur = input[j];
            let prev = input[(j + input.len() - 1) % input.len()];
            let (ci, pi) = (inside(cur), inside(prev));
            if ci {
                if !pi {
                    out.push(intersect(prev, cur, a, b));
                }
                out.push(cur);
            } else if pi {
                out.push(intersect(prev, cur, a, b));
            }
        }
    }
    out
}

fn intersect(p1: Point, p2: Point, a: Point, b: Point) -> Point {
    let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
    let (ex, ey) = (b.x - a.x, b.y - a.y);
    let denom = dx * ey - dy * ex;
    if denom.abs() < f32::EPSILON {
        return p2;
    }
    let t = ((a.x - p1.x) * ey - (a.y - p1.y) * ex) / denom;
    Point::new(p1.x + t * dx, p1.y + t * dy)
}

fn polygon_path(points: &[Point]) -> Path {
    Path::new(|b| {
        b.move_to(points[0]);
        for p in &points[1..] {
            b.line_to(*p);
        }
        b.close();
    })
}
