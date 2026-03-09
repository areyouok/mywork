use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::collections::HashSet;
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
