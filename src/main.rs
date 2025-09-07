use std::process::Command;
use std::io::{self, Write};
use std::time::Duration;
use std::thread;
use serde::Deserialize;
use regex::Regex;
use rand::Rng;
use std::collections::HashMap;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,

  #[arg(short, long, default_value = "config.yaml")]
  config: String,
}

#[derive(Subcommand)]
enum Commands {
  Start,
}

#[derive(Deserialize)]
struct Config {
    domains: Vec<String>,
    words: Vec<String>,
}

fn main(){
   let cli = Cli::parse();
   match cli.command {
        Some(Commands::Start) => {
           start_monitoring(); 
        }
        None => {
           start_monitoring(); 
        }
   }
   
}

fn start_monitoring() {
    println!(r#"
__     ___ _             ____ _ _       
\ \   / (_) |__   ___   / ___| (_)_ __  
 \ \ / /| | '_ \ / _ \ | |   | | | '_ \ 
  \ V / | | |_) |  __/ | |___| | | |_) |
   \_/  |_|_.__/ \___|  \____|_|_| .__/ 
                                 |_|                                 
"#);
    
    let config = load_config();
    let mut last_content = String::new();
    let word = generate_word();

    let mut wordmap: HashMap<String, String> = HashMap::new();

    println!("word: {}", word);
    // OS Detection Debug
    if cfg!(target_os = "windows") {
        println!("DEBUG: Detected OS: Windows");
    } else if cfg!(target_os = "macos") {
        println!("DEBUG: Detected OS: macOS");
    } else if cfg!(target_os = "linux") {
        println!("DEBUG: Detected OS: Linux");
    } else {
        println!("DEBUG: Detected OS: Unknown");
    }
    
    println!("Clipboard monitoring started. Press Ctrl+C to stop.");
    
    loop {
        let current_content = if cfg!(target_os = "windows") {
            read_windows_clipboard().unwrap_or_else(|e| {
                eprintln!("Error reading clipboard: {}", e);
                String::new()
            })
        } else if cfg!(target_os = "macos") {
            read_macos_clipboard().unwrap_or_else(|e| {
                eprintln!("Error reading clipboard: {}", e);
                String::new()
            })
        } else {
            // Linux/WSL - try Windows clipboard via PowerShell
            match Command::new("powershell.exe")
                .args(&["-NoProfile", "-Command", "Get-Clipboard -Raw"])
                .output() {
                Ok(output) if output.status.success() => {
                    String::from_utf8(output.stdout)
                        .unwrap_or_default()
                        .trim_end()
                        .to_string()
                }
                Ok(_) | Err(_) => String::new()
            }
        };
        
        if current_content != last_content && !current_content.is_empty() {
            let sanitized = sanitize_content(&current_content, &config.domains, &config.words, &mut wordmap);
            
            if sanitized != current_content {
                println!("Clipboard sanitized!");
                
                if cfg!(target_os = "windows") {
                    match write_windows_clipboard(&sanitized) {
                        Ok(()) => {},
                        Err(e) => eprintln!("could not write to clipboard: {}", e)
                    }
                } else if cfg!(target_os = "macos") {
                    match write_macos_clipboard(&sanitized) {
                        Ok(()) => {},
                        Err(e) => eprintln!("could not write to clipboard: {}", e)
                    }
                } else {
                    // Linux/WSL - try Windows clipboard via clip.exe
                    match Command::new("clip.exe")
                        .stdin(std::process::Stdio::piped())
                        .spawn() {
                        Ok(mut child) => {
                            if let Some(stdin) = child.stdin.as_mut() {
                                let _ = stdin.write_all(sanitized.as_bytes());
                            }
                            let _ = child.wait();
                        }
                        Err(e) => eprintln!("could not write to clipboard: {}", e)
                    }
                }
            }
            
            last_content = sanitized;
        }
        thread::sleep(Duration::from_millis(100));
    }
} 
fn load_config() -> Config {
    let config_content = match std::fs::read_to_string("config.yaml") {
        Ok(content) => content,
        Err(_) => {
            println!("Config file not found, using defaults");
            return Config { domains: vec![], words: vec![] };
        }
    };
    
    match serde_yaml::from_str(&config_content) {
        Ok(config) => config,
        Err(_) => {
            println!("Invalid config format, using defaults");
            Config { domains: vec![], words: vec![] }
        }
    }
}

fn sanitize_content(content: &str, domains: &[String], words: &[String], wordmap: &mut HashMap<String, String>) -> String {
    let mut result = content.to_string();

    result = sanitize_emails(&result);
    result = sanitize_api_keys(&result);
    result = sanitize_credit_cards(&result);
    result = sanitize_ip_addresses(&result);
    result = sanitize_domains(&result, domains);
    result = sanitize_words(&result, words, wordmap);
    
    result
}


fn read_windows_clipboard() -> Result<String, io::Error>{
    let output = Command::new("powershell.exe")
        .args(&["-Command", "Get-Clipboard"])
        .output()?;
    let text = String::from_utf8(output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(text)
}

fn write_windows_clipboard(content: &str) -> Result<(), io::Error>{
    let mut child = Command::new("clip.exe")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(content.as_bytes())?;
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to open stdin"));
    }

    child.wait()?;
    Ok(())
}

fn read_macos_clipboard() -> Result<String, io::Error> {
    let output = Command::new("pbpaste")
        .output()?;

    let text = String::from_utf8(output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(text)
}

fn write_macos_clipboard(content: &str) -> Result<(), io::Error>{
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(content.as_bytes())?;
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to open stdin"));
    }
    child.wait()?;
    Ok(())
}

/* Pattern-based sanitization functions */
fn sanitize_emails(content: &str) -> String {
    let re = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    re.replace_all(content, "[EMAIL]").to_string()
}

fn sanitize_api_keys(content: &str) -> String {
    let patterns = vec![
        r"sk-[a-zA-Z0-9]{48}",     // OpenAI
        r"AIza[a-zA-Z0-9]{35}",    // Google API
        r"ghp_[a-zA-Z0-9]{36}",    // GitHub Personal Token
        r"xoxb-[a-zA-Z0-9-]{50,}", // Slack Bot Token
        r"AKIA[0-9A-Z]{16}",       // AWS Access Key
    ];
    
    let mut result = content.to_string();
    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        result = re.replace_all(&result, "[API_KEY]").to_string();
    }
    result
}

fn sanitize_credit_cards(content: &str) -> String {
    let re = Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap();
    re.replace_all(content, "[CREDIT_CARD]").to_string()
}

fn sanitize_ip_addresses(content: &str) -> String {
    let re = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap();
    re.replace_all(content, "[IP_ADDRESS]").to_string()
}

fn sanitize_domains(content: &str, domains: &[String]) -> String {
    let mut result = content.to_string();
    for domain in domains {
        result = result.replace(domain, "[DOMAIN]");
    }
    result
}

fn sanitize_words(content: &str, words: &[String], wordmap: &mut HashMap<String, String>) -> String {
    let mut result = content.to_string();
      for word in words {
          let replacement = match wordmap.get(word) {
              Some(existing) => existing.clone(),      
              None => {
                  let new_word = generate_unique_word(wordmap);    
                  wordmap.insert(word.to_string(), new_word.clone());
                  new_word
              }
          };
          let re = Regex::new(word).unwrap();
          result = re.replace_all(&result, replacement).to_string();
      }
    result
}

fn generate_word() -> String {
    let adjectives = [
        "funny", "strong", "stinky", "grumpy"
    ];

    let subjectives = [
        "tree", "house", "mouse", "arm"
    ];

    let mut rng = rand::thread_rng();
    let adjectiv = adjectives[rng.gen_range(0..adjectives.len())];
    let subject = subjectives[rng.gen_range(0..subjectives.len())];

    format!("{}_{}", adjectiv, subject)
}

fn generate_unique_word(wordmap: &HashMap<String, String>) -> String {
    loop {
        let new_word = generate_word();
        if !wordmap.values().any(|v| v == &new_word) {
            return new_word;
        }
    }
}

