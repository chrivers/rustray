use std::path::PathBuf;

#[cfg(not(feature = "gui"))]
use std::str::FromStr;

use log::LevelFilter;

use rustray::types::RResult;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = 1440)]
    width: u32,

    #[arg(short, long, default_value_t = 1200)]
    height: u32,

    #[arg(value_name = "input")]
    input: PathBuf,

    #[arg(short, long, value_name = "output")]
    output: Option<PathBuf>,
}

fn main() -> RResult<()> {
    let mut logger = colog::default_builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    let cli = Cli::parse();

    type F = f64;

    #[cfg(feature = "gui")]
    return rustray::frontend::gui::run::<F>(cli.input, cli.width, cli.height);

    #[cfg(not(feature = "gui"))]
    return rustray::frontend::cli::run::<F>(
        &cli.input,
        cli.width,
        cli.height,
        cli.output
            .unwrap_or_else(|| PathBuf::from_str("output.png").unwrap()),
    );
}

#[cfg(test)]
mod test {
    use rustray::demoscene;
    use rustray::scene::BoxScene;

    #[test]
    fn test_demoscene() {
        colog::init();
        let mut scene = BoxScene::empty();
        demoscene::construct_demo_scene::<f32>(&mut scene).unwrap();
    }
}
