use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_user_subteam(home: &Path) -> std::io::Result<PathBuf> {
    let team = home.join(".npcsh").join("npc_team");
    let usr = team.join("usr");
    let jinxes = usr.join("jinxes");
    if !jinxes.exists() {
        fs::create_dir_all(&jinxes)?;
    }
    let user_npc = usr.join("user.npc");
    if !user_npc.exists() {
        fs::write(
            &user_npc,
            "# User NPC\nname: user\ndescription: The human user.\n",
        )?;
    }
    let usr_ctx = usr.join("usr.ctx");
    if !usr_ctx.exists() {
        fs::write(
            &usr_ctx,
            "# User context\n# Put personal context, preferences, and reminders here.\n",
        )?;
    }
    Ok(usr)
}

pub fn sync_team(source: &Path, target: &Path) -> std::io::Result<()> {
    if !source.exists() || !source.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(target)?;
    ensure_user_subteam(&target.ancestors().nth(2).unwrap_or(target))?;

    // Remove stale base entries in target, never touching usr/.
    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "usr" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    // Copy current base files from source, skipping its own usr/ if present.
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "usr" {
            continue;
        }
        let src = entry.path();
        let dst = target.join(&name);
        if src.is_dir() {
            copy_dir_all(&src, &dst)?;
        } else {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst)?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(name))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(name))?;
        }
    }
    Ok(())
}
