mod lib;

use std::{
    borrow::Cow,
    fs::File,
    io::{BufWriter, Cursor, Write},
    process::Command,
};

use ansi_term::{Colour::*, Style};
use arboard::{Clipboard, ImageData};
use image::{EncodableLayout, ImageEncoder, ImageFormat, ImageOutputFormat};
use structopt::clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum MathMode {
        Inline,
        Displayed
    }
}

fn latex_template(equation: String, mode: MathMode) -> String {
    match mode {
        MathMode::Inline => {
            format!(
                r#"\documentclass[12pt]{}
\usepackage{}
\usepackage{}
\usepackage{}
\usepackage{}
\usepackage{}
\usepackage[utf8]{}
\thispagestyle{}
\begin{}
\color{}
\( {} \)
\end{}"#,
                "{article}",
                "{amsmath}",
                "{amssymb}",
                "{amsfonts}",
                "{xcolor}",
                "{siunitx}",
                "{inputenc}",
                "{empty}",
                "{document}",
                "{white}",
                equation,
                "{document}"
            )
        }
        MathMode::Displayed => {
            format!(
                r#"\documentclass[12pt]{}
\usepackage{}
\usepackage{}
\usepackage{}
\usepackage{}
\usepackage{}
\usepackage[utf8]{}
\thispagestyle{}
\begin{}
\color{}
\[ {} \]
\end{}"#,
                "{article}",
                "{amsmath}",
                "{amssymb}",
                "{amsfonts}",
                "{xcolor}",
                "{siunitx}",
                "{inputenc}",
                "{empty}",
                "{document}",
                "{white}",
                equation,
                "{document}"
            )
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "texasimg")]
struct Opt {
    equation: String,
    #[structopt(short, long)]
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

    let tmp = {
        Command::new("mktemp")
            .arg("--dir")
            .output()
            .expect("executing mktmp failed")
    };
    let mut tmp_path = String::from_utf8_lossy(&tmp.stdout).to_string();

    // The stdout contains an unwanted `\n`
    tmp_path.truncate(tmp_path.len() - 1);

    let content = latex_template(opt.equation, opt.math_mode);

    let separator = ansi_term::Colour::RGB(55, 59, 65)
        .bold()
        .paint("────────────────────────────────────────");

    println!(
        "{}\n{}\n{}",
        separator,
        Style::new().bold().paint(&content),
        separator
    );

    println!("temp folder: \n\t{}", Green.underline().paint(&tmp_path));

    let mut file = File::create(format!("{}/{}", &tmp_path, "equation.tex"))?;
    file.write_all(&content.into_bytes())?;

    let _cmd = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-i")
        .arg(r#"--user=1000:1000"#)
        .arg("--net=none")
        .arg("-v")
        .arg(format!(r#"{}:/data"#, tmp_path))
        .arg(r#"blang/latex:ubuntu"#)
        .arg("/bin/bash")
        .arg("-c")
        .arg(format!("timeout 5 latex -no-shell-escape -interaction=nonstopmode -halt-on-error equation.tex && timeout 5 dvisvgm --no-fonts --scale={} --exact equation.dvi", opt.scale))
        .output();

    let mut svg_opt = usvg::Options::default();
    svg_opt.resources_dir = std::fs::canonicalize(&tmp_path)
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    svg_opt.fontdb.load_system_fonts();

    let svg_data = std::fs::read(format!("{}/equation.svg", &tmp_path))?;
    let rtree = usvg::Tree::from_data(&svg_data, &svg_opt.to_ref())?;

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap
        .save_png(format!("{}/equation.png", &tmp_path))
        .unwrap();

    let img = image::open(format!("{}/equation.png", &tmp_path))
        .unwrap()
        .to_rgba8();
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
    loop {}

    Ok(())
}
