use colored::*;
use std::{
    env::{self, args},
    fs::{self, File},
    path::Path,
    process::{exit, Command, Stdio},
    time::{Duration, SystemTime},
};
use std::os::unix::fs::MetadataExt;

pub mod config;
use crate::config::CFG;

fn main() {
    let issu = nix::unistd::Uid::effective().is_root();

    let home_dir = env::var("HOME").unwrap_or_else(|e| {
        print_error(
            "Unable to retrieve HOME environment variable",
            Some(&format!("{}", e)),
            Some("Ensure HOME is set correctly"),
        );
        exit(1);
    });

    let args = args().collect::<Vec<String>>();
    let mut cfg = CFG::parse();
    cfg.issu = issu;

    if cfg.auto_gc {
        perform_auto_gc(&cfg, &home_dir);
    }

    if args.len() < 2 {
        print_help();
        exit(0);
    }

    match args[1].as_str() {
        "home" => handle_home(&args, &cfg, &home_dir),
        "edit-cfg" => handle_edit_cfg(&cfg, &home_dir),
        "nix" => {
            if cfg.issu {
                handle_nix(&args, &cfg);
            } else {
                print_error(
                    "Nix commands require superuser privileges",
                    None,
                    Some("Use: sudo negma nix <subcommand>"),
                );
                exit(1);
            }
        }
        _ => {
            print_error(
                &format!("Unknown command '{}'", args[1]),
                None,
                Some("Run 'negma' without arguments to see available commands"),
            );
            print_help();
            exit(1);
        }
    }
}

/// Auto GC using marker file in config dir
fn perform_auto_gc(cfg: &CFG, home_dir: &str) {
    let marker_path = format!("{}/.config/negma/auto_gc_marker", home_dir);
    let marker = Path::new(&marker_path);
    let now = SystemTime::now();
    let interval = Duration::from_secs(cfg.gc_age_days.unwrap_or(7) as u64 * 86400);

    if marker.exists() {
        let metadata = fs::metadata(&marker).unwrap();
        let birth_time = SystemTime::UNIX_EPOCH + Duration::from_secs(metadata.ctime() as u64);
        if now.duration_since(birth_time).unwrap_or(Duration::from_secs(0)) >= interval {
            println!(
                "{} Auto GC: Collecting garbage, keeping last {} generations...",
                "[negma]".green().bold(),
                cfg.keep
            );
            let status = Command::new("nix-collect-garbage")
                .arg("-d")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "Auto GC failed");
            fs::remove_file(&marker).unwrap_or_else(|e| {
                print_error("Failed to remove old GC marker", Some(&e.to_string()), None);
                exit(1);
            });
            File::create(&marker).unwrap();
        }
    } else {
        File::create(&marker).unwrap();
    }
}

fn handle_edit_cfg(cfg: &CFG, home_dir: &str) {
    let path = format!("{}/.config/negma/config.cfg", home_dir);
    let status = Command::new(&cfg.editor)
        .arg(&path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    if let Ok(status) = status {
        if status.success() && cfg.auto_fmt {
            if let Some(fmt) = &cfg.formatter {
                let _ = Command::new(fmt)
                    .arg(&path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        } else if !status.success() {
            print_error("Editor exited with error", Some(&format!("Code: {}", status)), None);
            exit(1);
        }
    } else {
        print_error("Failed to launch editor", None, None);
        exit(1);
    }
}

fn handle_home(args: &[String], cfg: &CFG, home_dir: &str) {
    if args.len() < 3 {
        print_error(
            "Missing subcommand for 'home'",
            None,
            Some("Run 'negma' to see available home subcommands"),
        );
        return;
    }

    let home_config_dir = format!("{}/.config/home-manager", home_dir);

    match args[2].as_str() {
        "edit" => {
            println!("{} Editing {}...", "[negma]".green().bold(), home_config_dir.bright_black());
            let status = Command::new(&cfg.editor)
                .arg(&home_config_dir)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "Editing home-manager config failed");

            if cfg.auto_fmt {
                if let Some(fmt) = &cfg.formatter {
                    println!("{} Formatting {}...", "[negma]".green().bold(), home_config_dir.bright_black());
                    let status = Command::new(fmt)
                        .arg(&home_config_dir)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status();
                    exit_if_fail(status, "Formatting home-manager config failed");
                }
            }
        }
        "fmt" => {
            if let Some(fmt) = &cfg.formatter {
                println!("{} Formatting {}...", "[negma]".green().bold(), home_config_dir.bright_black());
                let status = Command::new(fmt)
                    .arg(&home_config_dir)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status();
                exit_if_fail(status, "Formatting home-manager config failed");
            } else {
                print_error("No formatter configured", None, Some("Set 'formatter' in negma config"));
            }
        }
        "make" => {
            println!("{} Applying home-manager switch...", "[negma]".green().bold());
            let status = Command::new("home-manager")
                .arg("switch")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "home-manager switch failed");
        }
        "gc" => {
            println!("{} Expiring old home-manager generations...", "[negma]".green().bold());
            let status = Command::new("home-manager")
                .arg("expire-generations")
                .arg("-d")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "home-manager expire-generations failed");
        }
        "clean" => {
            println!("{} Cleaning old Home Manager generations, keeping current...", "[negma]".green().bold());
            let status = Command::new("home-manager")
                .arg("expire-generations")
                .arg("0")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "home-manager clean failed");
        }
        "backup" => {
            let config_path = format!("{}/home.nix", home_config_dir);
            let backup_path = format!("{}/home.nix.bak", home_config_dir);

            fs::copy(&config_path, &backup_path).unwrap_or_else(|e| {
                print_error("Failed to backup home.nix", Some(&e.to_string()), None);
                exit(1);
            });
            println!(
                "{} Backup created: {}",
                "[negma]".green().bold(),
                backup_path.bright_black()
            );
        }
        "list-generations" => {
            println!("{} Listing home-manager generations...", "[negma]".green().bold());
            let status = Command::new("home-manager")
                .arg("generations")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "home-manager generations failed");
        }
        "rollback" => {
            let r#gen = if args.len() > 3 { &args[3] } else { "--rollback" };
            println!("{} Rolling back home-manager...", "[negma]".green().bold());
            let status = Command::new("home-manager")
                .args(&["switch", r#gen])
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "home-manager rollback failed");
        }
        _ => {
            print_error(
                &format!("Unknown home subcommand '{}'", args[2]),
                None,
                Some("Run 'negma' for available subcommands"),
            );
        }
    }
}

fn handle_nix(args: &[String], cfg: &CFG) {
    if args.len() < 3 {
        print_error(
            "Missing subcommand for 'nix'",
            None,
            Some("Run 'negma' to see available nix subcommands"),
        );
        return;
    }

    match args[2].as_str() {
        "edit" => {
            let config_path = "/etc/nixos/configuration.nix";
            println!("{} Editing {}...", "[negma]".green().bold(), config_path.bright_black());
            let status = Command::new(&cfg.editor)
                .arg(config_path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            exit_if_fail(status, "Failed to edit NixOS configuration");

            if cfg.auto_fmt {
                if let Some(fmt) = &cfg.formatter {
                    println!("{} Formatting {}...", "[negma]".green().bold(), config_path.bright_black());
                    let status = Command::new(fmt)
                        .arg(config_path)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status();
                    exit_if_fail(status, "Failed to format NixOS configuration");
                }
            }
        }
        "fmt" => {
            if let Some(fmt) = &cfg.formatter {
                let config_path = "/etc/nixos";
                println!("{} Formatting {}...", "[negma]".green().bold(), config_path.bright_black());
                let status = Command::new(fmt)
                    .arg(config_path)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status();
                exit_if_fail(status, "Failed to format NixOS configuration");
            } else {
                print_error("No formatter configured", None, Some("Set 'formatter' in negma config"));
            }
        }
        "gc" => run_nix_env(vec!["collect-garbage", "-d"]),
        "make" => run_nix_env(vec!["rebuild", "switch"]),
        "list-generations" => run_nix_env(vec!["--profile", "/nix/var/nix/profiles/system", "--list-generations"]),
        "rollback" => {
            if args.len() > 3 {
                run_nix_env(vec![
                    "--profile",
                    "/nix/var/nix/profiles/system",
                    "--switch-generation",
                    &args[3],
                ]);
            } else {
                run_nix_env(vec![
                    "--profile",
                    "/nix/var/nix/profiles/system",
                    "--rollback",
                ]);
            }
        }
        "clean" => run_nix_env(vec![
            "--profile",
            "/nix/var/nix/profiles/system",
            "--delete-generations",
            "old",
        ]),
        _ => {
            print_error(
                &format!("Unknown nix subcommand '{}'", args[2]),
                None,
                Some("Run 'negma' for available subcommands"),
            );
        }
    }
}

fn run_nix_env(args: Vec<&str>) {
    println!("{} Running nix-env {}...", "[negma]".green().bold(), args.join(" ").bright_black());
    let status = Command::new("nix-env")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    exit_if_fail(status, "nix-env command failed");
}

fn exit_if_fail(status: Result<std::process::ExitStatus, std::io::Error>, msg: &str) {
    match status {
        Ok(s) if !s.success() => {
            eprintln!("{} {}", "[error]".red().bold(), msg.bright_white());
            exit(1);
        }
        Err(e) => {
            eprintln!(
                "{} {} ({})",
                "[error]".red().bold(),
                msg.bright_white(),
                e.to_string().bright_black()
            );
            exit(1);
        }
        _ => {}
    }
}

fn print_error(title: &str, details: Option<&str>, hint: Option<&str>) {
    eprintln!("{} {}", "[negma error]".red().bold(), title.bright_white());
    if let Some(d) = details {
        eprintln!("{} {}", "↳".red(), d.bright_black());
    }
    if let Some(h) = hint {
        eprintln!("{} {}", "hint:".yellow().bold(), h.bright_white());
    }
}

fn print_help() {
    println!("\n{}\n{}", "[negma]".blue().bold(), "A clean, practical NixOS & Home Manager CLI helper.".bright_white());
    println!("\n{} {}", "Usage:".bright_white().underline(), "negma <command> [subcommand] [args]".bright_yellow());
    println!("\n{}", "Commands:".bright_white().underline());
    println!("  {} {}", "home".bright_cyan().bold(), "<subcommand>".bright_white());
    println!("  {} {}", "nix".bright_cyan().bold(), "<subcommand>".bright_white());
    println!("  {}", "edit-cfg".bright_cyan().bold());

    println!("\n{}:", "Home Manager Subcommands".bright_white().underline());
    println!("  edit, fmt, make, gc, clean, backup, list-generations, rollback [gen]");

    println!("\n{}:", "NixOS Subcommands (requires sudo)".bright_white().underline());
    println!("  edit, fmt, make, gc, clean, list-generations, rollback [gen]");

    println!("\n{}:", "Examples".bright_white().underline());
    println!("  negma home edit");
    println!("  negma home fmt");
    println!("  sudo negma nix edit");
    println!("  sudo negma nix fmt");
    println!("  negma edit-cfg");

    println!("\n{}", "✨ Keep your NixOS clean and workflow calm with negma ✨".bright_purple());
}
