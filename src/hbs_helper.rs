use std::process::Command;

use super::handlebars::{Handlebars, Helper, RenderContext, RenderError};
use super::exec::command_output;

pub fn git_log_fmt_helper(helper: &Helper,
                          _: &Handlebars,
                          context: &mut RenderContext)
                          -> Result<(), RenderError> {
    if let Some(param) = helper.param(0) {
        if let Some(param) = param.value().as_string() {
            let mut command = Command::new("git");
            command.env("LESS", "-iXFR");
            command.arg("log").arg("-1").arg(format!("--format={}", param));
            let output = command_output(&mut command,
                                        format!("git log -1 --format={}", param).to_string());
            if let Ok(output) = output {
                try!(context.writer.write(output.into_bytes().as_ref()));
            }
        }
    }
    Ok(())
}