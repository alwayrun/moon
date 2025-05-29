mod action;

pub use action::*;
use clap::{Arg, ArgMatches, Command};

const AUTHOR: &'static str = "Viet-Hung Nguyen <viethungax@gmail.com>";

pub fn accept_cli() -> ArgMatches {
    let html_file_arg = Arg::new("html").long("html").required(false);
    let size_arg = Arg::new("size").long("size").required(true);
    let once_flag = Arg::new("once").long("once");
    let ouput_arg = Arg::new("output").long("output").required(true);

    let render_once_subcommand = Command::new("render")
        .about("Start a rendering process of Moon and render once")
        .author(AUTHOR)
        .arg(html_file_arg.clone().required(true))
        .arg(size_arg.clone())
        .arg(once_flag.clone())
        .arg(ouput_arg.clone());

    Command::new("Moon Renderer")
        .author(AUTHOR)
        .about("Moon web browser!")
        .subcommand(render_once_subcommand)
        .get_matches()
}
