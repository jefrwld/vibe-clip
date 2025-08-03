use std::process::Command;
use std::io::Write;
use std::time::Duration;
use std::thread;
use serde::Deserialize;
use regex::Regex;

#[derive(Deserialize)]
struct Config {
    domains: Vec<String>,
    words: Vec<String>,
}

fn main(){
    let config = load_config();
    let mut last_content = String::new();
    
    println!("Clipboard monitoring started. Press Ctrl+C to stop.");
    
    loop {
        let current_content = if cfg!(target_os = "windows") {
            read_windows_clipboard()
        } else if cfg!(target_os = "macos") {
            read_macos_clipboard()
        } else {
            String::new()
        };
        
        if current_content != last_content && !current_content.is_empty() {
            let sanitized = sanitize_content(&current_content, &config.domains, &config.words);
            
            if sanitized != current_content {
                println!("Clipboard sanitized!");
                
                if cfg!(target_os = "windows") {
                    write_windows_clipboard(&sanitized);
                } else if cfg!(target_os = "macos") {
                    write_macos_clipboard(&sanitized);
                }
            }
            
            last_content = sanitized;
        }
        
        thread::sleep(Duration::from_millis(100));
    }
}

fn load_config() -> Config {
    let config_content = std::fs::read_to_string("config.yaml").unwrap();
    serde_yaml::from_str(&config_content).unwrap()
}

fn sanitize_content(content: &str, domains: &[String], words: &[String]) -> String {
    let mut result = content.to_string();
    
    // 1. Automatische Pattern-Erkennung (immer aktiv)
    result = sanitize_emails(&result);
    result = sanitize_api_keys(&result);
    result = sanitize_credit_cards(&result);
    result = sanitize_ip_addresses(&result);
    
    // 2. Custom Config-Regeln
    result = sanitize_domains(&result, domains);
    result = sanitize_words(&result, words);
    
    result
}


fn read_windows_clipboard() -> String {
    let output = Command::new("powershell.exe")
        .args(&["-Command", "Get-Clipboard"])
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap()
}

fn write_windows_clipboard(content: &str){
    let mut child = Command::new("clip.exe")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(content.as_bytes()).unwrap();
    child.wait().unwrap();
}

fn read_macos_clipboard() -> String {
    let output = Command::new("pbpaste")
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap()

}

fn write_macos_clipboard(content: &str){
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(content.as_bytes()).unwrap();
    child.wait().unwrap();
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

fn sanitize_words(content: &str, words: &[String]) -> String {
    let mut result = content.to_string();
    for word in words {
        let re = Regex::new(word).unwrap();
        result = re.replace_all(&result, "[REPLACED]").to_string();
    }
    result
}

