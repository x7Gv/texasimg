use arboard::{ImageData, Clipboard};
use eframe::{egui, epaint::{Rgba, Vec2}, Renderer};
use egui_extras::RetainedImage;
use image::EncodableLayout;
use mktemp::Temp;
use texasimg::latex_render::{ContentColour, RenderContent, RenderContentOptions, containerised::RenderInstanceCont, RenderBackend};
use std::{sync::mpsc, borrow::Cow};

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.always_on_top = true;
    options.transparent = true;
    options.drag_and_drop_support = true;
    options.decorated = false;
    options.maximized = true;
    options.initial_window_size = Some(Vec2::new(100., 100.));

    let channel = mpsc::channel();

    eframe::run_native("TeXasIMG", options, Box::new(|_cc| Box::new(TexasimgApp::new_with_channel(channel))))
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ContentType {
    Formula,
    Raw,
}

type ImagePacket = (Vec<u8>, (image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, (u32, u32)));
type ImageSender = mpsc::Sender<ImagePacket>;
type ImageReceiver = mpsc::Receiver<ImagePacket>;

struct TexasimgApp {
    input: String,
    scale: f32,
    render_rx: ImageReceiver,
    render_tx: ImageSender,
    render_ready: bool,
    preview_ready: bool,
    colour: ContentColour,
    content_type: ContentType,
    tmp_dir: Temp,
    cb_ctx: Clipboard,
    img: Option<RetainedImage>,
}

impl TexasimgApp {
    fn new_with_channel((tx, rx): (ImageSender, ImageReceiver)) -> Self {
        TexasimgApp {
            input: "x^2 + 1 = 0".to_string(),
            scale: 2.,
            render_rx: rx,
            render_tx: tx,
            render_ready: true,
            preview_ready: true,
            colour: ContentColour::default(),
            content_type: ContentType::Formula,
            tmp_dir: Temp::new_dir().unwrap(),
            cb_ctx: Clipboard::new().unwrap(),
            img: None,
        }
    }
}

impl eframe::App for TexasimgApp {

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        Rgba::TRANSPARENT
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        egui::Window::new("TeXasIMG").show(&ctx, |ui| {

            ui.label("Enter LaTeX here");
            ui.code_editor(&mut self.input);

            ui.group(|ui| {
                egui::ComboBox::from_label("Text colour")
                    .selected_text(format!("{:?}", self.colour))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.colour, ContentColour::Black, "Black");
                        ui.selectable_value(&mut self.colour, ContentColour::White, "White");
                    });

                egui::ComboBox::from_label("Input type")
                    .selected_text(format!("{:?}", self.content_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.content_type, ContentType::Formula, "Formula");
                        ui.selectable_value(&mut self.content_type, ContentType::Raw, "Raw");
                    });

                ui.horizontal(|ui| {
                    if ui.button("RENDER").clicked() {
                        let rc: RenderContent;
                        let mut rco = RenderContentOptions::default();
                        rco.scale = Some(self.scale);
                        rco.ink_colour = (&self.colour).clone();

                        match self.content_type {
                            ContentType::Formula => {
                                rc = RenderContent::new_with_options(self.input.clone(), rco);
                            },
                            ContentType::Raw => {
                                rc = RenderContent::new_with_options(self.input.clone(), rco);
                            },
                        }

                        let mut r_i = RenderInstanceCont::new(self.tmp_dir.as_path(), rc);

                        let tx_j = self.render_tx.clone();
                        std::thread::spawn(move || {
                            if let Ok(data) = r_i.render() {
                                let img = image::load_from_memory(&data).unwrap().to_rgba8();
                                let (w, h) = img.dimensions();

                                tx_j.send((data, (img, (w, h)))).unwrap();
                            }
                        });

                        self.render_ready = false;
                    }

                    if ui.button("EXIT").clicked() {
                        std::process::exit(0);
                    }

                    ui.add(egui::Slider::new(&mut self.scale, 1.0..=10.0).text("scale"));

                    if let Ok(data) = self.render_rx.try_recv() {
                        let img_data = ImageData {
                            width: (data.1).1.0 as usize,
                            height: (data.1).1.1 as usize,
                            bytes: Cow::Borrowed(data.1.0.as_bytes()),
                        };

                        match RetainedImage::from_image_bytes("out", data.0.as_bytes()) {
                            Ok(image) => {
                                self.img = Some(image);
                            },
                            Err(_) => {},
                        }

                        self.cb_ctx.set_image(img_data).unwrap();
                        self.render_ready = true;
                    } else {
                        if !self.render_ready {
                            ui.add(egui::Spinner::new());
                        }
                    }

                    if let Some(image) = &self.img {
                        image.show(ui);
                    }
                });
            });

            ui.collapsing("log", |ui| {
                ui.code("EMPTY");
            })
        });

        frame.set_window_size(ctx.used_size());

    }
}
