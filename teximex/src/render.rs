use crate::{document::Document, tex::TexString};

use self::state::{Loaded, Unloaded};

#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    scale: Option<f32>,
    margin: Option<f32>,
}

impl RenderOptions {
    pub fn new(scale: Option<f32>, margin: Option<f32>) -> Self {
        Self { scale, margin }
    }

    pub fn scale(&self) -> f32 {
        self.scale.unwrap_or(2.0)
    }

    pub fn margin(&self) -> f32 {
        self.margin.unwrap_or(4.0)
    }
}

pub mod state {
    pub struct Unloaded;
    pub struct Loaded;
}

pub struct RenderInstance<T: TexString = String, State = Unloaded> {
    pub options: RenderOptions,
    pub last_document: Option<Document<T>>,
    _state: std::marker::PhantomData<State>,
}

pub trait RenderBackend {
    fn render(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl<T: TexString> RenderInstance<T> {
    pub fn new() -> RenderInstance<T, Unloaded> {
        RenderInstance::<T, Unloaded> {
            options: RenderOptions::default(),
            last_document: None,
            _state: std::marker::PhantomData::default(),
        }
    }

    pub fn new_with_options(options: RenderOptions) -> Self {
        Self {
            options,
            last_document: None,
            _state: std::marker::PhantomData::default(),
        }
    }

    pub fn load(self, document: Document<T>) -> RenderInstance<T, Loaded> {
        RenderInstance::<T, Loaded> {
            options: self.options,
            last_document: Some(document),
            _state: std::marker::PhantomData::default(),
        }
    }
}

impl<T: TexString> RenderInstance<T, Loaded> {
    pub fn document(&self) -> &Document<T> {
        &self.last_document.as_ref().unwrap()
    }
}

pub mod log {
    #[derive(Debug, Clone)]
    pub struct LogRecord {
        pub args: String,
    }
}

pub mod native {
    use std::{fs::File, io::{Write, Stdout}, path::PathBuf, process::{Command, Stdio}, fmt::Display};
/*
    use tectonic::{
        config,
        driver::{self, ProcessingSessionBuilder},
        status::{plain::PlainStatusBackend, ChatterLevel, MessageKind, StatusBackend},
    };
*/

    use crate::tex::TexString;

    use super::{state::Loaded, RenderBackend, RenderInstance};

    #[derive(Debug, Clone)]
    pub struct PdflatexLogRecord {
        line: String,
        info: String,
        content: String,
    }

    #[derive(Debug, Clone)]
    pub enum LogRecord {
        Pdflatex(Vec<PdflatexLogRecord>),
    }

    pub fn parse_pdflatex_logs(input: &str) -> Result<Vec<PdflatexLogRecord>, Box<dyn std::error::Error>> {

        let re = regex::Regex::new(r"!(.*?)\n(l\.\d+) (.*?)(\n!|\n\(|\n|$)")?;

        let mut res = Vec::new();

        for cap in re.captures_iter(input) {
            let info = cap.get(1).unwrap().as_str();
            let line = cap.get(2).unwrap().as_str();
            let content = cap.get(3).unwrap().as_str();

            res.push(PdflatexLogRecord {
                line: line.to_string(),
                info: info.to_string(),
                content: content.to_string(),
            })
        }

        return Ok(res)
    }

    // #[derive(Debug, Clone)]
    // pub struct NativeLogRecord {
    //     pub kind: tectonic::status::MessageKind,
    //     pub args: String,
    // }

    pub struct RenderInstanceNative {
        pub instance: RenderInstance<String, Loaded>,
        pub path_root: PathBuf,
        pub logs: Vec<LogRecord>,
    }

    impl RenderInstanceNative {
        pub fn new<P: Into<PathBuf>>(root: P, instance: RenderInstance<String, Loaded>) -> Self {
            Self {
                instance,
                path_root: root.into(),
                logs: Vec::new(),
            }
        }

        fn create_tex(&self) -> Vec<u8> {
            self.instance.document().to_tex().as_bytes().to_vec()
        }

        fn _create_dvi(&mut self, tex: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            
            println!("{:?}", self.path_root);

            let mut tp_path = self.path_root.clone();
            tp_path.push("texput");
            tp_path.set_extension("tex");

            let mut texput = File::create(&tp_path)?;
            texput.write_all(tex)?;

            let pdflatex = Command::new("pdflatex")
                .arg("-jobname=texput")
                .arg("-output-format=dvi")
                .arg("-interaction=nonstopmode")
                .arg("texput.tex")
                .current_dir(&self.path_root)
                .output()?;

            let output = String::from_utf8_lossy(&pdflatex.stdout);

            let logs = parse_pdflatex_logs(&output);

            println!("{:?}", logs);

            self.logs.push(LogRecord::Pdflatex(logs.unwrap()));

            let mut tp_path_dvi = self.path_root.clone();
            tp_path_dvi.push("texput");
            tp_path_dvi.set_extension("dvi");

            let data = std::fs::read(&tp_path_dvi)?;

            Ok(data)
        }

        /*

        fn create_dvi(&mut self, tex: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut status = StoredStatusBackend::new(ChatterLevel::Normal, self);

            let auto_create_config = false;
            let config = config::PersistentConfig::open(auto_create_config)?;

            let only_cached = false;
            let bundle = config.default_bundle(only_cached, &mut status)?;

            let format_cache_path = config.format_cache_path()?;

            let mut files = {
                let mut session_builder = ProcessingSessionBuilder::default();
                session_builder
                    .output_format(driver::OutputFormat::Xdv)
                    .bundle(bundle)
                    .primary_input_buffer(tex)
                    .tex_input_name("texput.tex")
                    .format_name("latex")
                    .format_cache_path(format_cache_path)
                    .keep_logs(true)
                    .keep_intermediates(false)
                    .do_not_write_output_files();

                let mut session = session_builder.create(&mut status)?;
                session.run(&mut status)?;
                session.into_file_data()
            };

            println!("{:?}", self.logs);

            let data = files.remove("texput.xdv").unwrap().data;
            Ok(data)
        }
        */

        fn create_png(&self, dvi: Vec<u8>) -> anyhow::Result<Vec<u8>> {

            dbg!("{:?}", &self.path_root);

            let mut path = self.path_root.clone();
            path.push("texput2");
            path.set_extension("dvi");

            let mut file = File::create(path)?;
            file.write_all(&dvi[..])?;

            Command::new("dvisvgm")
                .arg("texput2.dvi")
                .arg("--no-fonts")
                .arg(format!("--scale={}", self.instance.options.scale()))
                .current_dir(&self.path_root)
                .output()?;

            let mut svg_opt = usvg::Options::default();
            svg_opt.resources_dir = std::fs::canonicalize(&self.path_root)
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()));
            svg_opt.fontdb.load_system_fonts();

            let mut svg_path = self.path_root.clone();
            svg_path.push("texput2");
            svg_path.set_extension("svg");

            let svg_data = std::fs::read(&svg_path)?;

            let rtree = usvg::Tree::from_data(&svg_data, &svg_opt.to_ref())?;
            let pixmap_size = rtree.size.to_screen_size();
            let mut pixmap =
                tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

            resvg::render(
                &rtree,
                usvg::FitTo::Original,
                tiny_skia::Transform::default(),
                pixmap.as_mut(),
            )
            .unwrap();

            let mut png_path = self.path_root.clone();
            png_path.push("texput2");
            png_path.set_extension("png");

            pixmap.save_png(&png_path)?;

            let data = std::fs::read(png_path)?;
            Ok(data)
        }
    }

    impl RenderBackend for RenderInstanceNative {
        fn render(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let tex = self.create_tex();
            let dvi = self._create_dvi(&tex)?;
            let png = self.create_png(dvi)?;

            let mut path = self.path_root.clone();
            path.push("out");
            path.set_extension("png");

            let mut file = File::create(path)?;
            file.write(&png)?;

            Ok(png.to_vec())
        }
    }

    // pub struct StoredStatusBackend<'a> {
    //     always_stderr: bool,
    //     chatter: ChatterLevel,
    //     plain: PlainStatusBackend,
    //     logs: Vec<NativeLogRecord>,
    //     instance: &'a mut RenderInstanceNative,
    // }

    // impl<'a> StoredStatusBackend<'a> {
    //     pub fn new(chatter: ChatterLevel, instance: &'a mut RenderInstanceNative) -> Self {
    //         Self {
    //             chatter,
    //             always_stderr: false,
    //             plain: PlainStatusBackend::new(chatter),
    //             logs: Vec::new(),
    //             instance,
    //         }
    //     }

    //     pub fn always_stderr(&mut self, setting: bool) -> &mut Self {
    //         self.plain.always_stderr(setting);
    //         self
    //     }

    //     pub fn logs(&self) -> &Vec<NativeLogRecord> {
    //         &self.logs
    //     }

    //     pub fn into_logs(self) -> Vec<NativeLogRecord> {
    //         self.logs
    //     }
    // }

    // impl StatusBackend for StoredStatusBackend<'_> {
    //     fn report(
    //         &mut self,
    //         kind: MessageKind,
    //         args: std::fmt::Arguments,
    //         err: Option<&anyhow::Error>,
    //     ) {
    //         let prefix = match kind {
    //             MessageKind::Note => "note:",
    //             MessageKind::Warning => "warning:",
    //             MessageKind::Error => "error:",
    //         };

    //         if kind == MessageKind::Note && !self.always_stderr {
    //             if !self.chatter.suppress_message(kind) {
    //                 println!("{} {}", prefix, args);
    //             }

    //             let rec = NativeLogRecord {
    //                 kind,
    //                 args: args.to_string(),
    //             };

    //             self.instance.logs.push(rec);
    //         } else {
    //             if !self.chatter.suppress_message(kind) {
    //                 eprintln!("{} {}", prefix, args);
    //             }
    //             let rec = NativeLogRecord {
    //                 kind,
    //                 args: args.to_string(),
    //             };

    //             self.instance.logs.push(rec);
    //         }

    //         if let Some(e) = err {
    //             for _item in e.chain() {
    //                 if !self.chatter.suppress_message(kind) {
    //                     println!("{} {}", prefix, args);
    //                 }
    //                 let rec = NativeLogRecord {
    //                     kind,
    //                     args: args.to_string(),
    //                 };

    //                 self.instance.logs.push(rec);
    //             }
    //         }
    //     }

    //     fn report_error(&mut self, err: &anyhow::Error) {
    //         let mut prefix = "error";

    //         for item in err.chain() {
    //             eprintln!("{}: {}", prefix, item);
    //             prefix = "caused by";
    //         }
    //     }

    //     fn note_highlighted(&mut self, before: &str, highlighted: &str, after: &str) {
    //         self.report(
    //             MessageKind::Note,
    //             format_args!("{}{}{}", before, highlighted, after),
    //             None,
    //         )
    //     }

    //     fn dump_error_logs(&mut self, output: &[u8]) {
    //         self.plain.dump_error_logs(output);
    //     }
    // }
}
