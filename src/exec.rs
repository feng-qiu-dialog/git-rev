use std::process::Command;

use super::{Error, GitInfo};

pub fn git_info(tag_filter: &Option<String>, short: &Option<usize>) -> Result<GitInfo, Error> {
    let rev = try!(git_rev_parse());
    let rev_short = try!(git_rev_parse_short(short));
    let branch = try!(git_branch());
    let tags = try!(git_tags(tag_filter));

    Ok(GitInfo {
        revision: rev,
        rev_short: rev_short,
        branch: branch,
        tags: tags,
    })
}

fn command_output(command: &mut Command, raw_command: String) -> Result<String, Error> {
    command.output()
        .map_err(|e| Error::CommandError(raw_command, e))
        .and_then(|output| {
            String::from_utf8(output.stdout).map_err(|_| Error::CommandOutputParsingError)
        })
        .map(|output| output.trim().to_string())
}

pub fn git_rev_parse() -> Result<String, Error> {
    let mut command = Command::new("git");
    command.arg("rev-parse").arg("HEAD");
    command_output(&mut command, "git rev-parse HEAD".to_string())
}

pub fn git_rev_parse_short(short: &Option<usize>) -> Result<String, Error> {
    let mut command = Command::new("git");
    command.arg("rev-parse");
    match *short {
        None => { command.arg("--short"); },
        Some(ref short_len) => { command.arg(format!("--short={}", short_len)); },
    }
    command.arg("HEAD");
    command_output(&mut command, "git rev-parse --short HEAD".to_string())
}

pub fn git_tags(filter: &Option<String>) -> Result<Vec<String>, Error> {
    let mut command = Command::new("git");
    command.arg("tag").arg("-l").arg("--points-at").arg("HEAD");
    let mut raw_command = "git tag -l --points-at HEAD".to_string();

    match *filter {
        None => (),
        Some(ref tag_pattern) => {
            command.arg(tag_pattern.clone());
            raw_command.push(' ');
            raw_command.push_str(&tag_pattern);
        }
    };

    command_output(&mut command, raw_command).map(|output| {
        output
            .split("\n")
            .map(|line| line.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
    })
}

pub fn git_branch() -> Result<String, Error> {
    let mut command = Command::new("git");
    command.arg("rev-parse").arg("--abbrev-ref").arg("HEAD");
    command_output(&mut command, "git rev-parse --abbrev-ref HEAD".to_string())
}