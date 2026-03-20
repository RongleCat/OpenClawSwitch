use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn trace_log_path() -> Option<PathBuf> {
    let home_dir = dirs::home_dir()?;
    Some(
        home_dir
            .join(".openclaw")
            .join("logs")
            .join("desktop-startup-trace.log"),
    )
}

fn build_trace_line(label: &str, detail: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{now}\tpid={}\t{label}\t{detail}\n", std::process::id())
}

pub fn append(label: &str, detail: impl AsRef<str>) {
    let Some(path) = trace_log_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = file.write_all(build_trace_line(label, detail.as_ref()).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::build_trace_line;

    #[test]
    fn build_trace_line_includes_label_and_detail() {
        let line = build_trace_line("setup.begin", "launchAtStartup=true");

        assert!(line.contains("setup.begin"));
        assert!(line.contains("launchAtStartup=true"));
        assert!(line.contains("pid="));
    }
}
