extern crate handlebars;
extern crate git_rev;

use std::env;
use std::process;
use std::io::Read;
use std::io::Write;
use std::fs::File;

use handlebars::Handlebars;

use git_rev::exec::git_rev;

static DEFAULT_TEMPLATE_NAME: &'static str = "DEFAULT_TEMPLATE";

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() != 3 {
        println_stderr!("USAGE: git-rev {{input-template}} {{output}}");
        process::exit(1);
    }

    let ref input_file = args[1];
    let ref output_file = args[2];

    let git_rev = git_rev();

    let mut handlebars = Handlebars::new();
    setup_handlerbars(&mut handlebars, args[1].as_str());

    match handlebars.render(DEFAULT_TEMPLATE_NAME, &git_rev) {
        Err(e) => {
            println_stderr!("Failed to render template {}.\n{}", input_file, e.desc);
            process::exit(2);
        },
        Ok(rendered) => {
            match File::create(args[2].as_str()) {
                Err(e) => {
                    println_stderr!("Failed to create {}.\n{}", output_file, e);
                    process::exit(3);
                },
                Ok(mut file) => {
                    match file.write_all(rendered.as_bytes()) {
                        Err(e) => {
                            println_stderr!("Failed to write to {}.\n{}", output_file, e);
                            process::exit(3);
                        },
                        Ok(_) => (),
                    }
                }
            }
        }
    }

    process::exit(0);
}

fn setup_handlerbars(handlebars: &mut Handlebars, input: &str) {
    match File::open(input) {
        Err(e) => {
            println_stderr!("Failed to open {}.\n{}", input, e);
            process::exit(2);
        },
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            match handlebars.register_template_string(DEFAULT_TEMPLATE_NAME, &content) {
                Err(e) => {
                    println_stderr!("Failed to register Template {}.\n{}", input, e.reason);
                    process::exit(2);
                },
                Ok(_) => ()
            }
        }
    }
}
