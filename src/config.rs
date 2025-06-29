use colored::*;
use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::exit,
};

/// Configuration for Negma: A NixOS management tool for advanced users.
///
/// Reads from `~/.config/negma/config.cfg` and provides structured access to:
/// - Flake management
/// - Channel management
/// - Automatic garbage collection
/// - Formatter integration (nixfmt, alejandra, etc.)
/// - Custom alias handling
/// - Git-based configuration management
/// - Custom editor for config editing
///
/// Negma will automatically generate a *self-documented config* if missing.
#[derive(Debug)]
pub struct CFG {
    pub editor: String,
    pub git: String,
    pub issu : bool,
    pub keep: i32,
    pub alias: Vec<(String, String)>,
    pub system_flake: Option<String>,
    pub rebuild_flags: Option<String>,
    pub channel: Option<String>,
    pub auto_gc: bool,
    pub gc_age_days: Option<u32>,
    pub formatter: Option<String>,
    pub auto_fmt: bool,
}

impl CFG {
    /// Loads or creates the Negma configuration file with defaults.
    pub fn parse() -> CFG {
        let home_dir = env::var("HOME").unwrap_or_else(|e| {
            eprintln!(
                "{} {} {}",
                "[negma:config]".red().bold(),
                "error: unable to retrieve home directory.".red(),
                format!(
                    "\n  → hint: ensure the HOME environment variable is set.\n  → context: std::env::var(\"HOME\")\n  → underlying error: {}",
                    e
                )
                .bright_black()
            );
            exit(1);
        });

        let config_path = PathBuf::from(format!("{}/.config/negma/config.cfg", home_dir));

        if !config_path.exists() {
            println!(
                "{} {} {}",
                "[negma:config]".yellow().bold(),
                "configuration file not found.".yellow(),
                "Creating default configuration...".bright_black()
            );

            if let Some(parent) = config_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!(
                        "{} {} {}",
                        "[negma:config]".red().bold(),
                        "error: failed to create configuration directory.".red(),
                        format!(
                            "\n  → context: {}\n  → underlying error: {}",
                            parent.display(),
                            e
                        )
                        .bright_black()
                    );
                    exit(1);
                }
            }

            let mut file = File::create(&config_path).unwrap_or_else(|e| {
                eprintln!(
                    "{} {} {}",
                    "[negma:config]".red().bold(),
                    "error: failed to create configuration file.".red(),
                    format!(
                        "\n  → context: {}\n  → underlying error: {}",
                        config_path.display(),
                        e
                    )
                    .bright_black()
                );
                exit(1);
            });

            let default_content = r#"# Negma Configuration File

###############################################################
#                                                             #
#                 Negma - NixOS Manager Config                #
#                                                             #
# This file controls how Negma manages your NixOS system.     #
# It is automatically created and updated by Negma.           #
#                                                             #
###############################################################

# === Basic Configuration ===

# EDITOR specifies your preferred editor for editing config files.
# Example: EDITOR = helix
EDITOR = nano

# GIT specifies your system configuration git repo (optional).
# Example: GIT = https://github.com/username/nixos-config
GIT = 

# KEEP specifies how many system generations to keep when cleanup is called.
# 0 = keep current, 1 = keep current + last one, etc.
# Example: KEEP = 5
KEEP = 5

# === Advanced Configuration ===

# SYSTEM_FLAKE specifies a flake URI or local path for nixos-rebuild.
# Example: SYSTEM_FLAKE = github:username/nixos-config
SYSTEM_FLAKE = 

# REBUILD_FLAGS specifies additional flags for nixos-rebuild.
# Example: REBUILD_FLAGS = --impure --show-trace
REBUILD_FLAGS =

# CHANNEL specifies your preferred Nix channel.
# Example: CHANNEL = nixos-unstable
CHANNEL = 

# AUTO_GC specifies if automatic garbage collection should run during rebuild.
# Valid values: true / false
AUTO_GC = true

# GC_AGE_DAYS specifies the maximum age (in days) before GC removal.
# Example: GC_AGE_DAYS = 15
GC_AGE_DAYS = 15

# FORMATTER specifies which Nix formatter to use for autofmt operations.
# Supported: nixfmt-rfc-style, alejandra, nixpkgs-fmt, etc.
# Example: FORMATTER = alejandra
FORMATTER = alejandra

# AUTO_FMT specifies whether Negma should auto-format system config before rebuild.
# Valid values: true / false
AUTO_FMT = true

# === Aliases ===
# Aliases allow you to create shortcuts for common commands.
# Example:
# alias mk = build
# alias bkup = backup

"#;

            if let Err(e) = file.write_all(default_content.as_bytes()) {
                eprintln!(
                    "{} {} {}",
                    "[negma:config]".red().bold(),
                    "error: failed to write default configuration.".red(),
                    format!(
                        "\n  → context: {}\n  → underlying error: {}",
                        config_path.display(),
                        e
                    )
                    .bright_black()
                );
                exit(1);
            }

            println!(
                "{} {} {}",
                "[negma:config]".green().bold(),
                "created default configuration at".green(),
                config_path.display().to_string().bright_black()
            );
        }

        let file = File::open(&config_path).unwrap_or_else(|e| {
            eprintln!(
                "{} {} {}",
                "[negma:config]".red().bold(),
                "error: unable to open configuration file.".red(),
                format!(
                    "\n  → context: {}\n  → underlying error: {}",
                    config_path.display(),
                    e
                )
                .bright_black()
            );
            exit(1);
        });

        let reader = BufReader::new(file);

        let mut editor = String::from("nano");
        let mut git = String::new();
        let mut clrupam = 5;
        let mut alias = Vec::new();
        let mut system_flake = None;
        let mut rebuild_flags = None;
        let mut channel = None;
        let mut auto_gc = false;
        let mut gc_age_days = None;
        let mut formatter = None;
        let mut auto_fmt = false;

        for (index, line) in reader.lines().enumerate() {
            let line_number = index + 1;
            let line = match line {
                Ok(l) => l.trim().to_string(),
                Err(e) => {
                    eprintln!(
                        "{} {} {}",
                        "[negma:config]".yellow().bold(),
                        format!("warning: failed to read line {}.", line_number).yellow(),
                        format!(
                            "\n  → context: {}\n  → underlying error: {}",
                            config_path.display(),
                            e
                        )
                        .bright_black()
                    );
                    continue;
                }
            };

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parse_kv = |line: &str, prefix: &str| -> Option<String> {
                let rest = line.strip_prefix(prefix)?;
                let parts: Vec<&str> = rest.trim().splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some(parts[1].trim().to_string())
                } else {
                    eprintln!(
                        "{} {} {}",
                        "[negma:config]".yellow().bold(),
                        format!(
                            "warning: invalid {} syntax at line {}.",
                            prefix.trim(),
                            line_number
                        )
                        .yellow(),
                        format!(
                            "\n  → hint: use '{} = value'\n  → line content: '{}'",
                            prefix.trim(),
                            line
                        )
                        .bright_black()
                    );
                    None
                }
            };

            if line.starts_with("alias") {
                let parts: Vec<&str> = line["alias".len()..].trim().splitn(2, '=').collect();
                if parts.len() == 2 {
                    alias.push((parts[0].trim().to_string(), parts[1].trim().to_string()));
                } else {
                    eprintln!(
                        "{} {} {}",
                        "[negma:config]".yellow().bold(),
                        format!("warning: invalid alias syntax at line {}.", line_number).yellow(),
                        format!(
                            "\n  → hint: use 'alias name = command'\n  → line content: '{}'",
                            line
                        )
                        .bright_black()
                    );
                }
            } else if let Some(val) = parse_kv(&line, "EDITOR") {
                editor = val;
            } else if let Some(val) = parse_kv(&line, "GIT") {
                git = val;
            } else if let Some(val) = parse_kv(&line, "KEEP") {
                match val.parse::<i32>() {
                    Ok(n) => clrupam = n,
                    Err(_) => eprintln!(
                        "{} {} {}",
                        "[negma:config]".yellow().bold(),
                        format!("warning: invalid KEEP value at line {}.", line_number).yellow(),
                        format!("\n  → hint: use an integer.\n  → line content: '{}'", line)
                            .bright_black()
                    ),
                }
            } else if let Some(val) = parse_kv(&line, "SYSTEM_FLAKE") {
                if !val.is_empty() {
                    system_flake = Some(val);
                }
            } else if let Some(val) = parse_kv(&line, "REBUILD_FLAGS") {
                if !val.is_empty() {
                    rebuild_flags = Some(val);
                }
            } else if let Some(val) = parse_kv(&line, "CHANNEL") {
                if !val.is_empty() {
                    channel = Some(val);
                }
            } else if let Some(val) = parse_kv(&line, "AUTO_GC") {
                auto_gc = matches!(val.to_lowercase().as_str(), "true" | "yes" | "1");
            } else if let Some(val) = parse_kv(&line, "GC_AGE_DAYS") {
                match val.parse::<u32>() {
                    Ok(n) => gc_age_days = Some(n),
                    Err(_) => eprintln!(
                        "{} {} {}",
                        "[negma:config]".yellow().bold(),
                        format!(
                            "warning: invalid GC_AGE_DAYS value at line {}.",
                            line_number
                        )
                        .yellow(),
                        format!("\n  → hint: use an integer.\n  → line content: '{}'", line)
                            .bright_black()
                    ),
                }
            } else if let Some(val) = parse_kv(&line, "FORMATTER") {
                if !val.is_empty() {
                    formatter = Some(val);
                }
            } else if let Some(val) = parse_kv(&line, "AUTO_FMT") {
                auto_fmt = matches!(val.to_lowercase().as_str(), "true" | "yes" | "1");
            } else {
                eprintln!(
                    "{} {} {}",
                    "[negma:config]".yellow().bold(),
                    format!("warning: unrecognized line at {}.", line_number).yellow(),
                    format!("\n  → line content: '{}'", line).bright_black()
                );
            }
        }

        CFG {
            editor,
            git,
            keep: clrupam,
            alias,
            system_flake,
            rebuild_flags,
            channel,
            auto_gc,
            gc_age_days,
            formatter,
            auto_fmt,
            issu: false,
        }
    }
}
