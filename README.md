# Vibe Clip

```
 __     ___ _             ____ _ _       
\ \   / (_) |__   ___   / ___| (_)_ __  
 \ \ / /| | '_ \ / _ \ | |   | | | '_ \ 
  \ V / | | |_) |  __/ | |___| | | |_) |
   \_/  |_|_.__/ \___|  \____|_|_| .__/ 
                                 |_|    
```

A small Rust CLI application that automatically sanitizes sensitive information from your system clipboard, ensuring great vibes when working with LLMs and sharing code.

## Installation

```bash
cargo install --git https://github.com/jefrwld/vibe-clip.git
```

## Usage

```bash
vibe-clip
```

## Configuration

To define custom strings that should be sanitized, edit the `config.yaml` file:

```yaml
domains:
  - "my-company.com"
  - "secret-project.com"

words:
  - "CompanyName"
  - "SecretVariable"
```

## Requirements

- Rust and Cargo installed
- On macOS: Uses `pbcopy`/`pbpaste`
- On Windows: Uses PowerShell clipboard commands
- On Linux: Uses `clip.exe` for WSL environments

## License

MIT