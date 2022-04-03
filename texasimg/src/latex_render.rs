use std::path::PathBuf;

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
            FormulaMode::Inline => format!(r#"\({}\)"#, formula_content),
            FormulaMode::Displayed => format!(r#"\[{}\]"#, formula_content),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ContentColour {
    Black,
    White,
    RGB((u8, u8, u8)),
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
        unimplemented!()
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

pub struct RenderInstance {
    root: PathBuf,
    content: String,
}
