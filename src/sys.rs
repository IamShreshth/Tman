use std::path::Path;

#[cfg(unix)]
pub fn cpu_load() -> String {
    let mut load = [0.0f64; 3];
    let ret = unsafe { libc::getloadavg(load.as_mut_ptr(), 3) };
    if ret == 3 {
        format!("{:.2}  {:.2}  {:.2}", load[0], load[1], load[2])
    } else {
        #[cfg(target_os = "linux")]
        {
            if let Ok(s) = std::fs::read_to_string("/proc/loadavg") {
                let parts: Vec<&str> = s.split_whitespace().take(3).collect();
                if parts.len() == 3 {
                    return parts.join("  ");
                }
            }
        }
        "N/A".into()
    }
}

#[cfg(not(unix))]
pub fn cpu_load() -> String {
    "N/A".into()
}

#[cfg(unix)]
pub fn disk_space(path: &Path) -> Option<(String, String)> {
    use std::ffi::CString;
    let c_path = CString::new(path.to_string_lossy().as_bytes()).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } == 0 {
        let block_size = if stat.f_frsize != 0 {
            stat.f_frsize as u64
        } else {
            stat.f_bsize as u64
        };
        let free = (stat.f_bavail as u64) * block_size;
        let total = (stat.f_blocks as u64) * block_size;
        let used = total.saturating_sub(free);
        let pct = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
        Some((format_bytes(free), format!("{pct:.0}%")))
    } else {
        None
    }
}

#[cfg(not(unix))]
pub fn disk_space(_path: &Path) -> Option<(String, String)> {
    None
}

#[cfg(target_os = "macos")]
pub fn uptime() -> String {
    use std::ffi::CString;
    let mut boottime: libc::timeval = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::timeval>();
    let name = CString::new("kern.boottime").unwrap();
    let res = unsafe { libc::sysctlbyname(name.as_ptr(), &mut boottime as *mut _ as *mut libc::c_void, &mut len, std::ptr::null_mut(), 0) };
    if res == 0 {
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let secs = (now - boottime.tv_sec).max(0);
        format_duration(secs as u64)
    } else {
        "N/A".into()
    }
}

#[cfg(not(target_os = "macos"))]
pub fn uptime() -> String {
    if let Ok(s) = std::fs::read_to_string("/proc/uptime") {
        if let Some(sec_str) = s.split_whitespace().next() {
            if let Ok(sec) = sec_str.parse::<f64>() {
                return format_duration(sec as u64);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut info: libc::sysinfo = unsafe { std::mem::zeroed() };
        if unsafe { libc::sysinfo(&mut info) } == 0 && info.uptime > 0 {
            return format_duration(info.uptime as u64);
        }
    }
    "N/A".into()
}

#[cfg(target_os = "macos")]
pub fn battery() -> Option<String> {
    let out = std::process::Command::new("pmset").arg("-g").arg("batt").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains('%') {
            let parts: Vec<&str> = line.split('\t').collect();
            if let Some(second) = parts.get(1) {
                let sub: Vec<&str> = second.split(';').map(|s| s.trim()).collect();
                if let (Some(pct), Some(status)) = (sub.get(0), sub.get(1)) {
                    return Some(format!("{pct} ({status})"));
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn battery() -> Option<String> {
    let power_supply = Path::new("/sys/class/power_supply");
    if let Ok(entries) = std::fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_bat = entry.file_name().to_string_lossy().starts_with("BAT")
                || std::fs::read_to_string(path.join("type"))
                    .map(|t| t.trim().eq_ignore_ascii_case("battery"))
                    .unwrap_or(false);

            if is_bat {
                let capacity = std::fs::read_to_string(path.join("capacity"))
                    .ok()
                    .and_then(|c| c.trim().parse::<u32>().ok())
                    .or_else(|| {
                        let now = std::fs::read_to_string(path.join("energy_now"))
                            .or_else(|_| std::fs::read_to_string(path.join("charge_now")))
                            .ok()?
                            .trim()
                            .parse::<f64>()
                            .ok()?;
                        let full = std::fs::read_to_string(path.join("energy_full"))
                            .or_else(|_| std::fs::read_to_string(path.join("charge_full")))
                            .ok()?
                            .trim()
                            .parse::<f64>()
                            .ok()?;
                        if full > 0.0 {
                            Some(((now / full) * 100.0).round() as u32)
                        } else {
                            None
                        }
                    });

                let status = std::fs::read_to_string(path.join("status"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "Unknown".to_string());

                if let Some(cap) = capacity {
                    return Some(format!("{cap}% ({status})"));
                }
            }
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn battery() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub fn memory() -> Option<String> {
    use std::ffi::CString;
    let mut memsize: u64 = 0;
    let mut len = std::mem::size_of::<u64>();
    let name = CString::new("hw.memsize").unwrap();
    let res = unsafe { libc::sysctlbyname(name.as_ptr(), &mut memsize as *mut _ as *mut libc::c_void, &mut len, std::ptr::null_mut(), 0) };
    if res == 0 && memsize > 0 {
        Some(format_bytes(memsize))
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
pub fn memory() -> Option<String> {
    if let Ok(s) = std::fs::read_to_string("/proc/meminfo") {
        for line in s.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(kb_str) = parts.get(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return Some(format_bytes(kb * 1024));
                    }
                }
            }
        }
    }
    let mut info: libc::sysinfo = unsafe { std::mem::zeroed() };
    if unsafe { libc::sysinfo(&mut info) } == 0 {
        let total = (info.totalram as u64) * (info.mem_unit as u64);
        if total > 0 {
            return Some(format_bytes(total));
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn memory() -> Option<String> {
    None
}

fn format_bytes(b: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    if b >= GB {
        format!("{:.1} GB", b as f64 / GB as f64)
    } else if b >= MB {
        format!("{:.0} MB", b as f64 / MB as f64)
    } else {
        format!("{b} B")
    }
}

fn format_duration(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}
