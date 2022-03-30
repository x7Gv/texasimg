use std::{
    borrow::Cow,
    fs::File,
    io::{BufWriter, Cursor, Write},
    process::Command,
    time::Duration,
};

use ansi_term::{Colour::*, Style};
use arboard::{Clipboard, ImageData};
use image::{EncodableLayout, ImageEncoder, ImageFormat, ImageOutputFormat};
use structopt::clap::arg_enum;
use structopt::StructOpt;
use texasimg::{ContentKind, ModefulContent, RenderContent, RenderInstance, Rendered};

arg_enum! {
    #[derive(Debug)]
    enum MathMode {
        Inline,
        Displayed
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "texasimg")]
struct Opt {
    equation: String,
    #[structopt(short, long, default_value = "2.0")]
    scale: f32,
    #[structopt(short, long, possible_values = &MathMode::variants(), case_insensitive = true, default_value = "Displayed")]
    math_mode: MathMode,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    println!(
        "
████████╗███████╗██╗░░██╗░█████╗░░██████╗██╗███╗░░░███╗░██████╗░
╚══██╔══╝██╔════╝╚██╗██╔╝██╔══██╗██╔════╝██║████╗░████║██╔════╝░
░░░██║░░░█████╗░░░╚███╔╝░███████║╚█████╗░██║██╔████╔██║██║░░██╗░
░░░██║░░░██╔══╝░░░██╔██╗░██╔══██║░╚═══██╗██║██║╚██╔╝██║██║░░╚██╗
░░░██║░░░███████╗██╔╝╚██╗██║░░██║██████╔╝██║██║░╚═╝░██║╚██████╔╝
░░░╚═╝░░░╚══════╝╚═╝░░╚═╝╚═╝░░╚═╝╚═════╝░╚═╝╚═╝░░░░░╚═╝░╚═════╝░"
    );

    let rc: RenderContent;
    match opt.math_mode {
        MathMode::Inline => {
            rc = RenderContent::builder(ContentKind::Formula(ModefulContent::Inline(opt.equation)))
                .build()
        }
        MathMode::Displayed => {
            rc = RenderContent::builder(ContentKind::Formula(ModefulContent::Displayed(
                opt.equation,
            )))
            .build()
        }
    }

    let mut ri = RenderInstance::builder().content(rc).build().unwrap();

    let ri = ri.render().unwrap();

    let separator = ansi_term::Colour::RGB(55, 59, 65)
        .bold()
        .paint("────────────────────────────────────────");

    println!(
        "{}\n{}\n{}",
        separator,
        Style::new().bold().paint(ri.content().content()),
        separator
    );

    println!(
        "temp folder: \n\t{}",
        Green.underline().paint(&*ri.root().to_string_lossy())
    );

    let img = image::load_from_memory(&ri.png()).unwrap().to_rgba8();
    let (w, h) = img.dimensions();

    let mut cb_ctx = Clipboard::new().unwrap();
    let img_data = ImageData {
        width: w as usize,
        height: h as usize,
        bytes: Cow::Borrowed(img.as_bytes()),
    };

    cb_ctx.set_image(img_data).unwrap();

    println!("{}\nThe rendered image should now be located at the systems clipboard.\nOne can paste it with (C^v) on most systems. \n{}", separator, separator);

    println!(
        "{}",
        Yellow.italic().paint(">>> waiting for termination (C^c)")
    );
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
