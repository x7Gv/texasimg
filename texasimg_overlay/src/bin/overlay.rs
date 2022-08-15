use std::borrow::Cow;
use std::sync::mpsc;

use arboard::{Clipboard, ImageData};
use egui::{FullOutput, Window};
use image::EncodableLayout;
use mktemp::Temp;
use texasimg::latex_render::{RenderContent, RenderContentOptions, ContentColour, containerised::RenderInstanceCont, RenderBackend};
use texasimg_overlay::{GlfwWindow, WgpuRenderer};
use wgpu::{
    CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum ContentType {
    Formula,
    Raw,
}

struct TexasimgEguiApp {
    input: String,
    scale: f32,
    render_ready: bool,
    colour: ContentColour,
    content_type: ContentType,
}

impl Default for TexasimgEguiApp {
    fn default() -> Self {
        TexasimgEguiApp {
            input: "x^2 + 1 = 0".to_string(),
            scale: 2.,
            render_ready: true,
            colour: ContentColour::default(),
            content_type: ContentType::Formula,
        }
    }
}

fn main() {
    let mut glfw_window = GlfwWindow::new().expect("failed to init window");
    let mut wgpu_renderer =
        pollster::block_on(WgpuRenderer::new(&glfw_window)).expect("failed to init wgpu");
    let ctx = egui::Context::default();

    let mut app = TexasimgEguiApp::default();
    let mut cb_ctx = Clipboard::new().unwrap();
    let tmp_dir = Temp::new_dir().unwrap();

    let (tx, rx) = mpsc::channel();

    glfw_window.window.set_floating(true);

    while !glfw_window.window.should_close() {
        glfw_window.tick();
        wgpu_renderer
            .pre_tick(&glfw_window)
            .expect("failed to tick");
        // use wgpu to draw whatever you want. here we just clear the surface. we only do this IF the framebuffer exists, otherwise, something's gone wrong
        // don't take out the framebuffer either, it is used by egui render pass later
        {
            let mut encoder =
                wgpu_renderer
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("clear pass encoder"),
                    });
            {
                if let Some((_fb, fbv)) = wgpu_renderer.framebuffer_and_view.as_ref() {
                    encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("clear render pass"),
                        color_attachments: &[RenderPassColorAttachment {
                            view: fbv,
                            resolve_target: None,
                            ops: Operations {
                                // transparent color
                                load: LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                }
            }
            wgpu_renderer
                .queue
                .submit(std::iter::once(encoder.finish()));
        }
        // for people who want to only do egui, just stay between begin_frame and end_frame functions. that's where you dela with gui.
        // now, we can do our own things with egui
        // take the input from glfw_window
        ctx.begin_frame(glfw_window.raw_input.take());
        Window::new("TeXAsIMG").show(&ctx, |ui| {

            ui.label("Enter LaTeX here");
            ui.code_editor(&mut app.input);

            ui.group(|ui| {

                /*
                egui::ComboBox::from_label("Text colour")
                    .selected_text(format!("{:?}", app.colour))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.colour, ContentColour::Black, "Black");
                        ui.selectable_value(&mut app.colour, ContentColour::White, "White");
                    });

                egui::ComboBox::from_label("Input type")
                    .selected_text(format!("{:?}", app.content_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.content_type, ContentType::Formula, "Formula");
                        ui.selectable_value(&mut app.content_type, ContentType::Raw, "Raw");
                    });
                */

                ui.horizontal(|ui| {
                    if ui.button("RENDER").clicked() {

                        let rc: RenderContent;
                        let mut rco = RenderContentOptions::default();
                        rco.scale = Some(app.scale);
                        rco.ink_colour = ContentColour::White;

                        match app.content_type {
                            ContentType::Formula => {
                                rc = RenderContent::new_with_options(app.input.clone(), rco);
                            },
                            ContentType::Raw => {
                                rc = RenderContent::new_with_options(app.input.clone(), rco);
                            },
                        }


                        let mut r_i = RenderInstanceCont::new(tmp_dir.as_path(), rc);

                            let tx_j = tx.clone();
                            std::thread::spawn(move || {
                                if let Ok(data) = r_i.render() {
                                    let img =
                                        image::load_from_memory(&data).unwrap().to_rgba8();
                                    let (w, h) = img.dimensions();

                                    tx_j.send((img, (w, h))).unwrap();
                                }
                            });

                        app.render_ready = false;
                    }

                    if ui.button("EXIT").clicked() {
                        std::process::exit(0);
                    }

                    ui.add(egui::Slider::new(&mut app.scale, 1.0..=10.0).text("scale"));

                    if let Ok(data) = rx.try_recv() {
                        let img_data = ImageData {
                            width: (data.1).0 as usize,
                            height: (data.1).1 as usize,
                            bytes: Cow::Borrowed(data.0.as_bytes()),
                        };

                        cb_ctx.set_image(img_data).unwrap();
                        app.render_ready = true;
                    } else {
                        if !app.render_ready {
                            ui.add(egui::Spinner::new());
                        }
                    }
                });
            });

            ui.collapsing("log", |ui| {
                ui.code("EMPTY");
            })
        });

        // now at the end of all gui stuff, we end the frame to get platform output and textures_delta and shapes
        let FullOutput {
            platform_output,
            textures_delta,
            shapes,
            ..
        } = ctx.end_frame();
        let shapes = ctx.tessellate(shapes); // need to convert shapes into meshes to draw
                                             // in platform output, we only care about two things. first is whether some text has been copied, which needs ot be put into clipbaord
        if !platform_output.copied_text.is_empty() {
            glfw_window
                .window
                .set_clipboard_string(&platform_output.copied_text);
        }
        // here we draw egui to framebuffer and submit it finally
        wgpu_renderer
            .tick(textures_delta, shapes, &glfw_window)
            .expect("failed to draw for some reason");
        // based on whether egui wants the input or not, we will set the overlay to be passthrough or not.
        if ctx.wants_keyboard_input() || ctx.wants_pointer_input() {
            glfw_window.window.set_mouse_passthrough(false);
        } else {
            glfw_window.window.set_mouse_passthrough(true);
        }
    }
}
