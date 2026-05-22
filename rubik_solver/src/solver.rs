use kewb::{CubieCube, DataTable, FaceCube, Solver};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const DAEMON_PORT: u16 = 10023;
const DAEMON_SCRIPT: &str = include_str!("nxn_daemon.py");

/// Helper function to ensure the Python solver daemon is active and listening on the designated port.
/// If not active, it will automatically spawn the daemon and await its readiness.
fn ensure_daemon_running(port: u16) -> bool {
    // Try connecting first. If already running, connection succeeds immediately.
    if TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
        return true;
    }

    // Write python daemon script dynamically to ensure it is always present and up-to-date
    if let Err(e) = std::fs::write("python_solver/nxn_daemon.py", DAEMON_SCRIPT) {
        eprintln!("Failed to write dynamic nxn_daemon.py script: {e}");
        return false;
    }

    let python_path = std::fs::canonicalize(".venv/bin/python")
        .unwrap_or_else(|_| std::path::PathBuf::from(".venv/bin/python"));
    let daemon_path = std::fs::canonicalize("python_solver/nxn_daemon.py")
        .unwrap_or_else(|_| std::path::PathBuf::from("python_solver/nxn_daemon.py"));

    let mut path_env = std::env::var("PATH").unwrap_or_default();
    if let Ok(venv_bin_absolute) = std::fs::canonicalize(".venv/bin") {
        let venv_str = venv_bin_absolute.to_string_lossy();
        path_env = format!("{venv_str}:{path_env}");
    }

    let mut child = match Command::new(python_path)
        .arg(daemon_path)
        .arg(port.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .current_dir("python_solver/rubiks-cube-NxNxN-solver")
        .env("PATH", path_env)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to spawn solver daemon: {e}");
            return false;
        }
    };

    // Await the readiness token from daemon stdout with a 30s timeout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let start = Instant::now();
        let timeout = Duration::from_secs(30);

        for line in reader.lines() {
            if start.elapsed() > timeout {
                eprintln!("Timeout waiting for solver daemon to be ready.");
                break;
            }
            if let Ok(line_str) = line {
                if line_str.contains(&format!("READY:{port}")) {
                    return true;
                }
            }
        }
    }

    // Verify connection to be fully certain
    TcpStream::connect(format!("127.0.0.1:{port}")).is_ok()
}

/// Shuts down the active solver daemon by sending a SHUTDOWN signal.
pub fn shutdown_daemon(port: u16) {
    if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{port}")) {
        let _ = stream.write_all(b"SHUTDOWN\n");
    }
}

/// Invokes the Python daemon solver for `NxN` (size >= 4) with a given state string.
/// Communicates over local TCP socket to bypass Python startup time overhead.
pub fn solve_nxn_state_only(state_str: &str) -> Option<Vec<String>> {
    if !ensure_daemon_running(DAEMON_PORT) {
        eprintln!("NxN solver daemon is not running and could not be started.");
        return None;
    }

    let mut stream = match TcpStream::connect(format!("127.0.0.1:{DAEMON_PORT}")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to NxN solver daemon: {e}");
            return None;
        }
    };

    // Set robust timeouts to prevent freezing in case of processing anomalies.
    // Use 300s (5 minutes) read timeout to accommodate S3 lookup table download on first run.
    let _ = stream.set_read_timeout(Some(Duration::from_secs(300)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let request = format!("{state_str}\n");
    if let Err(e) = stream.write_all(request.as_bytes()) {
        eprintln!("Failed to send state request to daemon: {e}");
        return None;
    }

    let reader = BufReader::new(stream);
    for line in reader.lines() {
        match line {
            Ok(line_str) => {
                if line_str.starts_with("SUCCESS:") {
                    let sol_part = line_str.trim_start_matches("SUCCESS:").trim();
                    if sol_part.is_empty() {
                        return Some(Vec::new());
                    }
                    let moves = sol_part
                        .split_whitespace()
                        .map(String::from)
                        .collect::<Vec<String>>();
                    return Some(moves);
                } else if line_str.starts_with("ERROR:") {
                    let err_msg = line_str.trim_start_matches("ERROR:").trim();
                    eprintln!("NxN solver daemon returned error: {err_msg}");
                    return None;
                }
            }
            Err(e) => {
                eprintln!("Failed reading response from NxN solver daemon: {e}");
                return None;
            }
        }
    }

    None
}

/// Unified solver function for all supported cube sizes.
/// It solves the cube state using the Kociemba table or Python solver.
#[allow(clippy::cast_sign_loss)]
pub fn solve_cube_for_size(size: i32, state_str: &str, table: &DataTable) -> Option<Vec<String>> {
    if size >= 4 {
        solve_nxn_state_only(state_str)
    } else {
        solve_cube(state_str, table)
    }
}

pub fn solve_cube(state_str: &str, table: &DataTable) -> Option<Vec<String>> {
    let face_cube = FaceCube::try_from(state_str).ok()?;
    let cubie_cube = CubieCube::try_from(&face_cube).ok()?;
    let mut solver = Solver::new(table, 23, None);
    let sol = solver.solve(cubie_cube)?;
    Some(
        sol.to_string()
            .split_whitespace()
            .map(String::from)
            .collect(),
    )
}
