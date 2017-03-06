use std::process::Command;
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};

#[derive(Debug)]
pub struct GitRev {
    revision: String,
    branch: String,
    tags: Vec<String>,
}

pub fn git_rev() -> GitRev {
    GitRev {
        revision: git_rev_parse(),
        branch: git_branch(),
        tags: git_tags(),
    }
}

impl ToJson for GitRev {
    fn to_json(&self) -> Json {
        let mut obj: BTreeMap<String, Json> = BTreeMap::new();
        obj.insert("revision".to_string(), self.revision.to_json());
        obj.insert("branch".to_string(), self.branch.to_json());
        obj.insert("tags".to_string(), self.tags.to_json());
        let result = Json::Object(obj);
        result
    }
}

pub fn git_rev_parse() -> String {
    let output = Command::new("git").arg("rev-parse").arg("HEAD")
        .output()
        .ok()
        .expect(r#"Failed to run "git rev-parse HEAD""#);
    String::from_utf8(output.stdout)
        .ok()
        .expect("Failed to parse output of git command")
        .trim().to_string()
}

pub fn git_tags() -> Vec<String> {
    let output = Command::new("git").arg("tag").arg("-l").arg("--points-at").arg("HEAD")
        .output()
        .ok()
        .expect(r#"Failed to run "git -l --points-at HEAD""#);
    String::from_utf8(output.stdout)
        .ok()
        .expect("Failed to parse output of git command")
        .trim()
        .split("\n")
        .map(|line| line.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
}

pub fn git_branch() -> String {
    let output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD")
        .output()
        .ok()
        .expect(r#"Failed to run "git rev-parse --abbrev-ref HEAD""#);
    String::from_utf8(output.stdout)
        .ok()
        .expect("Failed to parse output of git command")
        .trim().to_string()
}