#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use eframe::egui;
use eframe::IconData;
use egui::*;
use egui_extras::RetainedImage;
use std::time::Instant;
pub const WIDTH: i32 = 1976 / 2;
pub const HEIGHT: i32 = 1792 / 2;

mod fractal;
use crate::fractal::{coord, mandelbrot, mandelcomp, px, py};
//TODO
// allow negative powers (look on wikipedia, theres a cool formula thing that you dont understand)
// better colours
// add option to show axes
// styling
fn main() -> Result<(), eframe::Error> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(12)
        .build_global()
        .unwrap();
    let mut native_options = eframe::NativeOptions::default();
    native_options.icon_data = Some(IconData {
        rgba: mandelbrot(
            coord { x: -0.765, y: 0. },
            1.,
            250.,
            2,
            256,
            256,
            ColoringMode::Hsl(0.),
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
#[derive(Copy, Clone, PartialEq)]
pub enum ColoringMode {
    Hsl(f64),
    Monochrome(Color32),
}
impl fmt::Debug for ColoringMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
struct Content {
    center: coord,
    zoom: f64,
    image: RetainedImage,
    time: f64,
    maxitr: i32,
    exponent: i32,
    prev: (coord, f64, i32, i32, ColoringMode),
    coloring: ColoringMode,
}
impl Default for Content {
    fn default() -> Self {
        Self {
            center: coord { x: -0.765, y: 0. },
            zoom: 1.,
            image: RetainedImage::from_color_image(
                "mandel",
                ColorImage::from_rgba_unmultiplied(
                    [WIDTH as usize, HEIGHT as usize],
                    &mandelbrot(
                        coord { x: -0.765, y: 0. },
                        1.,
                        300.,
                        2,
                        WIDTH,
                        HEIGHT,
                        ColoringMode::Hsl(0.),
                    ),
                ),
            ),
            time: 50000000.,
            maxitr: 300,
            exponent: 2,
            prev: (coord { x: -0.765, y: 0. }, 1., 0, 2, ColoringMode::Hsl(0.)),
            coloring: ColoringMode::Hsl(0.),
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
                    self.image.show(ui);
                    let pos = ctx.pointer_hover_pos();
                    if pos.is_some() {
                        let pos = pos.unwrap();
                        if pos.x < WIDTH as f32 && pos.y < HEIGHT as f32 {
                            let x = px(pos.x as f64, self.zoom, self.center.x, WIDTH);
                            let y = py(pos.y as f64, self.zoom, self.center.y, HEIGHT);

                            ui.label(format!("pointer x: {}", x));
                            ui.label(format!("pointer y: {}", y));
                            let (iterations, _) =
                                mandelcomp(x, y, self.maxitr as f64, self.exponent);
                            if iterations == self.maxitr {
                                ui.label(format!(
                                    "this point stays bounded (in {} iterations)",
                                    self.maxitr
                                ));
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
                .selected_text(format!("{:?}", self.coloring))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.coloring, ColoringMode::Hsl(0.), "Hsl");
                    ui.selectable_value(
                        &mut self.coloring,
                        ColoringMode::Monochrome(egui::Color32::WHITE),
                        "Monochrome",
                    );
                });
            if let ColoringMode::Hsl(ref mut shift) = &mut self.coloring {
                let _ = ui.add(egui::Slider::new(shift, 0.0..=360.).text("hue shift"));
            };
            if let ColoringMode::Monochrome(ref mut color) = &mut self.coloring {
                ui.horizontal(|ui| {
                    ui.label("tint:");
                    let _ = egui::color_picker::color_edit_button_srgba(
                        ui,
                        color,
                        egui::color_picker::Alpha::Opaque,
                    );
                });
            };
            let new: (coord, f64, i32, i32, ColoringMode) = (
                self.center,
                self.zoom,
                self.maxitr,
                self.exponent,
                self.coloring,
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
