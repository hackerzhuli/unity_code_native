use std::path::Path;

use sysinfo::{Pid, ProcessExt, ProcessRefreshKind, System, SystemExt};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProcessInfo {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) project_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProcessType {
    Unity,
    HotReload,
}

pub(crate) struct ProcessMonitor {
    pub(crate) system: System,
    pub(crate) target_project_path: String,
    pub(crate) unity_pid: Option<Pid>,
    pub(crate) hotreload_pid: Option<Pid>,
}

impl ProcessMonitor {
    pub(crate) fn new(target_project_path: String) -> Self {
        Self {
            system: System::new(),
            target_project_path,
            unity_pid: None,
            hotreload_pid: None,
        }
    }

    /**
     *  refresh the processes we detected
     *
     * @param is_full whether we need to refresh everything, or just Unity
     */
    pub(crate) fn update(&mut self, is_full: bool) -> () {
        self.update_unity_process();
        self.update_hotreload_process();

        // if everything is still running, no need to continue
        if self.unity_pid().is_some() && self.hotreload_pid().is_some() {
            return;
        }

        // if we only need to update unity, and unity is running, we don't need to continue
        if !is_full && self.unity_pid().is_some() {
            return;
        }

        // Slow path: full system scan (when we don't have cached PIDs or they became invalid)
        self.system
            .refresh_processes_specifics(ProcessRefreshKind::new()); // we need nothing, no cpu, no memory, just basics
        let normalized_project_path = normalize_path(&self.target_project_path.as_str());

        // First pass: Check for Unity processes
        let mut found_unity_pid = None::<Pid>;
        for (pid, process) in self.system.processes() {
            let process_name = process.name();

            if process_name.to_lowercase() == "unity.exe" || process_name.to_lowercase() == "unity"
            {
                if self.is_valid_unity_process(process) {
                    if let Some(path) = extract_unity_project_path(process) {
                        let normalized_path = normalize_path(path.as_str());
                        if normalized_project_path == normalized_path {
                            found_unity_pid = Some(*pid);
                            break;
                        }
                    }
                }
            }
        }

        self.set_unity_pid(found_unity_pid);

        if !is_full {
            let mut found_hotreload_pid = None::<Pid>;

            // Second pass: Only check for CodePatcherCLI processes if Unity exists for the target project
            if self.unity_pid().is_some() {
                for (pid, process) in self.system.processes() {
                    if self.is_valid_hotreload_process(process) {
                        if let Some(path) = extract_hotreload_project_path(process) {
                            let normalized_path = normalize_path(path.as_str());
                            if normalized_project_path == normalized_path {
                                found_hotreload_pid = Some(*pid);
                                break;
                            }
                        }
                    }
                }
            }

            self.set_hotreload_pid(found_hotreload_pid);
        }
    }

    /**
     * if unity process exists, check whether if it is still valid and if not, we invalidate the cache
     */
    pub(crate) fn update_unity_process(&mut self) {
        if self.unity_pid().is_none() {
            return;
        }

        let unity_pid = self.unity_pid().unwrap();

        self.system.refresh_process(unity_pid);

        if self.system.process(unity_pid).is_none() {
            self.set_unity_pid(None); // Process no longer exists, invalidate cache
        }
    }

    /**
     * if hotreload process exists, check whether if it is still valid and if not, we invalidate the cache
     */
    pub(crate) fn update_hotreload_process(&mut self) {
        if self.hotreload_pid().is_none() {
            return;
        }

        let hotreload_pid = self.hotreload_pid().unwrap();

        self.system.refresh_process(hotreload_pid);

        // Check if HotReload process still exists
        if self.system.process(hotreload_pid).is_none() {
            self.set_hotreload_pid(None); // Process no longer exists, invalidate cache
        }
    }

    pub(crate) fn is_valid_unity_process(&self, process: &sysinfo::Process) -> bool {
        if process.name() != get_unity_name() {
            return false;
        }
        // Check if this Unity process has a Unity parent (ignore child Unity processes)
        if let Some(parent_pid) = process.parent() {
            if let Some(parent_process) = self.system.process(parent_pid) {
                return parent_process.name() != process.name();
            }
        }
        true
    }

    pub(crate) fn is_valid_hotreload_process(&self, process: &sysinfo::Process) -> bool {
        process.name() == get_hot_reload_name()
    }

    pub(crate) fn unity_pid(&self) -> Option<Pid> {
        self.unity_pid
    }

    pub(crate) fn set_hotreload_pid(&mut self, hotreload_pid: Option<Pid>) {
        if self.hotreload_pid == hotreload_pid {
            return;
        }
        if hotreload_pid.is_none() {
            println!("HotReload process is closed");
        } else {
            println!("HotReload process detected, id = {:?}", hotreload_pid);
        }
        self.hotreload_pid = hotreload_pid;
    }

    pub(crate) fn hotreload_pid(&self) -> Option<Pid> {
        self.hotreload_pid
    }

    pub(crate) fn set_unity_pid(&mut self, unity_pid: Option<Pid>) {
        if self.unity_pid == unity_pid {
            return;
        }
        if unity_pid.is_none() {
            println!("Unity process is closed");
        } else {
            println!("Unity process detected, id = {:?}", unity_pid);
        }
        self.unity_pid = unity_pid;
    }
}

pub(crate) fn get_unity_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Unity.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "Unity"
    }
}

pub(crate) fn get_hot_reload_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "CodePatcherCLI.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "CodePatcherCLI"
    }
}

pub(crate) fn normalize_path(path: &str) -> String {
    let normalized = Path::new(path)
        .to_string_lossy()
        .to_lowercase()
        .replace('\\', "/");
    // Remove duplicate slashes
    let mut result = String::new();
    let mut prev_char = ' ';
    for ch in normalized.chars() {
        if ch == '/' && prev_char == '/' {
            continue; // Skip duplicate slash
        }
        result.push(ch);
        prev_char = ch;
    }
    result
}

pub(crate) fn extract_unity_project_path(process: &sysinfo::Process) -> Option<String> {
    let cmd_args = process.cmd();

    for i in 0..cmd_args.len() {
        let arg = &cmd_args[i];

        // Check for -projectpath, -createproject, -createProject (case insensitive)
        if arg.to_lowercase() == "-projectpath" || arg.to_lowercase() == "-createproject" {
            if i + 1 < cmd_args.len() {
                let path = &cmd_args[i + 1];
                return Some(path.trim_matches('"').to_string());
            }
        }
    }

    None
}

pub(crate) fn extract_hotreload_project_path(process: &sysinfo::Process) -> Option<String> {
    let cmd_args = process.cmd();

    for i in 0..cmd_args.len() {
        let arg = &cmd_args[i];

        // Check for -u option
        if arg == "-u" {
            if i + 1 < cmd_args.len() {
                let path = &cmd_args[i + 1];
                return Some(path.trim_matches('"').to_string());
            }
        }
    }

    None
}
