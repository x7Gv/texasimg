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
\usepackage{tikz}
\usepackage{tikz-cd}
"#;

fn default_imports() -> Vec<RenderContentImport> {
    DEFAULT_IMPORTS.split("\n").map(|s| RenderContentImport::Custom(s.to_string())).collect()
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FormulaMode {
    Inline,
    Displayed,
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
            RenderContentImport::Usepackage(val) => {
                USEPACKAGE.replace("{}", &val).to_string()
            },
            RenderContentImport::Custom(val) => {
                val.clone()
            }
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
        Self { data: default_imports() }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderContentOptions {
    pub ink_colour: ContentColour,
    pub formula_mode: FormulaMode,
    pub imports: RenderContentImports,
    pub scale: Option<f32>
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
        let documentclass = r#"\documentclass[12pt]{article}"#;
        let pagestyle = r#"\thispagestyle{empty}"#;
        let begin = r#"\begin{document}"#;
        let color = self.options.ink_colour.as_tex();
        let formula = self.options.formula_mode.as_tex(&self.formula_content);
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
            documentclass, self.options.imports.to_string(), pagestyle, begin, color, formula, end
        )
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
    use crate::latex_render::{RenderContentOptions, FormulaMode, ContentColour};

    use super::RenderContent;

    #[test]
    fn render_content_new() {
        {
            let rc = RenderContent::new("x^2+1=0".to_string());
            assert_eq!(rc.formula_content, "x^2+1=0");
        }

        {
            let mut rco = RenderContentOptions::default();
            rco.formula_mode = FormulaMode::Inline;
            rco.ink_colour = ContentColour::White;

            let rc = RenderContent::new_with_options("x^2+1=0".to_string(), rco.clone());
            assert_eq!(rc.formula_content, "x^2+1=0");
            assert_eq!(rc.options, rco);
        }
    }

    #[test]
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
    Unknown
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait RenderBackend {
    fn render(&mut self) -> Result<Vec<u8>>;
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
        use mktemp::Temp;
        use super::*;

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
