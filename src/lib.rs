extern crate handlebars;
extern crate rustc_serialize;

pub mod exec;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use handlebars::Handlebars;
use rustc_serialize::json;
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
    pub debug: bool,
}

impl Opts {
    pub fn new() -> Opts {
        Opts {
            template: String::new(),
            output: String::new(),
            tag_pattern: Option::None,
            extra_vars: Option::None,
            debug: false,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    CommandError(String, std::io::Error),
    CommandOutputParsingError,
    TemplateError(TemplateError),
    OutputError(std::io::Error),
    JsonError(self::JsonError),
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
pub enum JsonError {
    NotOject,
    Error(json::ParserError),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            JsonError::NotOject => write!(f, "JSON Object expected"),
            JsonError::Error(ref e) => write!(f, "{}", e),
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
            ExitStatus::Success => write!(f, ""),
            ExitStatus::Error(Error::CommandError(ref command, ref e)) => {
                write!(f, "Failed to run command '{}'. {}", command, e)
            },
            ExitStatus::Error(Error::CommandOutputParsingError) => {
                write!(f, "Failed to parse command output")
            },
            ExitStatus::Error(Error::TemplateError(ref e)) => {
                write!(f, "Failed to render template. {}", e)
            },
            ExitStatus::Error(Error::OutputError(ref e)) => {
                write!(f, "Failed to write the output. {}", e)
            },
            ExitStatus::Error(Error::JsonError(ref e)) => {
                write!(f, "Failed to parse extra vars. {}", e)
            },
        }
    }
}

pub type Context = BTreeMap<String, Json>;

#[derive(Debug)]
pub struct GitInfo {
    revision: String,
    branch: String,
    tags: Vec<String>,
}

impl GitInfo {
    fn to_context(&self) -> Context {
        let mut obj = Context::new();
        obj.insert("revision".to_string(), self.revision.to_json());
        obj.insert("branch".to_string(), self.branch.to_json());
        obj.insert("tags".to_string(), self.tags.to_json());
        obj
    }
}

impl ToJson for GitInfo {
    fn to_json(&self) -> Json {
        Json::Object(self.to_context())
    }
}

static DEFAULT_TEMPLATE_NAME: &'static str = "DEFAULT_TEMPLATE";

pub fn create_context(git_info: &GitInfo, extra_vars: Json) -> Context {
    let mut context = git_info.to_context();
    add_env_vars_to_context(&mut context);
    add_extra_vars_to_context(&mut context, extra_vars);
    context
}

fn add_env_vars_to_context(context: &mut Context) {
    let mut env = Context::new();
    for (key, value) in std::env::vars() {
        env.insert(key, value.to_json());
    }
    context.insert("env".to_string(), Json::Object(env));
}

fn add_extra_vars_to_context(context: &mut Context, extra_vars: Json) {
    context.insert("extra".to_string(), extra_vars);
}

pub fn render_context(context: Context, opts: &Opts) -> Result<String, Error> {
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
                    let json = Json::Object(context);
                    handlebars.render(DEFAULT_TEMPLATE_NAME, &json)
                        .map_err(|e| Error::TemplateError(TemplateError::RenderError(e)))
                })
        })
}

pub fn render_context_to_file(context: Context, opts: &Opts) -> Result<String, Error> {
    render_context(context, opts).and_then(|rendered| {
        File::create(&opts.output)
            .map(|mut file| file.write_all(rendered.as_bytes()))
            .map_err(|e| Error::OutputError(e))
            .map(|_| rendered)
    })
}

fn parse_extra_vars(opt: &Option<String>) -> Result<Json, Error> {
    match *opt {
        None => Ok(Json::Object(Context::new())),
        Some(ref raw_json) => {
            Json::from_str(&raw_json)
                .map_err(|e| Error::JsonError(JsonError::Error(e)))
                .and_then(|obj: Json| {
                    if obj.is_object() {
                        Ok(obj)
                    } else {
                        Err(Error::JsonError(JsonError::NotOject))
                    }
                })
        }
    }
}

pub fn render_to_file(opts: &Opts) -> Result<String, Error> {
    let info = try!(exec::git_info(&opts.tag_pattern));
    let extra_vars = try!(parse_extra_vars(&opts.extra_vars));
    let context = create_context(&info, extra_vars);
    if opts.debug {
        print!("{}", context.to_json());
    }
    render_context_to_file(context, opts)
}