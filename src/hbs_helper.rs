use std::process::Command;

use super::handlebars::{Handlebars, Helper, RenderContext, RenderError};
use super::exec::command_output;

pub const HBS_HELPER_GIT_LOG_FMT: &'static str = "git_log_format";

pub fn git_log_fmt_helper(helper: &Helper,
                          _: &Handlebars,
                          context: &mut RenderContext)
                          -> Result<(), RenderError> {
    match helper.param(0) {
        None => Err(RenderError::new(format!("{}: one argument expected", HBS_HELPER_GIT_LOG_FMT))),
        Some(param) => {
            match param.value().as_string() {
                None => Err(RenderError::new(format!("{}: only string argument is accepted", HBS_HELPER_GIT_LOG_FMT))),
                Some(param) => {
                    let mut command = Command::new("git");
                    command.env("LESS", "-iXFR");
                    command.arg("log").arg("-1").arg(format!("--format={}", param));
                    let output = command_output(&mut command,
                                                format!("git log -1 --format={}", param).to_string());
                    match output {
                        Err(e) => Err(RenderError::new(format!("{}", e))),
                        Ok(output) => {
                            context.writer.write(output.into_bytes().as_ref())?;
                            Ok(())
                        },
                    }
                }
            }
        }
    }
}