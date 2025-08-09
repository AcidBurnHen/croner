use std::{env, fs, io, path::PathBuf};

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
                print_help();
                return;
            }
            "--uninstall" => {
                uninstall();
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

fn print_help() {
    println!(
        "\x1b[1;36m{}\x1b[0m - A high-performance cron-style job runner

\x1b[1mUSAGE:\x1b[0m
    \x1b[32mcroner\x1b[0m [OPTIONS]

\x1b[1mOPTIONS:\x1b[0m
    \x1b[33m--config=<path>\x1b[0m    Path to config file (default: ./config.croner)
    \x1b[33m--print=<bool>\x1b[0m     Enable/disable printing job output (default: true)
    \x1b[33m--version, -v\x1b[0m      Show version and exit
    \x1b[33m--help, -h\x1b[0m         Show this help message and exit
    \x1b[33m--uninstall\x1b[0m        Remove Croner from system

\x1b[1mEXAMPLES:\x1b[0m
    croner
    croner --config=/etc/croner/jobs.croner
    croner --print=false
    croner --version
    croner --uninstall
",
        r#"
   ______                          
  / ____/________  ____  ___  _____
 / /   / ___/ __ \/ __ \/ _ \/ ___/
/ /___/ /  / /_/ / / / /  __/ /    
\____/_/   \____/_/ /_/\___/_/     
                                   
                                   
"#
    );
}

#[inline]
fn home_dir() -> Option<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        return Some(PathBuf::from(home));
    }
    if let Ok(profile) = env::var("USERPROFILE") {
        return Some(PathBuf::from(profile));
    }
    None
}

fn confirm_uninstall() -> bool {
    println!("\x1b[31;1m⚠ WARNING:\x1b[0m This will remove Croner from your system.");
    print!("Are you sure? (y/N): ");
    io::Write::flush(&mut io::stdout()).unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let ans = input.trim().to_lowercase();
        return ans == "y" || ans == "yes";
    }
    false
}

fn uninstall() {
    if !confirm_uninstall() {
        println!("Uninstall cancelled.");
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let possible_paths = vec![
            PathBuf::from(r"C:\Program Files\croner\croner.exe"),
            home_dir()
                .map(|h| h.join(r"AppData\Local\Programs\croner\croner.exe"))
                .unwrap_or_default(),
        ];

        let mut removed_any = false;
        for path in possible_paths {
            if path.exists() {
                match fs::remove_file(&path) {
                    Ok(_) => {
                        println!("Removed {}", path.display());
                        removed_any = true;
                    }
                    Err(e) => eprintln!("Failed to remove {}: {}", path.display(), e),
                }
            }
        }

        if !removed_any {
            println!("Croner binary failed to be removed or not found");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let possible_paths = vec![
            PathBuf::from("/usr/local/bin/croner"),
            PathBuf::from("/usr/bin/croner"),
            home_dir()
                .map(|h| h.join(".local/bin/croner"))
                .unwrap_or_default(),
        ];

        let mut removed_any = false;
        for path in possible_paths {
            if path.exists() {
                match fs::remove_file(&path) {
                    Ok(_) => {
                        println!("Removed {}", path.display());
                        removed_any = true;
                    }
                    Err(e) => eprintln!("Failed to remove {}: {}", path.display(), e),
                }
            }
        }

        if !removed_any {
            println!("Croner binary failed to be removed or not found");
        }
    }
}
