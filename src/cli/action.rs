use clap::ArgMatches;

pub enum Action {
    RenderOnce(RenderOnceParams),
    StartMain,
}

pub struct RenderOnceParams {
    pub html_path: String,
    pub viewport_size: (u32, u32),
    pub output_path: String,
}

pub fn get_action<'a>(matches: ArgMatches) -> Action {
    if let Some(matches) = matches.subcommand_matches("render") {
        let html = matches.get_one::<String>("html").expect("Required");
        let raw_size = matches.get_one::<String>("size").expect("Required");
        let output_path = matches.get_one::<String>("output").expect("Required");

        let is_render_once = matches.get_flag("once");

        let viewport_size = parse_size(&raw_size);

        if is_render_once {
            return Action::RenderOnce(RenderOnceParams {
                html_path: html.clone(),
                output_path: output_path.clone(),
                viewport_size,
            });
        }
    }

    Action::StartMain
}

fn parse_size(raw_size: &str) -> (u32, u32) {
    let size_params = raw_size
        .split('x')
        .filter_map(|size| size.parse::<u32>().ok())
        .take(2)
        .collect::<Vec<u32>>();

    match &size_params[..] {
        &[width, height, ..] => (width, height),
        _ => unreachable!(),
    }
}
