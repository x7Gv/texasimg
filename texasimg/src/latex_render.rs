use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use cairo::{Context, Format};
use poppler::PopplerDocument;
use tectonic::{
    config, ctry,
    driver::{self, ProcessingSession, ProcessingSessionBuilder},
    errmsg,
    status::self,
};

const DEFAULT_IMPORTS: &'static str = r#"\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsfonts}
\usepackage{xcolor}
\usepackage{siunitx}
\usepackage[utf8]{inputenc}
\usepackage{tikz}
\usepackage{tikz-cd}
"#;

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
pub struct RenderContentImports {
    data: String,
}
impl Default for RenderContentImports {
    fn default() -> Self {
        let data = DEFAULT_IMPORTS.to_string();
        Self { data }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct RenderContentOptions {
    ink_colour: ContentColour,
    formula_mode: FormulaMode,
    imports: RenderContentImports,
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
            documentclass, self.options.imports.data, pagestyle, begin, color, formula, end
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

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Renderer {
    fn render(&mut self) -> Result<Vec<u8>>;
}

pub mod native {

    use super::*;

    pub struct RenderInstanceNative {
        root: PathBuf,
        content: RenderContent,
    }

    impl RenderInstanceNative {
        pub fn new<P: Into<PathBuf>>(root: P, content: RenderContent) -> Self {
            Self {
                root: root.into(),
                content,
            }
        }

        fn create_tex(&self) -> Vec<u8> {
            let data = self.content.as_tex().as_bytes().to_vec();
            data
        }

        fn create_pdf(&self, tex: &[u8]) -> Result<Vec<u8>> {
            let mut status = status::NoopStatusBackend::default();

            let auto_create_config_file = false;
            let config = ctry!(config::PersistentConfig::open(auto_create_config_file);
                           "failed to open the default configuration file");

            let only_cached = false;
            let bundle = ctry!(config.default_bundle(only_cached, &mut status);
                           "failed to load the default resource bundle");

            let format_cache_path = ctry!(config.format_cache_path();
                                      "failed to set up the format cache");

            let mut files = {
                let mut sb = ProcessingSessionBuilder::default();
                sb.bundle(bundle)
                    .primary_input_buffer(tex)
                    .tex_input_name("texput.tex")
                    .format_name("latex")
                    .format_cache_path(format_cache_path)
                    .keep_logs(false)
                    .keep_intermediates(false)
                    .print_stdout(false)
                    .output_format(driver::OutputFormat::Xdv)
                    .do_not_write_output_files();

                let mut sess = ctry!(sb.create(&mut status); "failed to initialize the LaTeX processing session");
                ctry!(sess.run(&mut status); "the LaTeX engine failed");
                sess.into_file_data()
            };

            let data = files.remove("texput.xdv").unwrap().data;
            Ok(data)
        }

        fn create_png(&self, pdf: Vec<u8>) -> Result<Vec<u8>> {
            let mut output: Vec<u8> = Vec::new();

            let mut path = self.root.clone();
            path.push("texput");
            path.set_extension("dvi");

            let mut file = File::create(path)?;
            file.write_all(&pdf[..])?;

            let dvipng_cmd = Command::new("dvipng")
                .arg("texput.dvi")
                .current_dir(&self.root)
                .spawn()?;

            let mut out_path = self.root.clone();
            out_path.push("texput");
            out_path.set_extension("png");
            let mut out = File::open(out_path)?;
            out.read(&mut output[..]).unwrap();

            Ok(output)
        }
    }

    impl Renderer for RenderInstanceNative {
        fn render(&mut self) -> Result<Vec<u8>> {
            let tex = self.create_tex();
            let pdf = self.create_pdf(&tex)?;
            let png = self.create_png(pdf)?;

            let mut file = File::create("out.png")?;
            file.write(&png)?;

            Ok(png.to_vec())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::RenderContent;

        #[test]
        pub fn content() {
            let r_c = RenderContent::new("a + b = c".to_string()).as_tex();
            assert_eq!(
            r_c,
            "\\documentclass[12pt]{article}\n\\usepackage{amsmath}\n\\usepackage{amssymb}\n\\usepackage{amsfonts}\n\\usepackage{xcolor}\n\\usepackage{siunitx}\n\\usepackage[utf8]{inputenc}\n\\usepackage{tikz}\n\\usepackage{tikz-cd}\n\n\\thispagestyle{empty}\n\\begin{document}\n\\color{black}\n\\[ a + b = c \\]\n\\end{document}\n".to_string())
        }
    }
}

mod containerised {
    use super::*;

    pub struct RenderInstanceCont {
        root: PathBuf,
        content: RenderContent,
    }

    /// Structural impl.
    impl RenderInstanceCont {
        pub fn new<P: Into<PathBuf>>(root: P, content: RenderContent) -> Self {
            Self {
                root: root.into(),
                content,
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

        pub fn create_tex(&self) -> Vec<u8> {
            self.content.as_tex().into_bytes()
        }

        fn docker_cmd(&self) -> Result<()> {

            let _cmd = Command::new("docker")
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
                .arg(format!("timeout 5 latex -no-shell-escape -interaction=nonstopmode -halt-on-error equation.tex && timeout 5 dvisvgm --no-fonts --scale={} --exact equation.dvi", 2.0))
                .output()?;

            Ok(())
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

    impl Renderer for RenderInstanceCont {
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
        fn render() {

            let tmp_dir = Temp::new_dir().unwrap();

            let rc = RenderContent::new("x^2+1=0".to_string());
            let mut ri = RenderInstanceCont::new(tmp_dir.as_path(), rc);

            let out = ri.render().unwrap();
            assert_eq!(out.is_empty(), false);
        }
    }
}
