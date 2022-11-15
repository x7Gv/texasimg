use crate::tex::{Color, MathMode, TexString};
use std::marker::PhantomData;

const DEFAULT_IMPORTS: &'static str = r#"\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsfonts}
\usepackage{xcolor}
\usepackage{siunitx}
\usepackage[utf8]{inputenc}
"#;

/// Represents options for documents.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentOptions {
    /// Color to be applied to the document text.
    pub text_color: Color,
    /// Preamble to be put before the begin document.
    pub preamble: String,
}

impl Default for DocumentOptions {
    fn default() -> Self {
        Self {
            text_color: Color::default(),
            preamble: DEFAULT_IMPORTS.to_string(),
        }
    }
}

/// Represents a document to be rendered.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document<T: TexString> {
    options: DocumentOptions,
    content: T,
}

impl<T: TexString> Document<T> {
    pub fn new(content: T) -> Self {
        Self {
            options: DocumentOptions::default(),
            content,
        }
    }

    pub fn builder(content: T) -> DocumentBuilder {
        DocumentBuilder::new(content)
    }

    pub fn new_with_options(content: T, options: DocumentOptions) -> Self {
        Self { options, content }
    }

    pub fn options(&self) -> &DocumentOptions {
        &self.options
    }

    pub fn content(&self) -> &T {
        &self.content
    }

    pub fn set_options(&mut self, options: DocumentOptions) -> &mut Self {
        self.options = options;
        self
    }

    pub fn set_content(&mut self, content: T) -> &mut Self {
        self.content = content;
        self
    }
}

impl<T: TexString> TexString for Document<T> {
    fn to_tex(&self) -> String {
        let documentclass = r#"\documentclass[12pt]{article}"#;
        let pagestyle = r#"\thispagestyle{empty}"#;
        let begin = r#"\begin{document}"#;
        let color = &self.options.text_color.to_tex();
        let content = &self.content.to_tex();
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
            &self.options.preamble.to_tex(),
            pagestyle,
            begin,
            color,
            content,
            end,
        )
    }
}

/// Refers to [`crate::tex::MathMode`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentMathMode {
    /// Refers to [`crate::tex::MathMode::Inline`]
    Inline,
    /// Refers to [`crate::tex::MathMode::Displayed`]
    Displayed,
}

impl DocumentMathMode {
    /// Transform from [`Self`] to [`crate::tex::MathMode`] applying a [`crate::tex::TexString`]
    pub fn transform<T: TexString>(&self, tex: T) -> MathMode<T> {
        match self {
            DocumentMathMode::Inline => MathMode::Inline(vec![tex]),
            DocumentMathMode::Displayed => MathMode::Displayed(vec![tex]),
        }
    }
}

pub mod state {
    pub struct MathModeApplied;
    pub struct MathModeUnapplied;
}

pub struct DocumentBuilder<State = state::MathModeUnapplied> {
    options: DocumentOptions,
    content: String,
    _state: std::marker::PhantomData<State>,
}

// TODO: Optimise out redundant `.clone()`

impl<S> DocumentBuilder<S> {
    pub fn new<T: TexString>(content: T) -> Self {
        DocumentBuilder {
            options: DocumentOptions::default(),
            content: content.to_tex(),
            _state: PhantomData::default(),
        }
    }

    pub fn options(&mut self, options: DocumentOptions) -> &mut Self {
        self.options = options;
        self
    }

    pub fn add_preamble(&mut self, preamble: String) -> &mut Self {
        self.options.preamble.push_str(&preamble);
        self
    }

    pub fn color(&mut self, color: crate::tex::Color) -> &mut Self {
        let mut opt = self.options.clone();
        opt.text_color = color;
        self.options(opt)
    }

    pub fn build(self) -> Document<String> {
        Document {
            options: self.options,
            content: self.content,
        }
    }
}

impl DocumentBuilder<state::MathModeUnapplied> {
    pub fn mathmode(mut self, mode: DocumentMathMode) -> DocumentBuilder<state::MathModeApplied> {
        self.content = mode.transform(self.content.clone()).to_tex();
        DocumentBuilder::<state::MathModeApplied> {
            options: self.options,
            content: self.content,
            _state: PhantomData::default(),
        }
    }
}
