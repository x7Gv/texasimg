use std::sync::{Arc, Mutex};
use std::{borrow::Cow, sync::mpsc};

use arboard::{Clipboard, ImageData};
use eframe::egui::{Id, ScrollArea, Sense, RichText, Button, Style};
use eframe::emath::Align2;
use eframe::epaint::{vec2, Color32, FontId, Rgba, Stroke};
use eframe::IconData;
use eframe::{egui, epaint::Vec2};
use egui_extras::RetainedImage;
use image::EncodableLayout;
use mktemp::Temp;
use teximex::{
    document::{Document, DocumentBuilder, DocumentMathMode, DocumentOptions},
    render::{
        native::{NativeLogRecord, RenderInstanceNative},
        RenderBackend, RenderInstance, RenderOptions,
    },
    tex::{Color, MathMode, TexString},
};

use egui_demo_lib::syntax_highlighting::code_view_ui;

fn main() {
    let img = include_bytes!("assets/teximex_icon.png");

    let mut options = eframe::NativeOptions::default();
    options.always_on_top = true;
    options.transparent = true;
    options.drag_and_drop_support = true;
    options.decorated = true;
    options.initial_window_size = Some(Vec2::new(300.0, 400.0));
    //options.icon_data = Some(IconData { rgba: img.to_vec(), width: 128, height: 128 });

    let channel = mpsc::channel();

    eframe::run_native(
        "TeXImEx",
        options,
        Box::new(|_cc| Box::new(TeximexApp::new_with_channel(channel))),
    ).unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ContentType {
    MathMode,
    Raw,
}

enum Packet {
    Image(ImagePacket),
    NoImage(Logs),
}

type ImagePacket = (
    Vec<u8>,
    (image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, (u32, u32)),
    Logs,
);

type Logs = Vec<NativeLogRecord>;

type ImageSender = mpsc::Sender<Packet>;
type ImageReceiver = mpsc::Receiver<Packet>;

struct TeximexApp {
    input: String,
    scale: f32,
    margin: f32,
    render_rx: ImageReceiver,
    render_tx: ImageSender,
    render_ready: bool,
    preview_ready: bool,
    color: teximex::tex::Color,
    content_type: ContentType,
    tmp: Temp,
    clipboard: Clipboard,
    logs: Vec<NativeLogRecord>,
    img: Option<RetainedImage>,
    additional_preamble: String,
}

impl TeximexApp {
    fn new_with_channel((tx, rx): (ImageSender, ImageReceiver)) -> Self {
        TeximexApp {
            input: "x^2+1=0".to_string(),
            scale: 2.,
            margin: 4.,
            render_rx: rx,
            render_tx: tx,
            render_ready: true,
            preview_ready: true,
            color: Color::default(),
            content_type: ContentType::MathMode,
            tmp: Temp::new_dir().unwrap(),
            clipboard: Clipboard::new().unwrap(),
            logs: Vec::new(),
            img: None,
            additional_preamble: String::new(),
        }
    }
}

impl TeximexApp {
    fn compile_document(&self) -> Document<String> {
        match &self.content_type {
            ContentType::MathMode => {
                let doc = Document::builder(&*self.input);
                let mut math_doc = doc.mathmode(DocumentMathMode::Displayed);
                math_doc.color(self.color);
                math_doc.add_preamble(self.additional_preamble.clone());

                math_doc.build()
            }
            ContentType::Raw => {
                let mut doc = Document::builder(&*self.input);
                doc.color(self.color);
                doc.add_preamble(self.additional_preamble.clone());

                doc.build()
            }
        }
    }

    fn document_to_string(&self) -> String {
        let doc = self.compile_document();
        doc.to_tex()
    }

    fn render_img(&mut self) {
        let doc = self.compile_document();

        println!("{}", doc.to_tex());

        let ri = RenderInstance::<String>::new_with_options(RenderOptions::new(
            Some(self.scale),
            Some(self.margin),
        ))
        .load(doc);

        let mut rin = RenderInstanceNative::new(&self.tmp.as_path(), ri);

        let tx_j = self.render_tx.clone();
        std::thread::spawn(move || {
            if let Ok(data) = rin.render() {
                let img = image::load_from_memory(&data).unwrap().into_rgba8();
                let (w, h) = img.dimensions();

                tx_j.send(Packet::Image((data, (img, (w, h)), rin.logs)))
                    .unwrap();
            } else {
                tx_j.send(Packet::NoImage(rin.logs)).unwrap();
            }
        });

        self.render_ready = false;
    }
}

impl eframe::App for TeximexApp {

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0,0.0,0.0,0.0]
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        let style = ctx.style();

        let mut styl = Style::default();
        styl.visuals.selection.bg_fill = Color32::from_rgb(94, 150, 93);
        ctx.set_style(styl);


        ctx.input(|state| {
            if state.key_pressed(egui::Key::F5) {
                self.render_img()
            }
        });

        egui::CentralPanel::default().show(&ctx, |ui| {

            ui.label("Enter (La)TeX here");

            ui.add(
                egui::TextEdit::multiline(&mut self.input)
                    .desired_width(f32::INFINITY)
                    .desired_rows(usize::MAX)
                    .font(egui::TextStyle::Monospace)
                    .interactive(true)
                    .code_editor()
                    .desired_rows(5)
                    .lock_focus(true),
            );

             ui.horizontal(|ui| {
                if ui.button(RichText::new("RENDER").monospace()).clicked() {
                    self.render_img();
                }

                 if ui.add(Button::new(RichText::new("EXIT").monospace().color(Color32::WHITE)).fill(Color32::DARK_RED)).clicked() {
                     std::process::exit(0);
                 }
            });

            ui.collapsing(RichText::new("raw document"), |ui| {
                code_view_ui(ui, &self.document_to_string());
            });

            ui.group(|ui| {

                ui.label("(La)TeX related options.");

                egui::ComboBox::from_label("Text color")
                    .selected_text(format!("{:?}", self.color))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.color, Color::Black, "Black");
                        ui.selectable_value(&mut self.color, Color::White, "White");
                    });

                egui::ComboBox::from_label("Input type")
                    .selected_text(format!("{:?}", self.content_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.content_type,
                            ContentType::MathMode,
                            "MathMode",
                        );
                        ui.selectable_value(&mut self.content_type, ContentType::Raw, "Raw");
                    });
            });

            ui.group(|ui| {
                 ui.vertical(|ui| {
                    ui.label("Rendering / post-TeX related options.");
                    ui.add(egui::Slider::new(&mut self.scale, 1.0..=10.0).text("scale"));
                    ui.add(egui::Slider::new(&mut self.margin, 0.0..=128.0).text("margin"));
                })
            });

            ui.group(|ui| {
                if let Some(image) = &self.img {
                    image.show(ui);
                }
            });

            ui.collapsing("preamble", |ui| {
                ui.label(r#"Enter (optional)[additional] preamble to be inserted before `\begin{document}`"#);
                ui.code_editor(&mut self.additional_preamble);
            });

            ui.collapsing("log", |ui| {
                ScrollArea::both().show(ui, |ui| {

                    code_view_ui(ui, {
                        &self.logs.iter().enumerate().map(|pair| {
                        if pair.0 == self.logs.len() - 1 {
                            format!("{}", pair.1)
                        } else {
                            format!("{}\n", pair.1)
                        }
                    }).collect::<String>()
                    });
                });
            });

            if let Ok(data) = self.render_rx.try_recv() {
                match data {
                    Packet::Image(data) => {
                        let img_data = ImageData {
                            width: (data.1).1 .0 as usize,
                            height: (data.1).1 .1 as usize,
                            bytes: Cow::Borrowed(data.1 .0.as_bytes()),
                        };

                        match RetainedImage::from_image_bytes("out", data.0.as_bytes()) {
                            Ok(image) => {
                                self.img = Some(image);
                            }
                            Err(_) => {}
                        }

                        dbg!(self.img.is_none());

                        self.clipboard.set_image(img_data).unwrap();

                        self.logs = data.2;

                        self.render_ready = true;
                    }
                    Packet::NoImage(data) => {

                        println!("NO IMAGI");

                        self.logs = data;
                        self.render_ready = true;
                    }
                }
            } else {
                if !self.render_ready {
                    ui.add(egui::Spinner::new());
                }
            }
        });

        frame.set_window_size(ctx.used_size());
    }
}
