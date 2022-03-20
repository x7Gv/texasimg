use std::{path::PathBuf, marker::PhantomData};

pub struct Constructing;
pub struct Instantiated;
pub struct Rendered;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RenderInstance<State = Constructing> {
    root: PathBuf,
    content: RenderContent,
    state: std::marker::PhantomData<State>,
}

impl<State> RenderInstance<State> {
    pub fn builder(root: impl Into<PathBuf>) -> RenderInstanceBuilder {
        RenderInstanceBuilder::new(root)
    }
}

#[derive(Default)]
pub struct RenderInstanceBuilder {
    root: PathBuf,
    content: Option<RenderContent>,
}

impl RenderInstanceBuilder {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            content: None,
        }
    }

    pub fn content(&mut self, content: impl Into<RenderContent>) -> &mut Self {
        let mut new = self;
        new.content = Some(content.into());
        new
    }

    pub fn build(&self) -> Option<RenderInstance<Instantiated>> {
        if let Some(content) = &self.content {
            Some(RenderInstance {
                root: Clone::clone(&self.root),
                content: Clone::clone(&content),
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
}

impl RenderContent {
    pub fn builder(content: String) -> RenderContentBuilder {
        RenderContentBuilder::new(content)
    }
}

#[derive(Default, Clone)]
pub struct RenderContentBuilder {
    colour: ContentColour,
    mode: ContentMode,
    content: String,
}

impl RenderContentBuilder {
    pub fn new(content: String) -> Self {
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
    fn render_instance() {
        let r_c = RenderContent::builder("test".to_string()).build();
        let r_i = RenderInstance::builder("/tmp/test")
            .content(r_c.clone())
            .build()
            .unwrap();

        assert_eq!(r_i.content, r_c);
    }

    #[test]
    fn render_content() {
        let content = "test";

        let r_c = RenderContent::builder(content.to_string())
            .colour(ContentColour::Black)
            .build();

        assert_eq!(r_c.content, content);
        assert_eq!(r_c.colour, ContentColour::Black);
    }
}
