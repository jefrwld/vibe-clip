use std::process::Command;
use std::io::Write;

fn main(){

    let content = read_windows_clipboard();
    let sanitized = content.replace("meine-firma.com", "example.com");
    println!("Clipboard Inhalt:");
    println!("{}", sanitized);

    write_windows_clipboard(&sanitized);

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
