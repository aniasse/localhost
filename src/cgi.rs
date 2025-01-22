use std::process::{Command, Stdio};
use std::io::Write;

pub fn execute_cgi(script_path: &str, request_body: &str) -> String {
    let mut child = Command::new("php")
        .arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Erreur lors de l'exécution du CGI");

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(request_body.as_bytes()).unwrap();

    let output = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}
