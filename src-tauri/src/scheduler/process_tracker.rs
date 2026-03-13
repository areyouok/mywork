use crate::environment::hydrated_path;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

static RUNNING_PIDS: OnceLock<Mutex<HashSet<u32>>> = OnceLock::new();

fn get_pids() -> &'static Mutex<HashSet<u32>> {
    RUNNING_PIDS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn register_pid(pid: u32) {
    if let Ok(mut pids) = get_pids().lock() {
        pids.insert(pid);
    }
}

pub fn unregister_pid(pid: u32) {
    if let Ok(mut pids) = get_pids().lock() {
        pids.remove(&pid);
    }
}

pub fn kill_all_processes() {
    let pids: Vec<u32> = {
        if let Ok(mut pids) = get_pids().lock() {
            pids.drain().collect()
        } else {
            return;
        }
    };

    for pid in pids {
        let pgid = Pid::from_raw(-(pid as i32));
        let _ = kill(pgid, Signal::SIGKILL);
    }
}

pub fn running_count() -> usize {
    if let Ok(pids) = get_pids().lock() {
        pids.len()
    } else {
        0
    }
}

pub fn cleanup_orphan_processes(app_data_dir: &Path) {
    let workdir = app_data_dir.to_string_lossy().to_string();

    let mut ps_cmd = Command::new("ps");
    ps_cmd.args(["-eo", "pid,comm"]);
    if let Some(path) = hydrated_path() {
        ps_cmd.env("PATH", path);
    }
    let output = ps_cmd.output();

    let Ok(output) = output else {
        return;
    };

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        if !line.contains("opencode") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let pid: u32 = match parts.first() {
            Some(p) => match p.parse() {
                Ok(n) => n,
                Err(_) => continue,
            },
            None => continue,
        };

        let mut pwdx_cmd = Command::new("pwdx");
        pwdx_cmd.arg(pid.to_string());
        if let Some(path) = hydrated_path() {
            pwdx_cmd.env("PATH", path);
        }
        let cwd_output = pwdx_cmd.output();
        let Ok(cwd_output) = cwd_output else {
            continue;
        };

        let cwd = String::from_utf8_lossy(&cwd_output.stdout);

        if cwd.contains(&workdir) {
            let pgid = Pid::from_raw(-(pid as i32));
            let _ = kill(pgid, Signal::SIGKILL);
            eprintln!("Cleaned up orphan opencode process: {}", pid);
        }
    }
}
