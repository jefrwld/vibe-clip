use std::process::Command;
use std::io::Write;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    domains: Vec<String>,
}

fn main(){
    let config = load_config();

    if cfg!(target_os = "windows"){
        let content = read_windows_clipboard();
        let sanitized = sanitize_content(&content, &config.domains);
        println!("Clipboard Inhalt:");
        println!("{}", sanitized);

        write_windows_clipboard(&sanitized);

    } else if cfg!(target_os = "macos"){
        let content = read_macos_clipboard();
        let sanitized = sanitize_content(&content, &config.domains);
        println!("Clipboard Inhalt:");
        println!("{}", sanitized);
        write_macos_clipboard(&sanitized);
    }


}

fn load_config() -> Config {
    let config_content = std::fs::read_to_string("config.yaml").unwrap();
    serde_yaml::from_str(&config_content).unwrap()
}

fn sanitize_content(content: &str, domains: &[String]) -> String {
    let mut result = content.to_string();

    for domain in domains {
        result = result.replace(domain, "example.com");
    }

    result
}


fn read_windows_clipboard() -> String {
    let output = Command::new("powershell.exe")
        .args(&["-Command", "Get-Clipboard"])
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap() /* es returned von allein ? */
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

