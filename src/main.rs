#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod i18n;
mod styles;
mod render;
mod install;
#[allow(dead_code)]
mod log;
#[allow(dead_code)]
mod cache;
mod wizard;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--render") {
        render::run();
    } else {
        wizard::run();
    }
}
