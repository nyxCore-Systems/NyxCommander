use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::command;

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CmdError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

impl Serialize for CmdError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

type CmdResult<T> = Result<T, CmdError>;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: u64,
    pub extension: String,
    pub is_hidden: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct VolumeInfo {
    pub name: String,
    pub path: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory_kb: u64,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemStats {
    pub total_memory_kb: u64,
    pub used_memory_kb: u64,
    pub cpu_usage: f32,
    pub process_count: usize,
}

pub struct SystemState(pub Mutex<System>);

// ─── Commands ─────────────────────────────────────────────────────────────────

/// List directory contents. Returns entries sorted: dirs first (alpha), then
/// files (alpha). Includes a synthetic ".." entry when path is not root.
#[command]
pub fn list_dir(path: String) -> CmdResult<Vec<FileEntry>> {
    let dir = Path::new(&path);
    let mut entries: Vec<FileEntry> = Vec::new();

    // Synthetic parent entry (omit at filesystem root)
    if let Some(parent) = dir.parent() {
        entries.push(FileEntry {
            name: "..".to_string(),
            path: parent.to_string_lossy().to_string(),
            is_dir: true,
            is_symlink: false,
            size: 0,
            modified: 0,
            extension: String::new(),
            is_hidden: false,
        });
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let symlink_meta = entry.path().symlink_metadata()?;
        let is_symlink = symlink_meta.file_type().is_symlink();
        // Follow symlinks for size/date; fall back to symlink_meta on error
        let meta = entry.metadata().unwrap_or(symlink_meta);
        let name = entry.file_name().to_string_lossy().to_string();
        let extension = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        entries.push(FileEntry {
            path: entry.path().to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            is_symlink,
            size: if meta.is_dir() { 0 } else { meta.len() },
            modified,
            extension,
            is_hidden: name.starts_with('.'),
            name,
        });
    }

    // Sort: ".." first, then dirs alpha, then files alpha
    entries.sort_by(|a, b| {
        if a.name == ".." {
            return std::cmp::Ordering::Less;
        }
        if b.name == ".." {
            return std::cmp::Ordering::Greater;
        }
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

/// List mounted volumes.
/// - macOS: reads /Volumes
/// - Linux: parses /proc/mounts, filters to real storage filesystems
/// - other: returns home directory as single entry
#[command]
pub fn list_volumes() -> CmdResult<Vec<VolumeInfo>> {
    #[cfg(target_os = "macos")]
    {
        let mut vols = Vec::new();
        for entry in std::fs::read_dir("/Volumes")? {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                let path = entry.path().to_string_lossy().to_string();
                let name = entry.file_name().to_string_lossy().to_string();
                let (total_bytes, free_bytes) = statvfs_bytes(&path);
                vols.push(VolumeInfo { name, path, total_bytes, free_bytes });
            }
        }
        return Ok(vols);
    }

    #[cfg(target_os = "linux")]
    {
        use std::io::{BufRead, BufReader};
        use std::collections::HashSet;

        // Filesystem types that represent real storage (not pseudo/virtual fs).
        const REAL_FS: &[&str] = &[
            "ext2", "ext3", "ext4", "btrfs", "xfs", "zfs", "jfs", "reiserfs",
            "nilfs2", "f2fs", "ntfs", "ntfs3", "vfat", "exfat", "hfsplus",
            "fuseblk", "fuse", "overlay", "squashfs",
        ];

        let f = std::fs::File::open("/proc/mounts")?;
        let reader = BufReader::new(f);
        let mut vols = Vec::new();
        let mut seen = HashSet::new();

        for line in reader.lines().flatten() {
            // Format: <device> <mountpoint> <fstype> <options> <dump> <pass>
            let mut parts = line.splitn(6, ' ');
            let _device     = parts.next().unwrap_or("");
            let mount_point = parts.next().unwrap_or("");
            let fs_type     = parts.next().unwrap_or("");

            if !REAL_FS.contains(&fs_type) { continue; }
            if !seen.insert(mount_point.to_string()) { continue; }

            let name = if mount_point == "/" {
                "root".to_string()
            } else {
                std::path::Path::new(mount_point)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| mount_point.to_string())
            };

            let (total_bytes, free_bytes) = statvfs_bytes(mount_point);
            vols.push(VolumeInfo { name, path: mount_point.to_string(), total_bytes, free_bytes });
        }

        if vols.is_empty() {
            // Fallback: at minimum expose root so the panel isn't blank
            let (total_bytes, free_bytes) = statvfs_bytes("/");
            vols.push(VolumeInfo {
                name: "root".to_string(),
                path: "/".to_string(),
                total_bytes,
                free_bytes,
            });
        }

        return Ok(vols);
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        Ok(vec![VolumeInfo {
            name: "Home".to_string(),
            path: home,
            total_bytes: 0,
            free_bytes: 0,
        }])
    }
}

// statvfs is POSIX — available on both macOS and Linux.
#[cfg(unix)]
fn statvfs_bytes(path: &str) -> (u64, u64) {
    use std::ffi::CString;
    let Ok(cpath) = CString::new(path) else { return (0, 0) };
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::statvfs(cpath.as_ptr(), &mut stat) };
    if ret == 0 {
        let total = stat.f_blocks as u64 * stat.f_frsize as u64;
        let free  = stat.f_bfree  as u64 * stat.f_frsize as u64;
        (total, free)
    } else {
        (0, 0)
    }
}

/// Copy srcs into dst_dir. Directories are copied recursively.
#[command]
pub fn copy_items(srcs: Vec<String>, dst_dir: String) -> CmdResult<()> {
    let dst = Path::new(&dst_dir);
    for src_str in &srcs {
        let src = Path::new(src_str);
        let name = src
            .file_name()
            .ok_or_else(|| CmdError::Other("invalid source path".into()))?;
        let dst_path = dst.join(name);
        if src.is_dir() {
            copy_dir_recursive(src, &dst_path)?;
        } else {
            std::fs::copy(src, &dst_path)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> CmdResult<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_child = dst.join(entry.file_name());
        if entry.metadata()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_child)?;
        } else {
            std::fs::copy(entry.path(), dst_child)?;
        }
    }
    Ok(())
}

/// Move srcs into dst_dir. Tries rename first; falls back to copy+delete for cross-device.
#[command]
pub fn move_items(srcs: Vec<String>, dst_dir: String) -> CmdResult<()> {
    let dst = Path::new(&dst_dir);
    for src_str in &srcs {
        let src = Path::new(src_str);
        let name = src
            .file_name()
            .ok_or_else(|| CmdError::Other("invalid source path".into()))?;
        let dst_path = dst.join(name);
        match std::fs::rename(src, &dst_path) {
            Ok(_) => {}
            Err(_) => {
                // Cross-device move: copy then remove
                if src.is_dir() {
                    copy_dir_recursive(src, &dst_path)?;
                    std::fs::remove_dir_all(src)?;
                } else {
                    std::fs::copy(src, &dst_path)?;
                    std::fs::remove_file(src)?;
                }
            }
        }
    }
    Ok(())
}

/// Delete paths. Directories removed recursively.
#[command]
pub fn delete_items(paths: Vec<String>) -> CmdResult<()> {
    for p in &paths {
        let path = Path::new(p);
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Create a directory (and all parents).
#[command]
pub fn create_dir(path: String) -> CmdResult<()> {
    std::fs::create_dir_all(&path)?;
    Ok(())
}

/// Rename a path. new_name is just the filename component (no path separators).
#[command]
pub fn rename_path(src: String, new_name: String) -> CmdResult<()> {
    let src_path = Path::new(&src);
    let parent = src_path
        .parent()
        .ok_or_else(|| CmdError::Other("cannot rename root".into()))?;
    let dst = parent.join(&new_name);
    std::fs::rename(src_path, dst)?;
    Ok(())
}

/// Open a file or directory with the default system application.
#[command]
pub fn open_file(path: String, app: tauri::AppHandle) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| CmdError::Other(e.to_string()))?;
    Ok(())
}

/// Get home directory path.
#[command]
pub fn get_home() -> String {
    // HOME is standard on macOS and Linux.
    // Fallback to / so the panel always has somewhere to start.
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}

/// Read raw bytes from any file. Returns up to `count` bytes starting at `offset`.
/// Used by the hex viewer; loads one 64 KB page at a time.
#[command]
pub fn read_file_bytes(path: String, offset: u64, count: u64) -> CmdResult<ByteChunk> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(&path)?;
    let file_size = f.metadata()?.len();
    let start = offset.min(file_size);
    if start > 0 { f.seek(SeekFrom::Start(start))?; }
    let to_read = count.min(file_size.saturating_sub(start)) as usize;
    let mut data = vec![0u8; to_read];
    f.read_exact(&mut data)?;
    Ok(ByteChunk { data, file_size, offset: start })
}

#[derive(Debug, Serialize, Clone)]
pub struct ByteChunk {
    pub data: Vec<u8>,
    pub file_size: u64,
    pub offset: u64,
}

/// Read a text file, capped at 512 KB.
#[command]
pub fn read_text_file(path: String) -> CmdResult<String> {
    const MAX: u64 = 512 * 1024;
    let meta = std::fs::metadata(&path)?;
    if meta.len() > MAX {
        return Err(CmdError::Other(format!(
            "File too large to preview ({} bytes, limit 512 KB)",
            meta.len()
        )));
    }
    Ok(std::fs::read_to_string(&path)?)
}

/// Run unified `diff -u left right` and return stdout.
/// Exit code 1 (files differ) is normal, not an error; exit code 2 is an error.
#[command]
pub fn diff_files(left: String, right: String) -> CmdResult<String> {
    let out = std::process::Command::new("diff")
        .args(["-u", "--", &left, &right])
        .output()
        .map_err(|e| CmdError::Other(format!("diff: {e}")))?;
    if out.status.code() == Some(2) {
        return Err(CmdError::Other(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Search for files/dirs whose name contains `pattern` (case-insensitive).
/// Set recursive=false to search only the immediate directory.
#[command]
pub fn search_files(
    root: String,
    pattern: String,
    recursive: bool,
    max_results: usize,
) -> CmdResult<Vec<FileEntry>> {
    let limit = if max_results == 0 { 500 } else { max_results };
    let mut results = Vec::new();
    search_recursive(Path::new(&root), &pattern.to_lowercase(), recursive, &mut results, limit);
    Ok(results)
}

fn search_recursive(dir: &Path, pattern: &str, recursive: bool, results: &mut Vec<FileEntry>, max: usize) {
    if results.len() >= max { return; }
    let Ok(read_dir) = std::fs::read_dir(dir) else { return };
    for entry in read_dir.flatten() {
        if results.len() >= max { break; }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        let is_match = name.to_lowercase().contains(pattern);
        let entry_path = entry.path();
        let Ok(symlink_meta) = entry_path.symlink_metadata() else { continue };
        let is_symlink = symlink_meta.file_type().is_symlink();
        let meta = entry.metadata().unwrap_or(symlink_meta);
        if is_match {
            let extension = Path::new(&name)
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let modified = meta.modified().ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            results.push(FileEntry {
                path: entry_path.to_string_lossy().to_string(),
                is_dir: meta.is_dir(),
                is_symlink,
                size: if meta.is_dir() { 0 } else { meta.len() },
                modified,
                extension,
                is_hidden: false,
                name,
            });
        }
        if recursive && meta.is_dir() {
            search_recursive(&entry_path, pattern, true, results, max);
        }
    }
}

/// Return directory entries whose names start with the final component of `partial`.
/// Appends `/` to directory completions. Returns at most 20 entries.
#[command]
pub fn complete_path(partial: String) -> Vec<String> {
    let expanded = if partial.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_default();
        partial.replacen('~', &home, 1)
    } else {
        partial.clone()
    };

    let (dir, prefix): (String, String) = if expanded.ends_with('/') {
        (expanded, String::new())
    } else {
        let p = Path::new(&expanded);
        let d = p.parent()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        let f = p.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        (d, f)
    };

    let Ok(rd) = std::fs::read_dir(&dir) else { return vec![] };
    let prefix_lower = prefix.to_lowercase();
    let mut completions: Vec<String> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { return None; }
            if !name.to_lowercase().starts_with(&prefix_lower) { return None; }
            let path = e.path().to_string_lossy().to_string();
            let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
            Some(if is_dir { format!("{}/", path) } else { path })
        })
        .collect();
    completions.sort();
    completions.truncate(20);
    completions
}

/// List running processes sorted by CPU usage descending.
#[command]
pub fn list_processes(state: tauri::State<'_, SystemState>) -> CmdResult<Vec<ProcessInfo>> {
    let mut sys = state.0.lock().map_err(|e| CmdError::Other(e.to_string()))?;
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut procs: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().to_string(),
            cpu: p.cpu_usage(),
            memory_kb: p.memory() / 1024,
            status: format!("{:?}", p.status()),
        })
        .collect();

    procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    Ok(procs)
}

/// Send SIGKILL to a process by PID.
#[command]
pub fn kill_process(pid: u32, state: tauri::State<'_, SystemState>) -> CmdResult<()> {
    let sys = state.0.lock().map_err(|e| CmdError::Other(e.to_string()))?;
    let sysinfo_pid = Pid::from(pid as usize);
    if let Some(process) = sys.process(sysinfo_pid) {
        process.kill();
        Ok(())
    } else {
        Err(CmdError::Other(format!("process {} not found", pid)))
    }
}

/// List the contents of an archive (zip / tar.gz / tar.bz2 / tar.xz / tar)
/// at a virtual directory path inside the archive.
/// inner_path = "" means the root of the archive.
/// Returns entries sorted dirs-first; always prepends ".." (with path = parent inner_path).
#[command]
pub fn list_archive_dir(archive_path: String, inner_path: String) -> CmdResult<Vec<FileEntry>> {
    let path = Path::new(&archive_path);
    let raw = read_archive_flat(path)?;
    let mut entries = filter_archive_entries(&raw, &inner_path);
    // prepend ".."
    let parent = if inner_path.is_empty() {
        String::new()
    } else if let Some(idx) = inner_path.rfind('/') {
        inner_path[..idx].to_string()
    } else {
        String::new()
    };
    entries.insert(0, FileEntry {
        name: "..".to_string(),
        path: parent,
        is_dir: true,
        is_symlink: false,
        size: 0,
        modified: 0,
        extension: String::new(),
        is_hidden: false,
    });
    Ok(entries)
}

/// Read all entries from an archive as flat (inner_path, is_dir, size, modified) tuples.
fn read_archive_flat(path: &Path) -> CmdResult<Vec<(String, bool, u64, u64)>> {
    let p = path.to_string_lossy();
    if p.ends_with(".tar.gz") || p.ends_with(".tgz") {
        read_tar_flat(path, ArchiveCompression::Gz)
    } else if p.ends_with(".tar.bz2") || p.ends_with(".tbz2") {
        read_tar_flat(path, ArchiveCompression::Bz2)
    } else if p.ends_with(".tar.xz") || p.ends_with(".txz") {
        read_tar_flat(path, ArchiveCompression::Xz)
    } else if p.ends_with(".tar") {
        read_tar_flat(path, ArchiveCompression::None)
    } else {
        // Assume ZIP (covers .zip, .jar, .war, .apk, .docx, .xlsx …)
        read_zip_flat(path)
    }
}

enum ArchiveCompression { None, Gz, Bz2, Xz }

fn read_zip_flat(path: &Path) -> CmdResult<Vec<(String, bool, u64, u64)>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| CmdError::Other(e.to_string()))?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| CmdError::Other(e.to_string()))?;
        let name = entry.name().trim_end_matches('/').to_string();
        if name.is_empty() { continue; }
        let is_dir = entry.is_dir();
        let size = entry.size();
        let modified = entry.last_modified()
            .map(|dt| {
                // rough unix timestamp from zip DateTime
                let y = dt.year() as i64 - 1970;
                (y * 31_536_000 + dt.month() as i64 * 2_592_000
                 + dt.day() as i64 * 86_400
                 + dt.hour() as i64 * 3_600
                 + dt.minute() as i64 * 60
                 + dt.second() as i64).max(0) as u64
            })
            .unwrap_or(0);
        entries.push((name, is_dir, size, modified));
    }
    Ok(entries)
}

fn read_tar_flat(path: &Path, comp: ArchiveCompression) -> CmdResult<Vec<(String, bool, u64, u64)>> {
    use std::io::BufReader;
    let file = std::fs::File::open(path)?;
    let buf = BufReader::new(file);
    let mut entries = Vec::new();

    macro_rules! iter_tar {
        ($reader:expr) => {{
            let mut archive = tar::Archive::new($reader);
            for entry in archive.entries().map_err(|e| CmdError::Other(e.to_string()))? {
                let entry = entry.map_err(|e| CmdError::Other(e.to_string()))?;
                let p = entry.path().map_err(|e| CmdError::Other(e.to_string()))?;
                let name = p.to_string_lossy().trim_end_matches('/').to_string();
                if name.is_empty() { continue; }
                let is_dir = entry.header().entry_type().is_dir();
                let size = entry.header().size().unwrap_or(0);
                let modified = entry.header().mtime().unwrap_or(0);
                entries.push((name, is_dir, size, modified));
            }
        }};
    }

    match comp {
        ArchiveCompression::None => { iter_tar!(buf); }
        ArchiveCompression::Gz   => { iter_tar!(flate2::read::GzDecoder::new(buf)); }
        ArchiveCompression::Bz2  => { iter_tar!(bzip2::read::BzDecoder::new(buf)); }
        ArchiveCompression::Xz   => { iter_tar!(xz2::read::XzDecoder::new(buf)); }
    }
    Ok(entries)
}

/// Given the flat listing of an archive, return the entries visible at `inner_path`.
fn filter_archive_entries(
    all: &[(String, bool, u64, u64)],
    inner: &str,
) -> Vec<FileEntry> {
    let prefix = if inner.is_empty() {
        String::new()
    } else {
        format!("{}/", inner.trim_matches('/'))
    };

    let mut seen: std::collections::HashMap<String, FileEntry> = Default::default();

    for (arc_path, _arc_is_dir, size, modified) in all {
        let relative = if prefix.is_empty() {
            arc_path.trim_start_matches('/')
        } else if let Some(rest) = arc_path.strip_prefix(&prefix) {
            rest
        } else {
            continue;
        };
        if relative.is_empty() { continue; }

        let (first, rest) = if let Some(idx) = relative.find('/') {
            (&relative[..idx], &relative[idx + 1..])
        } else {
            (relative, "")
        };
        if first.is_empty() { continue; }

        let is_dir = !rest.is_empty() || arc_path.ends_with('/');
        let internal_path = if prefix.is_empty() {
            first.to_string()
        } else {
            format!("{}{}", prefix, first)
        };

        let e = seen.entry(first.to_string()).or_insert_with(|| {
            let extension = Path::new(first)
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            FileEntry {
                name: first.to_string(),
                path: internal_path,
                is_dir,
                is_symlink: false,
                size: if is_dir { 0 } else { *size },
                modified: *modified,
                extension,
                is_hidden: first.starts_with('.'),
            }
        });
        if is_dir { e.is_dir = true; }
    }

    let mut result: Vec<FileEntry> = seen.into_values().collect();
    result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    result
}

/// Returns true if path looks like a supported archive.
#[command]
pub fn is_archive(path: String) -> bool {
    let p = path.as_str();
    p.ends_with(".zip") || p.ends_with(".jar") || p.ends_with(".war")
        || p.ends_with(".apk") || p.ends_with(".ipa")
        || p.ends_with(".tar.gz") || p.ends_with(".tgz")
        || p.ends_with(".tar.bz2") || p.ends_with(".tbz2")
        || p.ends_with(".tar.xz") || p.ends_with(".txz")
        || p.ends_with(".tar")
        || p.ends_with(".docx") || p.ends_with(".xlsx")
        || p.ends_with(".odt") || p.ends_with(".ods")
}

/// Extract `inner_paths` from an archive into `dst_dir`.
/// If inner_paths is empty, extracts everything.
#[command]
pub fn extract_from_archive(
    archive_path: String,
    inner_paths: Vec<String>,
    dst_dir: String,
) -> CmdResult<()> {
    let path = Path::new(&archive_path);
    let dst = Path::new(&dst_dir);
    std::fs::create_dir_all(dst)?;

    let p = archive_path.as_str();
    if p.ends_with(".tar.gz") || p.ends_with(".tgz") {
        extract_tar(path, &inner_paths, dst, ArchiveCompression::Gz)
    } else if p.ends_with(".tar.bz2") || p.ends_with(".tbz2") {
        extract_tar(path, &inner_paths, dst, ArchiveCompression::Bz2)
    } else if p.ends_with(".tar.xz") || p.ends_with(".txz") {
        extract_tar(path, &inner_paths, dst, ArchiveCompression::Xz)
    } else if p.ends_with(".tar") {
        extract_tar(path, &inner_paths, dst, ArchiveCompression::None)
    } else {
        extract_zip(path, &inner_paths, dst)
    }
}

fn extract_zip(archive: &Path, filter: &[String], dst: &Path) -> CmdResult<()> {
    use std::io::Read;
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| CmdError::Other(e.to_string()))?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| CmdError::Other(e.to_string()))?;
        let name = entry.name().to_string();
        if !filter.is_empty() && !filter.iter().any(|f| name.starts_with(f.as_str())) {
            continue;
        }
        let out = dst.join(&name);
        if entry.is_dir() {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() { std::fs::create_dir_all(parent)?; }
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| CmdError::Other(e.to_string()))?;
            std::fs::write(&out, &buf)?;
        }
    }
    Ok(())
}

fn extract_tar(archive: &Path, filter: &[String], dst: &Path, comp: ArchiveCompression) -> CmdResult<()> {
    use std::io::BufReader;
    let file = std::fs::File::open(archive)?;
    let buf = BufReader::new(file);

    macro_rules! unpack_tar {
        ($reader:expr) => {{
            let mut arch = tar::Archive::new($reader);
            for entry in arch.entries().map_err(|e| CmdError::Other(e.to_string()))? {
                let mut entry = entry.map_err(|e| CmdError::Other(e.to_string()))?;
                let p = entry.path().map_err(|e| CmdError::Other(e.to_string()))?.to_string_lossy().to_string();
                if !filter.is_empty() && !filter.iter().any(|f| p.starts_with(f.as_str())) {
                    continue;
                }
                entry.unpack_in(dst).map_err(|e| CmdError::Other(e.to_string()))?;
            }
        }};
    }

    match comp {
        ArchiveCompression::None => { unpack_tar!(buf); }
        ArchiveCompression::Gz   => { unpack_tar!(flate2::read::GzDecoder::new(buf)); }
        ArchiveCompression::Bz2  => { unpack_tar!(bzip2::read::BzDecoder::new(buf)); }
        ArchiveCompression::Xz   => { unpack_tar!(xz2::read::XzDecoder::new(buf)); }
    }
    Ok(())
}

/// List all files in a directory tree (flat view). No ".." entry.
#[command]
pub fn list_dir_flat(root: String, max_results: usize) -> CmdResult<Vec<FileEntry>> {
    let limit = if max_results == 0 { 5000 } else { max_results };
    let mut results = Vec::new();
    flat_recursive(Path::new(&root), &mut results, limit);
    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

fn flat_recursive(dir: &Path, results: &mut Vec<FileEntry>, max: usize) {
    if results.len() >= max { return; }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        if results.len() >= max { break; }
        let entry_path = entry.path();
        let Ok(sym_meta) = entry_path.symlink_metadata() else { continue };
        let is_symlink = sym_meta.file_type().is_symlink();
        let meta = entry.metadata().unwrap_or(sym_meta);
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        let extension = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let modified = meta.modified().ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        results.push(FileEntry {
            path: entry_path.to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            is_symlink,
            size: if meta.is_dir() { 0 } else { meta.len() },
            modified,
            extension,
            is_hidden: false,
            name,
        });
        if meta.is_dir() {
            flat_recursive(&entry_path, results, max);
        }
    }
}

/// Execute a shell command in a working directory. Returns combined stdout+stderr.
#[command]
pub fn exec_shell(cmd: String, cwd: String) -> CmdResult<String> {
    let out = std::process::Command::new("zsh")
        .args(["-c", &cmd])
        .current_dir(&cwd)
        .output()
        .map_err(|e| CmdError::Other(format!("exec: {e}")))?;
    let mut result = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !stderr.is_empty() {
        result.push_str(&stderr);
    }
    Ok(result)
}

/// Batch rename: apply a list of (old_path, new_name) pairs.
#[command]
pub fn batch_rename(renames: Vec<(String, String)>) -> CmdResult<()> {
    for (src, new_name) in &renames {
        let src_path = Path::new(src);
        let parent = src_path
            .parent()
            .ok_or_else(|| CmdError::Other(format!("no parent for {src}")))?;
        let dst = parent.join(new_name);
        std::fs::rename(src_path, &dst)?;
    }
    Ok(())
}

/// Compare two directories and return sync info.
#[command]
pub fn sync_dirs_list(left: String, right: String) -> CmdResult<Vec<SyncEntry>> {
    use std::collections::HashMap;

    fn collect_names(dir: &str) -> HashMap<String, u64> {
        let Ok(rd) = std::fs::read_dir(dir) else { return HashMap::new() };
        rd.flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { return None; }
                let size = e.metadata().map(|m| if m.is_dir() { 0 } else { m.len() }).unwrap_or(0);
                Some((name, size))
            })
            .collect()
    }

    let left_map = collect_names(&left);
    let right_map = collect_names(&right);

    let mut all_names: std::collections::HashSet<String> = Default::default();
    all_names.extend(left_map.keys().cloned());
    all_names.extend(right_map.keys().cloned());

    let mut entries: Vec<SyncEntry> = all_names
        .into_iter()
        .map(|name| {
            let lp = format!("{}/{}", left, name);
            let rp = format!("{}/{}", right, name);
            let ls = left_map.get(&name).copied();
            let rs = right_map.get(&name).copied();
            let status = match (ls, rs) {
                (Some(_), None) => "left-only".to_string(),
                (None, Some(_)) => "right-only".to_string(),
                (Some(l), Some(r)) => if l == r { "same".to_string() } else { "different".to_string() },
                _ => "same".to_string(),
            };
            SyncEntry {
                name,
                left_path: if ls.is_some() { Some(lp) } else { None },
                right_path: if rs.is_some() { Some(rp) } else { None },
                left_size: ls,
                right_size: rs,
                status,
            }
        })
        .collect();

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

#[derive(Debug, Serialize, Clone)]
pub struct SyncEntry {
    pub name: String,
    pub left_path: Option<String>,
    pub right_path: Option<String>,
    pub left_size: Option<u64>,
    pub right_size: Option<u64>,
    pub status: String, // "left-only" | "right-only" | "different" | "same"
}

/// Get aggregate system memory + CPU stats.
#[command]
pub fn get_system_stats(state: tauri::State<'_, SystemState>) -> CmdResult<SystemStats> {
    let mut sys = state.0.lock().map_err(|e| CmdError::Other(e.to_string()))?;
    sys.refresh_memory();
    sys.refresh_cpu_usage();

    let cpus = sys.cpus();
    let cpu_usage = if cpus.is_empty() {
        0.0
    } else {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    };

    Ok(SystemStats {
        total_memory_kb: sys.total_memory() / 1024,
        used_memory_kb: sys.used_memory() / 1024,
        cpu_usage,
        process_count: sys.processes().len(),
    })
}
