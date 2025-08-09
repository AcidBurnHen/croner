use std::env;
use std::path::PathBuf;

use croner::loader::ConfigCache;
use croner::printer::Printer;
use croner::scheduler::Scheduler;

fn main() {
    let mut config_path = PathBuf::from("config.croner");
    let mut print_enabled = true;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-v" => {
                println!("croner {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => {
                println!(
                    "\x1b[1;36m{}\x1b[0m - A high-performance cron-style job runner

\x1b[1mUSAGE:\x1b[0m
    \x1b[32mcroner\x1b[0m [OPTIONS]

\x1b[1mOPTIONS:\x1b[0m
    \x1b[33m--config=<path>\x1b[0m    Path to config file (default: ./config.croner)
    \x1b[33m--print=<bool>\x1b[0m     Enable/disable printing job output (default: true)
    \x1b[33m--version, -v\x1b[0m      Show version and exit
    \x1b[33m--help, -h\x1b[0m         Show this help message and exit

\x1b[1mEXAMPLES:\x1b[0m
    croner
    croner --config=/etc/croner/jobs.croner
    croner --print=false
    croner --version
",
                    r#"
   ______                          
  / ____/________  ____  ___  _____
 / /   / ___/ __ \/ __ \/ _ \/ ___/
/ /___/ /  / /_/ / / / /  __/ /    
\____/_/   \____/_/ /_/\___/_/     
                                   
                                   
"#,
                );
                return;
            }
            _ => {
                if let Some(path) = arg.strip_prefix("--config=") {
                    config_path = PathBuf::from(path);
                } else if let Some(flag) = arg.strip_prefix("--print=") {
                    print_enabled = flag != "false";
                }
            }
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
