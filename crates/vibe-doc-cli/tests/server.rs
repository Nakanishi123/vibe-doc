use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn server_rejects_invalid_host_and_port_with_stable_diagnostics() {
    let repo = TestRepo::new("cli-server-invalid");

    let host = run_vdoc(repo.path(), ["server", "--host", "localhost"]);
    assert!(!host.status.success());
    assert_eq!(stdout(&host), "");
    assert!(stderr(&host).contains("invalid host `localhost`; expected an IP address"));

    let port = run_vdoc(repo.path(), ["server", "--port", "70000"]);
    assert!(!port.status.success());
    assert_eq!(stdout(&port), "");
    assert!(stderr(&port).contains("invalid port `70000`; expected 0-65535"));
}

#[test]
fn server_prints_json_startup_information_after_binding() {
    if loopback_bind_is_restricted() {
        return;
    }

    let repo = TestRepo::new("cli-server-json");
    let mut child = Command::new(env!("CARGO_BIN_EXE_vdoc"))
        .current_dir(repo.path())
        .args(["server", "--port", "0", "--json"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
        let _ = sender.send(result);
    });

    let line = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("server should print startup JSON")
        .unwrap();
    child.kill().unwrap();
    let _ = child.wait();

    let value: Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(value["command"], "server");
    assert_eq!(value["host"], "127.0.0.1");
    assert_ne!(value["port"], 0);
    assert!(value["url"]
        .as_str()
        .unwrap()
        .starts_with("http://127.0.0.1:"));
    assert_eq!(
        value["repository_root"].as_str().unwrap(),
        repo.path().to_string_lossy()
    );
}

fn loopback_bind_is_restricted() -> bool {
    match TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))) {
        Ok(listener) => {
            drop(listener);
            false
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => true,
        Err(_) => false,
    }
}

fn run_vdoc<const N: usize>(cwd: &Path, args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vdoc"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

struct TestRepo {
    root: PathBuf,
}

impl TestRepo {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("vibe-doc-{name}-{unique}"));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}
