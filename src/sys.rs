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
    parse_pmset(&String::from_utf8_lossy(&out.stdout))
}

#[cfg(any(target_os = "macos", test))]
fn parse_pmset(text: &str) -> Option<String> {
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
    let mut total: u64 = 0;
    let mut len = std::mem::size_of::<u64>();
    let name = CString::new("hw.memsize").unwrap();
    if unsafe { libc::sysctlbyname(name.as_ptr(), &mut total as *mut _ as *mut libc::c_void, &mut len, std::ptr::null_mut(), 0) } != 0 || total == 0 {
        return None;
    }
    if let Ok(out) = std::process::Command::new("vm_stat").output() {
        if let Some(s) = parse_vm_stat(&String::from_utf8_lossy(&out.stdout), total) {
            return Some(s);
        }
    }
    Some(format_bytes(total))
}

#[cfg(any(target_os = "macos", test))]
fn parse_vm_stat(text: &str, total: u64) -> Option<String> {
    let page_size: u64 = text.lines().next()
        .and_then(|l| l.split("page size of ").nth(1))
        .and_then(|s| s.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(4096);
    let mut used_pages = 0u64;
    for line in text.lines() {
        if line.starts_with("Pages active:") || line.starts_with("Pages wired down:") || line.starts_with("Pages occupied by compressor:") {
            if let Some(v) = line.split(':').nth(1).and_then(|s| s.trim().trim_end_matches('.').parse::<u64>().ok()) {
                used_pages += v;
            }
        }
    }
    if used_pages > 0 && total > 0 {
        let used = (used_pages * page_size).min(total);
        Some(format!("{} / {} ({:.0}%)", format_bytes(used), format_bytes(total), (used as f64 / total as f64) * 100.0))
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
pub fn memory() -> Option<String> {
    if let Ok(s) = std::fs::read_to_string("/proc/meminfo") {
        let (mut total_kb, mut avail_kb) = (None, None);
        for line in s.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok());
            } else if line.starts_with("MemAvailable:") {
                avail_kb = line.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok());
            }
        }
        if let Some(t) = total_kb {
            let total = t * 1024;
            if let Some(a) = avail_kb {
                let used = total.saturating_sub(a * 1024);
                return Some(format!("{} / {} ({:.0}%)", format_bytes(used), format_bytes(total), (used as f64 / total as f64) * 100.0));
            }
            return Some(format_bytes(total));
        }
    }
    let mut info: libc::sysinfo = unsafe { std::mem::zeroed() };
    if unsafe { libc::sysinfo(&mut info) } == 0 {
        let unit = info.mem_unit as u64;
        let total = (info.totalram as u64) * unit;
        if total > 0 {
            let used = total.saturating_sub((info.freeram as u64) * unit);
            return Some(format!("{} / {} ({:.0}%)", format_bytes(used), format_bytes(total), (used as f64 / total as f64) * 100.0));
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

pub struct PortInfo {
    pub port: String,
    pub pid: String,
    pub proc_name: String,
}

pub fn listening_ports() -> Vec<PortInfo> {
    use std::collections::HashSet;
    use std::process::Command;
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(o) = Command::new("lsof").args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"]).output() {
        if o.status.success() {
            return parse_lsof(&String::from_utf8_lossy(&o.stdout));
        }
    }

    if let Ok(o) = Command::new("ss").args(["-lptnH"]).output() {
        if o.status.success() {
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if cols.len() >= 5 {
                    let local = cols[3];
                    let port = local.rsplit(':').next().unwrap_or("").to_string();
                    let proc_col = cols.get(5).copied().unwrap_or("");
                    let proc_name = proc_col.split('"').nth(1).unwrap_or("process").to_string();
                    let pid = proc_col.split("pid=").nth(1).and_then(|s| s.split(',').next()).unwrap_or("").to_string();
                    if !port.is_empty() && !pid.is_empty() && seen.insert((port.clone(), pid.clone())) {
                        out.push(PortInfo { port, pid, proc_name });
                    }
                }
            }
        }
    }
    out
}

fn parse_lsof(text: &str) -> Vec<PortInfo> {
    use std::collections::HashSet;
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 9 {
            let proc_name = cols[0].to_string();
            let pid = cols[1].to_string();
            let addr = cols[cols.len() - 2];
            if let Some(port) = addr.rsplit(':').next() {
                if seen.insert((port.to_string(), pid.clone())) {
                    out.push(PortInfo { port: port.to_string(), pid, proc_name });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_vm_stat_and_pmset_and_lsof() {
        let vm = "Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free:                               12000.\nPages active:                            262144.\nPages inactive:                          100000.\nPages wired down:                        131072.\nPages occupied by compressor:            131072.\n";
        let total = 16 * 1024 * 1024 * 1024u64; // 16 GB
        assert_eq!(parse_vm_stat(vm, total).as_deref(), Some("8.0 GB / 16.0 GB (50%)"));

        let pm = "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=123)\t84%; discharging; 3:45 remaining present: true\n";
        assert_eq!(parse_pmset(pm).as_deref(), Some("84% (discharging)"));

        let lsof_out = "COMMAND   PID   USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME\nnode    41230   user   21u  IPv6 0x123456789      0t0  TCP *:3000 (LISTEN)\n";
        let ports = parse_lsof(lsof_out);
        assert_eq!(ports.len(), 1);
        assert_eq!((ports[0].port.as_str(), ports[0].pid.as_str(), ports[0].proc_name.as_str()), ("3000", "41230", "node"));
    }
}
