extern crate handlebars;
extern crate rustc_serialize;

pub mod exec;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use handlebars::Handlebars;
use rustc_serialize::json::{Json, ToJson};

#[macro_export]
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("Failed printing to stderr");
    } }
);

#[derive(Debug)]
pub struct Opts {
    pub template: String,
    pub output: String,
    pub tag_pattern: Option<String>,
    pub extra_vars: Option<String>,
}

impl Opts {
    pub fn new() -> Opts {
        Opts {
            template: String::new(),
            output: String::new(),
            tag_pattern: Option::None,
            extra_vars: Option::None,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    CommandError(String, std::io::Error),
    CommandOutputParsingError,
    TemplateError(TemplateError),
    OutputError(std::io::Error),
}

#[derive(Debug)]
pub enum TemplateError {
    IOError(std::io::Error),
    TemplateError(handlebars::TemplateError),
    RenderError(handlebars::RenderError),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TemplateError::IOError(ref e) => write!(f, "{}", e),
            TemplateError::TemplateError(ref e) => write!(f, "{}", e),
            TemplateError::RenderError(ref e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub enum ExitStatus {
    Success,
    Error(Error),
}

impl ExitStatus {
    pub fn exit_code(&self) -> i32 {
        match *self {
            ExitStatus::Success => 0,
            ExitStatus::Error(_) => 9,
        } 
    }
}

impl std::fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ExitStatus::Success => 
                write!(f, ""),
            ExitStatus::Error(Error::CommandError(ref command, ref e)) => 
                write!(f, "Failed to run command '{}'.\n{}", command, e),
            ExitStatus::Error(Error::CommandOutputParsingError) => 
                write!(f, "Failed to parse command output"),
            ExitStatus::Error(Error::TemplateError(ref e)) => 
                write!(f, "Failed to render template.\n{}", e),
            ExitStatus::Error(Error::OutputError(ref e)) => 
                write!(f, "Failed to write the output.\n{}", e),
        }
    }
}

#[derive(Debug)]
pub struct GitInfo {
    revision: String,
    branch: String,
    tags: Vec<String>,
}

impl ToJson for GitInfo {
    fn to_json(&self) -> Json {
        let mut obj: BTreeMap<String, Json> = BTreeMap::new();
        obj.insert("revision".to_string(), self.revision.to_json());
        obj.insert("branch".to_string(), self.branch.to_json());
        obj.insert("tags".to_string(), self.tags.to_json());
        let result = Json::Object(obj);
        result
    }
}

static DEFAULT_TEMPLATE_NAME: &'static str = "DEFAULT_TEMPLATE";

pub fn render_info(git_info: &GitInfo, opts: &Opts) -> Result<String, Error> {
    let mut handlebars = Handlebars::new();
    File::open(&opts.template)
        .map_err(|e| Error::TemplateError(TemplateError::IOError(e)))
        .and_then(|mut file| {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| Error::TemplateError(TemplateError::IOError(e)))
                .map(|_| content)
        })
        .and_then(|content| {
            handlebars.register_template_string(DEFAULT_TEMPLATE_NAME, content)
                .map_err(|e| Error::TemplateError(TemplateError::TemplateError(e)))
                .and_then(|_| {
                    handlebars.render(DEFAULT_TEMPLATE_NAME, git_info)
                        .map_err(|e| Error::TemplateError(TemplateError::RenderError(e)))
                })
        })
}

pub fn render_info_to_file(git_info: &GitInfo, opts: &Opts) -> Result<String, Error> {
    render_info(git_info, opts)
        .and_then(|rendered| {
            File::create(&opts.output)
                .map(|mut file| file.write_all(rendered.as_bytes()))
                .map_err(|e| Error::OutputError(e))
                .map(|_| rendered)
        })
}

pub fn render_to_file(opts: &Opts) -> Result<String, Error> {
    let info = try!(exec::git_info(&opts.tag_pattern));
    render_info_to_file(&info, opts)
}