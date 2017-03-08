extern crate argparse;

#[macro_use]
extern crate git_rev;

use std::env;
use std::process;
use std::io::Write;

use argparse::ArgumentParser;

use git_rev::{Opts, ExitStatus};

fn exit(exit_status: ExitStatus) {
    match exit_status {
        ExitStatus::Success => (),
        _ => println_stderr!("{}", exit_status)
    };

    process::exit(exit_status.exit_code());
}

fn parse_args(args: &mut Opts) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Render template with git info");

    parser.refer(&mut args.template)
        .add_argument("template", argparse::Store, "Path to the template file to render")
        .required();
    parser.refer(&mut args.output)
        .add_argument("output", argparse::Store, "Path to the output file")
        .required();

    parser.refer(&mut args.tag_pattern)
        .add_option(
            &["-t", "--tag-pattern"], 
            argparse::StoreOption, 
            "Extra argument passed to 'git -l --points-at HEAD' to filter tags");

    parser.refer(&mut args.debug)
        .add_option(
            &["-d", "--debug"], 
            argparse::StoreTrue, 
            "Turn on debug mode. Prints the context object fed into the template.");

    parser.refer(&mut args.extra_vars)
        .add_option(
            &["-e", "--vars"], 
            argparse::StoreOption, 
            "JSON string which contains extra variables to be rendered");

    parser.parse_args_or_exit();
}

fn main() {
    let args = env::args_os().collect::<Vec<_>>();
    println!("{:?}", args);
    let mut opts = Opts::new();
    parse_args(&mut opts);
    match git_rev::render_to_file(&opts) {
        Ok(_) => (),
        Err(e) => exit(ExitStatus::Error(e))
    }
}
