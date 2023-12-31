#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use eframe::egui;
use eframe::IconData;
use egui::*;
use egui_extras::RetainedImage;
use std::time::Instant;
pub const WIDTH: i32 = 1976 / 2;
pub const HEIGHT: i32 = 1792 / 2;

mod fractal;
mod hsl;
use crate::fractal::{mandelbrot, mandelcomplist, px, py, Coord};
//TODO
// allow negative powers (look on wikipedia, theres a cool formula thing that you dont understand)
// better colours
// styling
fn main() -> Result<(), eframe::Error> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.icon_data = Some(IconData {
        rgba: mandelbrot(
            Coord { x: -0.765, y: 0. },
            1.,
            250.,
            2,
            256,
            256,
            ColoringMode::Hsl(0., 1., 360.),
        ),
        width: 256,
        height: 256,
    });
    native_options.maximized = true;

    eframe::run_native(
        "Mandelbrot Explorer",
        native_options,
        Box::new(|_cc| Box::<Content>::default()),
    )
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ColoringMode {
    Hsl(f64, f64, f64),
    Monochrome(Color32, f64),
    Funky(f64),
}
impl ColoringMode {
    fn output(&self) -> String {
        match self {
            ColoringMode::Hsl(_, _,_) => return String::from("HSL"),
            ColoringMode::Monochrome(_, _) => return String::from("Monochrome"),
            ColoringMode::Funky(_) => return String::from("Funky"),
        }
    }
}
struct Content {
    center: Coord,
    zoom: f64,
    image: RetainedImage,
    time: f64,
    maxitr: i32,
    exponent: i32,
    prev: (Coord, f64, i32, i32, ColoringMode, bool),
    coloring: ColoringMode,
    pi: f64,
    axes: bool,
    orbits: bool,
}
impl Default for Content {
    fn default() -> Self {
        Self {
            center: Coord { x: -0.765, y: 0. },
            zoom: 1.,
            image: RetainedImage::from_color_image(
                "mandel",
                ColorImage::from_rgba_unmultiplied(
                    [WIDTH as usize, HEIGHT as usize],
                    &mandelbrot(
                        Coord { x: -0.765, y: 0. },
                        1.,
                        300.,
                        2,
                        WIDTH,
                        HEIGHT,
                        ColoringMode::Hsl(0., 1., 360.),
                    ),
                ),
            ),
            time: 50000000.,
            maxitr: 300,
            exponent: 2,
            prev: (
                Coord { x: -0.765, y: 0. },
                1.,
                0,
                2,
                ColoringMode::Hsl(0., 1., 360.),
                false,
            ),
            coloring: ColoringMode::Hsl(0., 1., 360.),
            pi: 0.,
            axes: false,
            orbits: false,
        }
    }
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = Frame::side_top_panel(&ctx.style()).inner_margin(0.0);
        egui::SidePanel::left("my_left_panel")
            .frame(frame)
            .exact_width(WIDTH as f32 + 1.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.put(Rect{min:Pos2{x:0.,y:0.}, max:Pos2{x:WIDTH as f32, y:HEIGHT as f32}}, egui::Image::new(self.image.texture_id(ctx), Vec2{x: WIDTH as f32, y: HEIGHT as f32}));
                    let painter = egui::Painter::new(ctx.clone(), egui::LayerId::new(egui::Order::Foreground, egui::Id::new("mandel")),Rect{min:Pos2{x:0.,y:0.}, max:Pos2{x:WIDTH as f32, y:HEIGHT as f32}});
                    if self.axes{
                    painter.vline(fractal::xp(0.0,self.center.x,self.zoom,WIDTH) as f32,0.0..=HEIGHT as f32,  egui::Stroke{width: 5., color: Color32::WHITE});
                    painter.hline(0.0..=WIDTH as f32,fractal::yp(0.0,self.center.y,self.zoom,HEIGHT) as f32,  egui::Stroke{width: 5., color: Color32::WHITE});
                }

                    let pos = ctx.pointer_hover_pos();
                    if pos.is_some() {
                        let pos = pos.unwrap();
                        if pos.x < WIDTH as f32 && pos.y < HEIGHT as f32 {
                            let x = px(pos.x as f64, self.zoom, self.center.x, WIDTH);
                            let y = py(pos.y as f64, self.zoom, self.center.y, HEIGHT);
                            ui.label(format!("pointer x: {}", x));
                            ui.label(format!("pointer y: {}", -y));


                            let (iterations, points, period) =
                                mandelcomplist(x, y, self.maxitr as f64, self.exponent);
                            if self.orbits{
                                for point in points{
                                    painter.circle_filled(Pos2{x: fractal::xp(point.x, self.center.x, self.zoom, WIDTH) as f32, y: fractal::yp(point.y, self.center.y, self.zoom, HEIGHT) as f32}, 2.0, Color32::WHITE);
                                }
                            }
                            if iterations == self.maxitr {
                                if period > -1{
                                    ui.label(format!(
                                        "this point stays bounded, with a period of {})",
                                        period
                                    ));
                                }
                                else{

                                ui.label(format!(
                                    "this point stays bounded (in {} iterations), but no period detected",
                                    self.maxitr
                                ));
                            }
                            } else {
                                ui.label(format!(
                                    "this point escapes in : {} iterations",
                                    iterations
                                ));
                            }
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
                ui.label(format!(
                    "frame render time: {:.1}ms",
                    (self.time / 1000000.)
                ));
                let fps = 1000000000. / (self.time);
                let color = Color32::from_rgb(
                    (-(fps - 20.) * 255. / 5.) as u8,
                    ((fps - 10.) * 255. / 5.) as u8,
                    0,
                );
                ui.colored_label(color, format!("({:.1} fps)", fps));
            });
            ui.separator();
            ui.style_mut().spacing.item_spacing = Vec2 { x: 10., y: 15. };
            ui.checkbox(&mut self.axes, "show axes");
            ui.checkbox(&mut self.orbits, "show orbits");
            ui.add(egui::Slider::new(&mut self.maxitr, 0..=15000).text("max iterations"));
            ui.add(egui::Slider::new(&mut self.exponent, 1..=100).text("exponent"));
            ui.add(
                egui::DragValue::new(&mut self.center.x)
                    .speed(0.1 * self.zoom)
                    .prefix("x: "),
            );
            ui.add(
                egui::DragValue::new(&mut self.center.y)
                    .speed(0.1 * self.zoom)
                    .prefix("y: "),
            );
            ui.add(
                egui::Slider::new(&mut self.zoom, 1.0..=0.000000000001)
                    .logarithmic(true)
                    .text("scale"),
            );
            egui::ComboBox::from_label("Select one!")
                .selected_text(format!("{}", self.coloring.output()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.coloring, ColoringMode::Hsl(0., 1., 360.), "Hsl");
                    ui.selectable_value(
                        &mut self.coloring,
                        ColoringMode::Monochrome(egui::Color32::WHITE, 1.),
                        "Monochrome",
                    );
                    ui.selectable_value(&mut self.coloring, ColoringMode::Funky(0.), "funky mode");
                });
            if let ColoringMode::Hsl(ref mut shift, ref mut normalisation, ref mut range) = &mut self.coloring {
                let _ = ui.add(egui::Slider::new(shift, 0.0..=360.).text("hue shift"));
                let _ = ui.add(egui::Slider::new(normalisation, 0.0..=500.).text("hue normalisation"));
                let _ = ui.add(egui::Slider::new(range, 0.0..=360.).text("hue range"));

            };
            if let ColoringMode::Funky(ref mut shift) = &mut self.coloring {
                let _ = ui.add(egui::Slider::new(shift, 0.0..=360.).text("hue shift"));
            };
            if let ColoringMode::Monochrome(ref mut color, ref mut range) = &mut self.coloring {
                ui.horizontal(|ui| {
                    ui.label("tint:");
                    let _ = egui::color_picker::color_edit_button_srgba(
                        ui,
                        color,
                        egui::color_picker::Alpha::Opaque,
                    );
                });
                let _ = ui.add(egui::Slider::new(range, 0.0..=500.).text("colour normalisation"));
            };
            if ui.add(egui::Button::new("reset")).clicked() {
                *self = Self {
                    center: Coord { x: -0.765, y: 0. },
                    zoom: 1.,
                    image: RetainedImage::from_color_image(
                        "mandel",
                        ColorImage::from_rgba_unmultiplied(
                            [WIDTH as usize, HEIGHT as usize],
                            &mandelbrot(
                                Coord { x: -0.765, y: 0. },
                                1.,
                                300.,
                                2,
                                WIDTH,
                                HEIGHT,
                                ColoringMode::Hsl(0., 1., 360.),
                            ),
                        ),
                    ),
                    time: 50000000.,
                    maxitr: 300,
                    exponent: 2,
                    prev: (
                        Coord { x: -0.765, y: 0. },
                        1.,
                        0,
                        2,
                        ColoringMode::Hsl(0., 1., 360.),
                        false,
                    ),
                    coloring: ColoringMode::Hsl(0., 1., 360.),
                    pi: 0.,
                    axes: false,
                    orbits: false,
                }
            }
            if ui.add(egui::Button::new("calculate pi!")).clicked() {
                self.pi = crate::fractal::piapprox();
            }
            if self.pi != 0. {
                ui.label(format!("pi = {}", self.pi));
            }

            let new: (Coord, f64, i32, i32, ColoringMode, bool) = (
                self.center,
                self.zoom,
                self.maxitr,
                self.exponent,
                self.coloring,
                self.axes,
            );
            if new != self.prev {
                self.render();
            }
            if ctx.input(|i| i.pointer.is_decidedly_dragging()) {
                let origin = ctx.input(|i| i.pointer.press_origin());
                if origin.is_some() {
                    let origin = origin.unwrap();
                    let current = ctx.input(|i| i.pointer.interact_pos()).unwrap();
                    if origin.x < WIDTH as f32
                        && origin.y < HEIGHT as f32
                        && current.x < WIDTH as f32
                        && current.y < HEIGHT as f32
                        && origin.y > 0.
                        && current.y > 0.
                    {
                        let dy = ctx.input(|i| i.pointer.delta()).y as f64;
                        let dx = ctx.input(|i| i.pointer.delta()).x as f64;
                        self.center.x -= ((2.47) / WIDTH as f64) * self.zoom * dx;
                        self.center.y -= ((2.24) / HEIGHT as f64) * self.zoom * dy;
                        self.render()
                    }
                }
            }
            if ctx.input(|i| i.key_pressed(Key::A)) {
                self.center.x -= 0.1 * self.zoom;
                self.render();
            }
            if ctx.input(|i| i.key_pressed(Key::W)) {
                self.center.y -= 0.1 * self.zoom;
                self.render();
            }
            if ctx.input(|i| i.key_pressed(Key::S)) {
                self.center.y += 0.1 * self.zoom;
                self.render();
            }
            if ctx.input(|i| i.key_pressed(Key::D)) {
                self.center.x += 0.1 * self.zoom;
                self.render();
            }
            let current = ctx.input(|i| i.pointer.hover_pos());
            if current.is_some() {
                let current = current.unwrap();
                if current.x < WIDTH as f32 && current.y < HEIGHT as f32 && current.y > 0. {
                    if ctx.input(|i| i.scroll_delta.y > 0.) {
                        self.zoom *= 0.5;
                        self.render();
                    }
                    if ctx.input(|i| i.scroll_delta.y < 0.) {
                        self.zoom *= 2.;
                        self.render();
                    }
                    if ctx.input(|i| i.pointer.secondary_pressed()) {
                        self.center = Coord {
                            x: px(current.x as f64, self.zoom, self.center.x, WIDTH),
                            y: py(current.y as f64, self.zoom, self.center.y, HEIGHT),
                        }
                    }
                }
            }
            self.prev = new;
        });
    }
}
impl Content {
    fn render(&mut self) {
        let now = Instant::now();
        self.image = RetainedImage::from_color_image(
            "mandel",
            ColorImage::from_rgba_unmultiplied(
                [WIDTH as usize, HEIGHT as usize],
                &mandelbrot(
                    self.center,
                    self.zoom,
                    self.maxitr as f64,
                    self.exponent,
                    WIDTH,
                    HEIGHT,
                    self.coloring,
                ),
            ),
        );
        self.time = now.elapsed().as_nanos() as f64;
    }
}
