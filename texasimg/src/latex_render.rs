use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

const USEPACKAGE: &'static str = r#"\usepackage{{}}"#;

const DEFAULT_IMPORTS: &'static str = r#"\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsfonts}
\usepackage{xcolor}
\usepackage{siunitx}
\usepackage[utf8]{inputenc}
"#;

fn default_imports() -> Vec<RenderContentImport> {
    DEFAULT_IMPORTS
        .split("\n")
        .map(|s| RenderContentImport::Custom(s.to_string()))
        .collect()
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FormulaMode {
    Inline,
    Displayed,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ContentMode {
    Raw,
    Formula(FormulaMode),
}

impl Default for ContentMode {
    fn default() -> Self {
        Self::Formula(FormulaMode::default())
    }
}

impl Default for FormulaMode {
    fn default() -> Self {
        Self::Displayed
    }
}
impl FormulaMode {
    pub fn as_tex(&self, formula_content: &str) -> String {
        match self {
            FormulaMode::Inline => format!(r#"\( {} \)"#, formula_content),
            FormulaMode::Displayed => format!(r#"\[ {} \]"#, formula_content),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ContentColour {
    Black,
    White,
    // RGB((u8, u8, u8)),
}
impl Default for ContentColour {
    fn default() -> Self {
        Self::Black
    }
}
impl ContentColour {
    pub fn from_hex(hex: u32) -> Self {
        unimplemented!()
    }

    pub fn as_tex(&self) -> String {
        match self {
            ContentColour::Black => r#"\color{black}"#.to_string(),
            ContentColour::White => r#"\color{white}"#.to_string(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RenderContentImport {
    Usepackage(String),
    Custom(String),
}

impl RenderContentImport {
    pub fn as_tex(&self) -> String {
        match self {
            RenderContentImport::Usepackage(val) => USEPACKAGE.replace("{}", &val).to_string(),
            RenderContentImport::Custom(val) => val.clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RenderContentImports {
    pub data: Vec<RenderContentImport>,
}

impl RenderContentImports {
    pub fn add<I: Into<RenderContentImport>>(&mut self, import: I) -> &mut Self {
        self.data.push(import.into());
        self
    }
}

impl Extend<RenderContentImport> for RenderContentImports {
    fn extend<T: IntoIterator<Item = RenderContentImport>>(&mut self, iter: T) {
        self.data.extend(iter)
    }
}

impl ToString for RenderContentImports {
    fn to_string(&self) -> String {
        let mut output = String::new();

        for import in self.data.clone() {
            output.push_str(&format!("\n{}", import.as_tex()));
        }

        output
    }
}

impl Default for RenderContentImports {
    fn default() -> Self {
        Self {
            data: default_imports(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderContentOptions {
    pub ink_colour: ContentColour,
    pub content_mode: ContentMode,
    pub imports: RenderContentImports,
    pub scale: Option<f32>,
}

pub struct RenderContent {
    options: RenderContentOptions,
    formula_content: String,
}
impl RenderContent {
    pub fn new(formula_content: String) -> Self {
        Self {
            options: RenderContentOptions::default(),
            formula_content,
        }
    }

    pub fn new_with_options(formula_content: String, options: RenderContentOptions) -> Self {
        Self {
            options,
            formula_content,
        }
    }

    pub fn as_tex(&self) -> String {
        match &self.options.content_mode {
            ContentMode::Raw => {
                let documentclass = r#"\documentclass[12pt]{article}"#;
                let pagestyle = r#"\thispagestyle{empty}"#;
                let begin = r#"\begin{document}"#;
                let color = self.options.ink_colour.as_tex();
                let content = &self.formula_content;
                let end = r#"\end{document}"#;

                format!(
                    r#"{}
{}
{}
{}
{}
{}
{}
"#,
                    documentclass,
                    self.options.imports.to_string(),
                    pagestyle,
                    begin,
                    color,
                    content,
                    end
                )
            }
            ContentMode::Formula(formula) => {
                let documentclass = r#"\documentclass[12pt]{article}"#;
                let pagestyle = r#"\thispagestyle{empty}"#;
                let begin = r#"\begin{document}"#;
                let color = self.options.ink_colour.as_tex();
                let content = formula.as_tex(&self.formula_content);
                let end = r#"\end{document}"#;

                format!(
                    r#"{}
{}
{}
{}
{}
{}
{}
"#,
                    documentclass,
                    self.options.imports.to_string(),
                    pagestyle,
                    begin,
                    color,
                    content,
                    end
                )
            },
        }
    }

    pub fn set_options(&mut self, options: RenderContentOptions) -> &mut Self {
        self.options = options;
        self
    }

    pub fn options(&self) -> &RenderContentOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut RenderContentOptions {
        &mut self.options
    }

    pub fn set_formula_content(&mut self, formula_content: String) -> &mut Self {
        self.formula_content = formula_content;
        self
    }

    pub fn formula_content(&self) -> &String {
        &self.formula_content
    }

    pub fn formula_content_mut(&mut self) -> &mut String {
        &mut self.formula_content
    }
}

pub struct RenderOutput {
    png: Option<Vec<u8>>,
    stdout: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use crate::latex_render::{ContentColour, FormulaMode, RenderContentOptions, ContentMode};

    use super::RenderContent;

    #[test]
    fn render_content_new() {
        {
            let rc = RenderContent::new("x^2+1=0".to_string());
            assert_eq!(rc.formula_content, "x^2+1=0");
        }

        {
            let mut rco = RenderContentOptions::default();
            rco.content_mode = ContentMode::Formula(FormulaMode::Inline);
            rco.ink_colour = ContentColour::White;

            let rc = RenderContent::new_with_options("x^2+1=0".to_string(), rco.clone());
            assert_eq!(rc.formula_content, "x^2+1=0");
            assert_eq!(rc.options, rco);
        }
    }

    // #[test]
    fn render_content_as_tex() {
        let rc = RenderContent::new("x^2+1=0".to_string());
        assert_eq!(rc.as_tex(), "\\documentclass[12pt]{article}\n\\usepackage{amsmath}\n\\usepackage{amssymb}\n\\usepackage{amsfonts}\n\\usepackage{xcolor}\n\\usepackage{siunitx}\n\\usepackage[utf8]{inputenc}\n\\usepackage{tikz}\n\\usepackage{tikz-cd}\n\n\\thispagestyle{empty}\n\\begin{document}\n\\color{black}\n\\[ x^2+1=0 \\]\n\\end{document}\n");
    }
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum RenderBackendError {
    #[error("vital container image not present")]
    ImageNotPresent,
    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait RenderBackend {
    fn render(&mut self) -> Result<Vec<u8>>;
}

pub mod native {

    pub enum OutputLog {
        Success,
        Error(Vec<String>),
    }

    use anyhow::Error;
    use tectonic::{status::{self, StatusBackend, plain::PlainStatusBackend, ChatterLevel, MessageKind}, ctry, config, driver::{ProcessingSessionBuilder, self}};

    use super::*;

    pub struct RenderInstanceNative {
        root: PathBuf,
        content: RenderContent,
        pub logs: Vec<LogRecord>,
    }

    pub struct TAIStatusBackend {
        always_stderr: bool,
        chatter: ChatterLevel,
        plain: PlainStatusBackend,
        logs: Vec<LogRecord>,
    }

    #[derive(Debug, Clone)]
    pub struct LogRecord {
        kind: status::MessageKind,
        args: String,
    }

    impl TAIStatusBackend {

        pub fn new(chatter: ChatterLevel) -> Self {
            Self {
                always_stderr: false,
                chatter,
                plain: PlainStatusBackend::new(chatter),
                logs: Vec::new(),
            }
        }

        pub fn always_stderr(&mut self, setting: bool) -> &mut Self {
            self.plain.always_stderr(setting);
            self
        }

        pub fn logs(&self) -> &Vec<LogRecord> {
            &self.logs
        }

        pub fn into_logs(self) -> Vec<LogRecord> {
            self.logs
        }
    }

    impl StatusBackend for TAIStatusBackend {
        fn report(&mut self, kind: status::MessageKind, args: std::fmt::Arguments, err: Option<&anyhow::Error>) {

            let prefix = match kind {
                status::MessageKind::Note => "note:",
                status::MessageKind::Warning => "warning:",
                status::MessageKind::Error => "error:",
            };

            if kind == MessageKind::Note && !self.always_stderr {

                if !self.chatter.suppress_message(kind) {
                    println!("{} {}", prefix, args);
                }

                self.logs.push(LogRecord { kind, args: args.to_string()});
            } else {
                if !self.chatter.suppress_message(kind) {
                    eprintln!("{} {}", prefix, args);
                }
                self.logs.push(LogRecord { kind, args: args.to_string()});
            }

            if let Some(e) = err {
                for item in e.chain() {
                    if !self.chatter.suppress_message(kind) {
                        println!("{} {}", prefix, args);
                    }
                    self.logs.push(LogRecord { kind, args: args.to_string()});
                }
            }
        }

        fn report_error(&mut self, err: &Error) {
            let mut prefix = "error";

            for item in err.chain() {
                eprintln!("{}: {}", prefix, item);
                prefix = "caused by";
            }
        }

        fn dump_error_logs(&mut self, output: &[u8]) {
            self.plain.dump_error_logs(output);
        }
    }

    impl RenderInstanceNative {
        pub fn new<P: Into<PathBuf>>(root: P, content: RenderContent) -> Self {
            Self {
                logs: Vec::new(),
                root: root.into(),
                content,
            }
        }

        fn parse_output_log(&mut self, source: &str) {
            source.split("!");
        }

        fn create_tex(&self) -> Vec<u8> {
            let data = self.content.as_tex().as_bytes().to_vec();
            data
        }

        fn create_pdf(&mut self, tex: &[u8]) -> Result<Vec<u8>> {
            let mut status = TAIStatusBackend::new(ChatterLevel::Normal);

            let auto_create_config_file = false;
            let config = config::PersistentConfig::open(auto_create_config_file)?;

            let only_cached = false;
            let bundle = config.default_bundle(only_cached, &mut status)?;

            let format_cache_path = config.format_cache_path()?;

            let mut files = {
                let mut sb = ProcessingSessionBuilder::default();
                sb.bundle(bundle)
                    .primary_input_buffer(tex)
                    .tex_input_name("texput.tex")
                    .format_name("latex")
                    .format_cache_path(format_cache_path)
                    .keep_logs(true)
                    .keep_intermediates(false)
                    .output_format(driver::OutputFormat::Pdf)
                    .do_not_write_output_files();

                let mut sess = sb.create(&mut status)?;
                match sess.run(&mut status) {
                    Ok(_) => "success",
                    Err(_) => "err",
                };

                println!("s {:?}", status.logs().iter().map(|rec| &rec.args).collect::<Vec<_>>());

                sess.into_file_data()
            };

            self.logs = status.into_logs();

            let data = files.remove("texput.pdf").unwrap().data;
            Ok(data)
        }

        fn create_png(&self, pdf: Vec<u8>) -> Result<Vec<u8>> {
            let mut output: Vec<u8> = Vec::new();

            println!("{:?}", self.root);

            let mut path = self.root.clone();
            path.push("texput");
            path.set_extension("pdf");

            let mut file = File::create(path)?;
            file.write_all(&pdf[..])?;

            // dvisvgm --no-fonts --scale={} --exact equation.dv

            Command::new("pdfcrop")
                .arg("texput.pdf")
                .current_dir(&self.root).output().unwrap();

            Command::new("dvisvgm")
                .arg("texput-crop.pdf")
                .arg("--no-fonts")
                .arg(format!("--scale={}", self.content.options.scale.unwrap_or(2.)))
                .arg("--pdf=texput-crop.pdf")
                .current_dir(&self.root)
                .env("LIBGS", "/usr/lib/libgs.so")
                .env("GS_OPTIONS", "-dNEWPDF=false")
                .output();

            let mut svg_opt = usvg::Options::default();
            svg_opt.resources_dir = std::fs::canonicalize(&self.root)
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()));
            svg_opt.fontdb.load_system_fonts();

            let mut svg_path = self.root.clone();
            svg_path.push("texput-crop");
            svg_path.set_extension("svg");

            let svg_data = std::fs::read(&svg_path)?;

            let rtree = usvg::Tree::from_data(&svg_data, &svg_opt.to_ref())?;
            let pixmap_size = rtree.svg_node().size.to_screen_size();
            let mut pixmap =
                tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
            resvg::render(
                &rtree,
                usvg::FitTo::Original,
                tiny_skia::Transform::default(),
                pixmap.as_mut(),
            )
            .unwrap();

            let mut png_path = self.root.clone();
            png_path.push("texput");
            png_path.set_extension("png");

            pixmap.save_png(&png_path).unwrap();

            let data = std::fs::read(png_path)?;
            Ok(data)
        }
    }

    impl RenderBackend for RenderInstanceNative {
        fn render(&mut self) -> Result<Vec<u8>> {
            let tex = self.create_tex();
            let pdf = self.create_pdf(&tex)?;
            let png = self.create_png(pdf)?;

            let mut path = self.root.clone();
            path.push("out");
            path.set_extension("png");

            let mut file = File::create(path)?;
            file.write(&png)?;

            Ok(png.to_vec())
        }
    }
}

pub mod containerised {
    use super::*;

    pub enum RenderOutputLog {
        Success,
        Failure(Vec<String>),
    }

    pub struct RenderInstanceCont {
        root: PathBuf,
        content: RenderContent,
        output: Option<RenderOutputLog>,
    }

    /// Structural impl.
    impl RenderInstanceCont {
        pub fn new<P: Into<PathBuf>>(root: P, content: RenderContent) -> Self {
            Self {
                root: root.into(),
                content,
                output: None,
            }
        }

        pub fn root(&self) -> &PathBuf {
            &self.root
        }

        pub fn root_mut(&mut self) -> &mut PathBuf {
            &mut self.root
        }

        pub fn content(&self) -> &RenderContent {
            &self.content
        }

        pub fn content_mut(&mut self) -> &mut RenderContent {
            &mut self.content
        }
    }

    /// Functional impl.
    impl RenderInstanceCont {
        // TODO:
        // docker_cmd() - Change the utilisation of docker API from process fork to a crate one

        fn parse_output_log(s: &str) -> RenderOutputLog {
            unimplemented!()
        }

        pub fn create_tex(&self) -> Vec<u8> {
            println!("{}", self.content().as_tex());

            self.content.as_tex().into_bytes()
        }

        fn docker_cmd(&self) -> Result<RenderOutputLog> {
            let cmd = Command::new("docker")
                .arg("run")
                .arg("--rm")
                .arg("-i")
                .arg("--user=1000:1000")
                .arg("--net=none")
                .arg("-v")
                .arg(format!("{}:/data", self.root().as_path().to_str().unwrap()))
                .arg("blang/latex:ubuntu")
                .arg("/bin/bash")
                .arg("-c")
                .arg(format!("timeout 5 latex -no-shell-escape -interaction=nonstopmode -halt-on-error equation.tex && timeout 5 dvisvgm --no-fonts --scale={} --exact equation.dvi", self.content().options.scale.map_or(4.0, |f| f)))
                .output()?;

            println!("{}", String::from_utf8(cmd.stdout).unwrap());

            Ok(RenderOutputLog::Success)
        }

        fn render_png(&mut self) -> Result<Vec<u8>> {
            let mut svg_opt = usvg::Options::default();
            svg_opt.resources_dir = std::fs::canonicalize(&self.root())
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()));
            svg_opt.fontdb.load_system_fonts();

            let mut svg_path = self.root().clone();
            svg_path.push("equation");
            svg_path.set_extension("svg");

            let svg_data = std::fs::read(&svg_path)?;

            let rtree = usvg::Tree::from_data(&svg_data, &svg_opt.to_ref())?;
            let pixmap_size = rtree.svg_node().size.to_screen_size();
            let mut pixmap =
                tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
            resvg::render(
                &rtree,
                usvg::FitTo::Original,
                tiny_skia::Transform::default(),
                pixmap.as_mut(),
            )
            .unwrap();

            let mut png_path = self.root.clone();
            png_path.push("equation");
            png_path.set_extension("png");

            pixmap.save_png(&png_path).unwrap();

            let data = std::fs::read(png_path)?;
            Ok(data)
        }
    }

    impl RenderBackend for RenderInstanceCont {
        fn render(&mut self) -> Result<Vec<u8>> {
            let tex = self.create_tex();

            let mut tex_path = self.root.clone();
            tex_path.push("equation");
            tex_path.set_extension("tex");

            let mut tex_file = File::create(&tex_path)?;

            tex_file.write_all(&tex)?;
            self.docker_cmd()?;

            Ok(self.render_png()?)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use mktemp::Temp;

        #[test]
        fn usepackage() {
            let usepackage = r#"\usepackage{{}}"#;

            let t = usepackage.replace("{}", "jea");

            assert_eq!(r#"\usepackage{jea}"#, t);
        }

        #[test]
        fn render() {
            let tmp_dir = Temp::new_dir().unwrap();

            let rc = RenderContent::new("x^2+1=0".to_string());
            let mut ri = RenderInstanceCont::new(tmp_dir.as_path(), rc);

            let out = ri.render().unwrap();
            assert_eq!(out.is_empty(), false);
        }
    }
}
