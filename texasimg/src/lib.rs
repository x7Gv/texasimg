mod latex_render;

use std::{fs::File, io::Write, marker::PhantomData, path::PathBuf, process::Command};

use image::EncodableLayout;

pub struct Constructing;
pub struct Instantiated;
pub struct Rendered;

pub enum ModefulContent {
    Inline(String),
    Displayed(String),
}

fn default_imports() -> String {
    r#"\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsfonts}
\usepackage{xcolor}
\usepackage{siunitx}
\usepackage[utf8]{inputenc}
\usepackage{tikz}
\usepackage{tikz-cd}"#
        .to_string()
}

fn latex_template(content: ModefulContent, imports: String) -> String {
    match content {
        ModefulContent::Inline(content) => {
            format!(
                r#"\documentclass[12pt]{}
{}
\thispagestyle{}
\begin{}
\color{}
\( {} \)
\end{}"#,
                "{article}", imports, "{empty}", "{document}", "{white}", content, "{document}"
            )
        }
        ModefulContent::Displayed(content) => {
            format!(
                r#"\documentclass[12pt]{}
{}
\thispagestyle{}
\begin{}
\color{}
\[ {} \]
\end{}"#,
                "{article}", imports, "{empty}", "{document}", "{white}", content, "{document}"
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderInstance<State = Constructing> {
    root: PathBuf,
    content: RenderContent,
    png: Option<Vec<u8>>,
    stdout: Option<Vec<u8>>,
    scale: Option<f32>,
    state: std::marker::PhantomData<State>,
}

impl RenderInstance {
    pub fn builder() -> RenderInstanceBuilder {
        let tmp = {
            Command::new("mktemp")
                .arg("--dir")
                .output()
                .expect("failed to retrieve temp directory.")
        };

        let mut tmp_path = String::from_utf8_lossy(&tmp.stdout).to_string();
        tmp_path.truncate(tmp_path.len() - 1);

        return RenderInstanceBuilder::new(tmp_path);
    }
}

impl<Instantiated> RenderInstance<Instantiated> {
    pub fn render(&mut self) -> Result<RenderInstance<Rendered>, Box<dyn std::error::Error>> {
        self.create_tex()?;
        self.docker_cmd()?;

        let data = self.render_png()?;

        Ok(RenderInstance {
            root: self.root.clone(),
            content: self.content.clone(),
            png: Some(data).clone(),
            scale: self.scale.clone(),
            stdout: None,
            state: PhantomData::default(),
        })
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
    pub fn scale(&self) -> f32 {
        self.scale.unwrap()
    }

    fn create_tex(&self) -> Result<File, Box<dyn std::error::Error>> {
        let mut path = self.root.clone();
        path.push("equation");
        path.set_extension("tex");

        let mut file = File::create(path)?;
        file.write_all(self.content().content().as_bytes())?;

        Ok(file)
    }

    fn docker_cmd(&self) -> Result<(), Box<dyn std::error::Error>> {
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
            .arg(format!("timeout 5 latex -no-shell-escape -interaction=nonstopmode -halt-on-error equation.tex && timeout 5 dvisvgm --no-fonts --scale={} --exact equation.dvi", self.scale()))
            .output()?;

        println!("{}", String::from_utf8_lossy(&_cmd.stdout));

        Ok(())
    }

    fn render_png(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
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

impl<Rendered> RenderInstance<Rendered> {
    pub fn clean(self) {
        std::fs::remove_dir_all(self.root).unwrap();
    }

    pub fn png(&self) -> Vec<u8> {
        self.png.clone().unwrap()
    }
}

#[derive(Default)]
pub struct RenderInstanceBuilder {
    root: PathBuf,
    content: Option<RenderContent>,
    scale: Option<f32>,
}

impl RenderInstanceBuilder {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            content: None,
            scale: None,
        }
    }

    pub fn content(&mut self, content: impl Into<RenderContent>) -> &mut Self {
        let mut new = self;
        new.content = Some(content.into());
        new
    }

    pub fn scale(&mut self, scale: f32) -> &mut Self {
        let mut new = self;
        new.scale = Some(scale);
        new
    }

    pub fn build(&self) -> Option<RenderInstance<Instantiated>> {
        if let Some(content) = &self.content {
            Some(RenderInstance {
                root: Clone::clone(&self.root),
                content: Clone::clone(&content),
                png: Some(Vec::new()),
                scale: self.scale.map_or(Some(1.0), |value| Some(value)),
                stdout: None,
                state: PhantomData::default(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RenderContent {
    colour: ContentColour,
    mode: ContentMode,
    content: String,
    imports: String,
}

pub enum ContentKind {
    Formula(ModefulContent),
    Raw(String),
}

impl RenderContent {
    pub fn builder(content: ContentKind) -> RenderContentBuilder {
        let input = match content {
            ContentKind::Formula(formula) => latex_template(formula),
            ContentKind::Raw(raw) => raw,
        };

        RenderContentBuilder::new(input)
    }

    pub fn colour(&self) -> &ContentColour {
        &self.colour
    }

    pub fn colour_mut(&mut self) -> &mut ContentColour {
        &mut self.colour
    }

    pub fn mode(&self) -> &ContentMode {
        &self.mode
    }

    pub fn mode_mut(&mut self) -> &mut ContentMode {
        &mut self.mode
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }
}

#[derive(Default, Clone)]
pub struct RenderContentBuilder {
    colour: ContentColour,
    mode: ContentMode,
    content: String,
}

impl RenderContentBuilder {
    pub fn new(content: String, modified_imports: Option<String>) -> Self {
        let mut new = Self::default();
        new.content = content;
        new
    }

    pub fn colour(&mut self, colour: ContentColour) -> &mut Self {
        let mut new = self;
        new.colour = colour;
        new
    }

    pub fn mode(&mut self, mode: ContentMode) -> &mut Self {
        let mut new = self;
        new.mode = mode;
        new
    }

    pub fn build(&self) -> RenderContent {
        RenderContent {
            colour: self.colour,
            mode: self.mode,
            content: Clone::clone(&self.content),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ContentMode {
    Displayed,
    Inline,
}

impl Default for ContentMode {
    fn default() -> Self {
        ContentMode::Displayed
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ContentColour {
    Black,
    White,
}

impl Default for ContentColour {
    fn default() -> Self {
        ContentColour::Black
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn testedi_test_est() {
        todo!()
    }
}
