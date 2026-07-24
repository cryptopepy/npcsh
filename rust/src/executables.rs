use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ExecutableInfo {
    pub name: String,
    pub path: PathBuf,
    pub realpath: PathBuf,
    pub category: String,
    pub risk: String,
    pub mode: u32,
}

fn known_category(name: &str) -> Option<&'static str> {
    let name = name.to_lowercase();
    let read: HashSet<&str> = [
        "cat", "tac", "head", "tail", "less", "more", "nl", "wc", "awk",
        "grep", "egrep", "fgrep", "rg", "find", "ls", "exa", "tree",
        "file", "stat", "du", "df", "readlink", "realpath", "basename",
        "dirname", "strings", "hexdump", "od", "xxd", "git", "which",
        "whereis", "ldd", "nm", "objdump", "readelf", "ps", "top", "htop",
        "pidof", "pgrep", "whoami", "id", "groups", "env", "printenv",
        "date", "cal", "uptime", "uname", "hostname", "curl", "wget",
        "python", "python3", "node", "ruby", "perl",
    ]
    .iter()
    .copied()
    .collect();

    let write: HashSet<&str> = [
        "cp", "mv", "rm", "rmdir", "mkdir", "touch", "chmod", "chown",
        "chgrp", "ln", "sed", "tee", "dd", "truncate", "fallocate",
        "rename", "rsync", "scp", "sftp", "install", "tar", "zip",
        "unzip", "gzip", "gunzip", "bzip2", "bunzip2", "xz", "unxz",
    ]
    .iter()
    .copied()
    .collect();

    let execute: HashSet<&str> = [
        "sh", "bash", "zsh", "fish", "dash", "ksh", "csh", "tcsh",
        "npm", "yarn", "pnpm", "bundle", "rails", "php", "make",
        "cmake", "ninja", "gcc", "g++", "clang", "clang++", "rustc",
        "cargo", "go", "javac", "java", "gradle", "mvn", "sbt", "ant",
        "docker", "docker-compose", "podman", "kubectl", "helm",
        "terraform", "ansible-playbook", "vagrant", "systemctl",
        "service", "screen", "tmux", "nohup", "timeout", "watch",
    ]
    .iter()
    .copied()
    .collect();

    if read.contains(name.as_str()) {
        return Some("read");
    }
    if write.contains(name.as_str()) {
        return Some("write");
    }
    if execute.contains(name.as_str()) {
        return Some("execute");
    }
    None
}

fn risk_level(mode: u32, path: &Path) -> &'static str {
    let setuid = mode & 0o4000 != 0;
    let setgid = mode & 0o2000 != 0;
    let world_writable = mode & 0o0002 != 0;
    if setuid || setgid || world_writable {
        return "high";
    }
    let temp_dir = path
        .ancestors()
        .any(|p| p == Path::new("/tmp") || p == Path::new("/var/tmp") || p == Path::new("/dev/shm"));
    if temp_dir {
        return "high";
    }
    "low"
}

fn categorize(name: &str, mode: u32, path: &Path) -> String {
    if let Some(cat) = known_category(name) {
        return cat.to_string();
    }
    if mode & 0o4000 != 0 || mode & 0o2000 != 0 || mode & 0o0002 != 0 {
        return "execute".to_string();
    }
    "unknown".to_string()
}

pub fn discover_executables(extra_paths: Option<&str>) -> Vec<ExecutableInfo> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(p) = extra_paths {
        for part in p.split(':') {
            let pb = PathBuf::from(part.trim());
            if pb.is_dir() {
                dirs.push(pb);
            }
        }
    } else if let Ok(path_var) = std::env::var("PATH") {
        for part in path_var.split(':') {
            let pb = PathBuf::from(part.trim());
            if pb.is_dir() {
                dirs.push(pb);
            }
        }
    }

    let mut seen: HashSet<(String, PathBuf)> = HashSet::new();
    let mut results: Vec<ExecutableInfo> = Vec::new();

    for dir in dirs {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let mode = meta.mode();
            let is_exec = mode & 0o111 != 0;
            if !meta.is_file() || !is_exec {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let real = fs::canonicalize(entry.path()).unwrap_or_else(|_| entry.path());
            let key = (name.to_lowercase(), real.clone());
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);
            let cat = categorize(&name, mode, &real);
            let risk = risk_level(mode, &real);
            results.push(ExecutableInfo {
                name,
                path: entry.path(),
                realpath: real,
                category: cat,
                risk: risk.to_string(),
                mode,
            });
        }
    }

    results.sort_by(|a, b| a.name.cmp(&b.name));
    results
}

pub fn format_executables_context(execs: &[ExecutableInfo]) -> String {
    let mut by_cat: HashMap<&str, Vec<&ExecutableInfo>> = HashMap::new();
    for e in execs {
        by_cat.entry(e.category.as_str()).or_default().push(e);
    }

    let mut lines = vec!["Available executables (discovered at runtime):".to_string()];
    for cat in ["read", "write", "execute", "unknown"] {
        let list = by_cat.get(cat).unwrap_or(&Vec::new());
        if list.is_empty() {
            continue;
        }
        lines.push(format!("\n{} tools:", cat.to_uppercase()));
        for e in list.iter().take(40) {
            let high = if e.risk == "high" { " [HIGH RISK]" } else { "" };
            lines.push(format!("  - {}{}", e.name, high));
        }
        if list.len() > 40 {
            lines.push(format!("  ... and {} more", list.len() - 40));
        }
    }
    lines.join("\n")
}
