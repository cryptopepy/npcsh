use std::path::{Path, PathBuf};

const USER_SUBTEAM_DIR: &str = "usr";
const MERGED_TEAM_DIR: &str = ".npc_team_merged";

/// Locate the bundled npc_team directory that ships with the npcsh binary.
///
/// Resolution order:
/// 1. `NPCSH_BUNDLED_TEAM` environment variable.
/// 2. A sibling of the executable: `<exe_dir>/../share/npcsh/npc_team`.
/// 3. A sibling of the executable used for local dev: `<exe_dir>/../../npcsh/npc_team`.
/// 4. Relative to the Cargo manifest directory: `<manifest_dir>/../npcsh/npc_team`.
pub fn find_bundled_team_source() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("NPCSH_BUNDLED_TEAM") {
        let p = PathBuf::from(path);
        if p.is_dir() {
            return Some(p);
        }
    }

    // Installed layout: ~/.npcsh/share/npcsh/npc_team
    if let Some(home) = real_user_home_path() {
        let installed = home
            .join(".npcsh")
            .join("share")
            .join("npcsh")
            .join("npc_team");
        if installed.is_dir() {
            return Some(installed);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidates = [
                exe_dir.join("..").join("share").join("npcsh").join("npc_team"),
                exe_dir.join("..").join("..").join("npcsh").join("npc_team"),
                exe_dir.join("npc_team"),
            ];
            for c in &candidates {
                if c.is_dir() {
                    return Some(c.clone());
                }
            }
        }
    }

    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest)
            .join("..")
            .join("npcsh")
            .join("npc_team");
        if p.is_dir() {
            return Some(p);
        }
    }

    None
}

fn real_user_home_path() -> Option<PathBuf> {
    #[cfg(unix)]
    unsafe {
        let uid = libc::getuid();
        let pw = libc::getpwuid(uid);
        if !pw.is_null() {
            let home_ptr = (*pw).pw_dir;
            if !home_ptr.is_null() {
                let home = std::ffi::CStr::from_ptr(home_ptr)
                    .to_string_lossy()
                    .to_string();
                if !home.is_empty() {
                    return Some(PathBuf::from(home));
                }
            }
        }
    }
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Copy a directory recursively, skipping any excluded top-level entries.
fn copy_dir_filtered(
    src: &Path,
    dst: &Path,
    exclude: &[&str],
) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if exclude.iter().any(|e| name == *e) {
            continue;
        }
        let dest = dst.join(&name);
        let path = entry.path();
        if path.is_dir() {
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// Copy the bundled base team into the user's global team directory.
/// The `usr/` subfolder is preserved and never overwritten. Any base files
/// that exist in the user dir but no longer exist in the bundled team are
/// removed so the install stays faithful to the release.
pub fn sync_team(base_src: &Path, user_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(user_dir)?;

    // Remove stale base entries, preserving the user subteam.
    if let Ok(entries) = std::fs::read_dir(user_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name == USER_SUBTEAM_DIR {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(&path);
            } else {
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    copy_dir_filtered(base_src, user_dir, &[USER_SUBTEAM_DIR])?;
    Ok(())
}

/// Ensure the user-specific subteam exists inside the global team directory.
pub fn ensure_user_subteam(user_dir: &Path) -> std::io::Result<()> {
    let usr = user_dir.join(USER_SUBTEAM_DIR);
    std::fs::create_dir_all(usr.join("jinxes"))?;

    let ctx_path = usr.join("usr.ctx");
    if !ctx_path.exists() {
        let ctx = "context: |\n  \"\"\"\n  User subteam context.\n\n  Add your own preferences, directives, and persistent instructions here.\n  This context is appended to the base npcsh team context.\n  \"\"\"\n";
        std::fs::write(&ctx_path, ctx)?;
    }

    let npc_path = usr.join("user.npc");
    if !npc_path.exists() {
        let npc = "#!/usr/bin/env npc\n\
name: user\n\
primary_directive: Your personal assistant with access to your preferences and user-specific tools.\n\
model: qwen3.5:2b\n\
provider: ollama\n\
jinxes:\n\
  - sh\n\
  - chat\n";
        std::fs::write(&npc_path, npc)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&npc_path, std::fs::Permissions::from_mode(0o755))?;
        }
    }

    Ok(())
}

/// Merge a base `.ctx` file with a user `.ctx` file.
///
/// The `context` fields are concatenated; all other top-level fields from the
/// user file override the base file.
fn merge_ctx_files(base_path: &Path, usr_path: &Path, out_path: &Path) -> std::io::Result<()> {
    let base_raw = std::fs::read_to_string(base_path)?;
    let usr_raw = std::fs::read_to_string(usr_path)?;

    let base_val: serde_yaml::Value =
        serde_yaml::from_str(&base_raw).unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    let usr_val: serde_yaml::Value =
        serde_yaml::from_str(&usr_raw).unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    let mut merged = base_val;
    if let (
        serde_yaml::Value::Mapping(m1),
        serde_yaml::Value::Mapping(m2),
    ) = (&mut merged, usr_val)
    {
        for (k, v) in m2.iter() {
            if k.as_str() == Some("context") {
                let base_ctx = m1
                    .get("context")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let usr_ctx = v.as_str().unwrap_or("");
                let combined = if base_ctx.is_empty() {
                    usr_ctx.to_string()
                } else {
                    format!("{}\n\n# ── User subteam context ──\n{}", base_ctx, usr_ctx)
                };
                m1.insert(
                    k.clone(),
                    serde_yaml::Value::String(combined),
                );
            } else {
                m1.insert(k.clone(), v.clone());
            }
        }
    }

    std::fs::write(
        out_path,
        serde_yaml::to_string(&merged).unwrap_or_default(),
    )?;
    Ok(())
}

/// Build a merged view of the base team plus the `usr/` subteam overlay.
///
/// The merged directory is cleared and rebuilt each time. It is safe to delete
/// and recreate because all user data lives in `usr/`.
pub fn build_merged_team_dir(user_dir: &Path) -> std::io::Result<PathBuf> {
    let merged = user_dir
        .parent()
        .unwrap_or(user_dir)
        .join(MERGED_TEAM_DIR);

    let _ = std::fs::remove_dir_all(&merged);
    std::fs::create_dir_all(&merged)?;

    // Copy base team, skipping the user overlay folder.
    for entry in std::fs::read_dir(user_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == USER_SUBTEAM_DIR {
            continue;
        }
        let dest = merged.join(&name);
        let path = entry.path();
        if path.is_dir() {
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }

    // Overlay user subteam on top of the base copy.
    let usr = user_dir.join(USER_SUBTEAM_DIR);
    if usr.exists() {
        for entry in std::fs::read_dir(&usr)? {
            let entry = entry?;
            let name = entry.file_name();
            let dest = merged.join(&name);
            let path = entry.path();
            if path.is_dir() {
                copy_dir(&path, &dest)?;
            } else if name.to_string_lossy().ends_with(".ctx") && dest.exists() {
                merge_ctx_files(&dest, &path, &dest)?;
            } else {
                std::fs::copy(&path, &dest)?;
            }
        }
    }

    Ok(merged)
}

/// Ensure the global user team is set up and return the merged team directory.
///
/// If a bundled team source can be found, the base team is synced into
/// `~/.npcsh/npc_team/` (preserving `usr/`). The `usr/` subteam skeleton is
/// created if missing, and a merged view is returned for the kernel to boot.
pub fn setup_global_team(home: &Path) -> std::io::Result<PathBuf> {
    let user_dir = home.join(".npcsh").join("npc_team");
    std::fs::create_dir_all(&user_dir)?;

    if let Some(base_src) = find_bundled_team_source() {
        sync_team(&base_src, &user_dir)?;
    }

    ensure_user_subteam(&user_dir)?;
    build_merged_team_dir(&user_dir)
}
