# ✨ `negma` – Clean NixOS & Home Manager CLI ✨

> **A fast, minimal, colored CLI to manage NixOS + Home Manager confidently and cleanly in your daily workflow.**

---

## 🌱 What is `negma`?

`negma` is a **lightweight Rust CLI** that helps you:

✅ Rebuild and garbage collect your NixOS system effortlessly.  
✅ Manage Home Manager generations and rollbacks simply.  
✅ Auto-run garbage collection after a configurable number of days.  
✅ Edit and auto-format your `negma` configuration easily.  
✅ All with **clean, colored, minimal CLI output** designed for clarity.

---

## ✨ Features

### 🖥️ NixOS System Management
- `nix make` – rebuild and switch to the new system configuration
- `nix gc` – garbage collect old generations
- `nix list-generations` – list system generations with clarity
- `nix rollback [gen]` – rollback to a specific system generation

### 🏡 Home Manager Management
- `home make` – apply Home Manager configuration
- `home edit` – edit your `home.nix` easily
- `home gc` – garbage collect Home Manager generations
- `home backup` – backup your `home.nix` safely
- `home list-generations` – list Home Manager generations
- `home rollback [gen]` – rollback to a specific Home Manager generation

### ⚙️ Configuration Management
- `edit-cfg` – edit your `negma` configuration with auto-formatting if enabled

### ♻️ Auto GC
- Automatically runs `nix-collect-garbage` after N days.
- Uses a marker file in `~/.config/negma/` to track last run cleanly.

---

## 🚀 Installation

### 🔧 Build from source (recommended)

```bash
git clone https://github.com/skubed0007/negma
cd negma
cargo build --release
# Optionally move to your PATH:
sudo cp target/release/negma /usr/local/bin
```

### 📦 From Release

You may find prebuilt binaries for NixOS in the [Releases](https://github.com/skubed0007/negma/releases) section.

---

## ⚙️ Configuration

Your config file is located at:

```
~/.config/negma/config.cfg
```

Edit it easily with:

```bash
negma edit-cfg
```

> # Important: please read default config created by negma to get to know all the options

---

## 🛠 Usage

Run without arguments to see available commands:

```bash
negma
```

Example usage:

```bash
negma home make
negma home edit
sudo negma nix gc
negma edit-cfg
```

---

## 🖤 Clean, Colored, Readable Output

- Uses `colored` for clear status outputs.
- Aligned and minimal, no overwhelming logs.
- Shows **clear success/error** with hints.

---

## 📄 License

Apache 2.0 License

---

## 🤝 Contributing

Contributions are welcome:
- CLI UX improvements
- Flake integration
- Additional workflow hooks
- Bug fixes

---

> **Happy hacking & keep your NixOS clean! 🩶✨**

---

## 📫 Contact

- **Author:** [skubed0007](https://github.com/skubed0007)
- Feel free to open issues for feature requests, questions, or enhancements!