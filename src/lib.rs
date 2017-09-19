extern crate handlebars;
extern crate rustc_serialize;

pub mod exec;
mod hbs_helper;

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

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct Opts {
    pub template: String,
    pub output: Option<String>,
    pub tag_pattern: Option<String>,
    pub extra_vars: Option<String>,
    pub debug: bool,
    pub show_version: bool,
    pub short: Option<usize>,
}

impl Opts {
    pub fn new() -> Opts {
        Opts {
            template: String::new(),
            output: None,
            tag_pattern: None,
            extra_vars: None,
            debug: false,
            show_version: false,
            short: None,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    CommandError(String, std::io::Error),
    CommandFailure(String, String),
    CommandOutputParsingError,
    TemplateError(TemplateError),
    OutputError(std::io::Error),
    JsonError(JsonError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CommandFailure(ref command, ref details) => {
                write!(f, "Failed to run command '{}'.\n{}", command, details)
            },
            Error::CommandError(ref command, ref e) => {
                write!(f, "Failed to run command '{}'. {}", command, e)
            },
            Error::CommandOutputParsingError => {
                write!(f, "Failed to parse command output")
            },
            Error::TemplateError(ref e) => {
                write!(f, "Failed to render template. {}", e)
            },
            Error::OutputError(ref e) => {
                write!(f, "Failed to write the output. {}", e)
            },
            Error::JsonError(ref e) => {
                write!(f, "Failed to parse extra vars. {}", e)
            },
        }
    }
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
    NotObject,
    Error(json::ParserError),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            JsonError::NotObject => write!(f, "JSON Object expected"),
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
            ExitStatus::Error(ref e) => {
                write!(f, "{}", e)
            },
        }
    }
}

pub type Context = BTreeMap<String, Json>;

#[derive(Debug)]
pub struct GitInfo {
    revision: String,
    rev_short: String,
    branch: String,
    tags: Vec<String>,
}

impl GitInfo {
    fn to_context(&self) -> Context {
        let mut obj = Context::new();
        obj.insert("revision".to_string(), self.revision.to_json());
        obj.insert("rev_short".to_string(), self.rev_short.to_json());
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

fn setup_handlebars(handlebars: &mut Handlebars) {
    handlebars.register_helper(hbs_helper::HBS_HELPER_GIT_LOG_FMT, Box::new(hbs_helper::git_log_fmt_helper));
}

pub fn render_context(context: Context, opts: &Opts) -> Result<String, Error> {
    let mut handlebars = Handlebars::new();
    setup_handlebars(&mut handlebars);
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

fn write_string_to_file(content: String, path: &str) -> Result<String, Error> {
    File::create(path)
        .map(|mut file| file.write_all(content.as_bytes()))
        .map_err(|e| Error::OutputError(e))
        .map(|_| content)
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
                        Err(Error::JsonError(JsonError::NotObject))
                    }
                })
        }
    }
}

pub fn render_to_file(opts: &Opts) -> Result<String, Error> {
    let info = exec::git_info(&opts.tag_pattern, &opts.short)?;
    let extra_vars = parse_extra_vars(&opts.extra_vars)?;
    let context = create_context(&info, extra_vars);
    if opts.debug {
        print!("{}", json::as_pretty_json(&context));
    }

    render_context(context, opts)
        .and_then(|rendered| {
            match opts.output {
                Some(ref output) => write_string_to_file(rendered, &output),
                None => {
                    if opts.debug {
                        println!("\n\n{:#<1$}\n{2}", "#", 80, rendered);
                    } else {
                        println!("{}", rendered);
                    }
                    Ok(rendered)
                },
            }
        })
}
