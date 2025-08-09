use std::env;
use std::path::PathBuf;

use croner::loader::ConfigCache;
use croner::printer::Printer;
use croner::scheduler::Scheduler;

fn main() {
    let mut config_path = PathBuf::from("config.croner");
    let mut print_enabled = true;

    for arg in env::args().skip(1) {
        if let Some(path) = arg.strip_prefix("--config=") {
            config_path = PathBuf::from(path);
        } else if let Some(flag) = arg.strip_prefix("--print=") {
            print_enabled = flag != "false";
        }
    }

    let mut cache = ConfigCache::new();
    if let Err(e) = cache.reload_if_changed(&config_path) {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    }

    let printer = Printer::new(print_enabled);
    let mut scheduler = Scheduler::new(cache, printer);

    scheduler.init();
    scheduler.run(&config_path);
}
