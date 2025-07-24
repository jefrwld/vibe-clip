use std::process::Command;

fn main(){
    let output = Command::new("powershell.exe")
        .args(&["-Command", "Get-Clipboard"])
        .output()
        .unwrap();


    let content = String::from_utf8(output.stdout).unwrap();
    let sanitized = content.replace("meine-firma.com", "example.com");
    println!("Clipboard Inhalt:");
    println!("{}", sanitized);
}
