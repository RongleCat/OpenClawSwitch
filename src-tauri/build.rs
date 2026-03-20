use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn target_dir_from_out_dir(out_dir: &Path) -> Option<PathBuf> {
    Some(out_dir.parent()?.parent()?.parent()?.to_path_buf())
}

fn clear_stale_bundled_node_runtime(target_dir: &Path) -> std::io::Result<()> {
    let bundled_node_dir = target_dir.join("resources").join("vendor").join("node");
    if bundled_node_dir.exists() {
        fs::remove_dir_all(bundled_node_dir)?;
    }
    Ok(())
}

fn main() {
    if let Some(out_dir) = env::var_os("OUT_DIR") {
        if let Some(target_dir) = target_dir_from_out_dir(Path::new(&out_dir)) {
            clear_stale_bundled_node_runtime(&target_dir)
                .expect("failed to clear stale bundled node runtime artifacts");
        }
    }

    tauri_build::build()
}
