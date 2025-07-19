use std::path::Path;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use log::info;

pub(crate) struct ProcessMonitor {
    pub(crate) system: System,
    pub(crate) target_project_path: String,
    pub(crate) unity_pid: Option<Pid>,
    pub(crate) hot_reload_pid: Option<Pid>,
}

impl ProcessMonitor {
    pub(crate) fn new(target_project_path: String) -> Self {
        Self {
            system: System::new(),
            target_project_path,
            unity_pid: None,
            hot_reload_pid: None,
        }
    }

    /**
     *  refresh the processes we detected
     *
     * @param is_full whether we need to refresh everything, or just Unity
     */
    pub(crate) fn update(&mut self, is_full: bool) -> () {
        self.update_unity_process();
        self.update_hot_reload_process();

        // if everything is still running, no need to continue
        if self.unity_pid().is_some() && self.hot_reload_pid().is_some() {
            return;
        }

        // if we only need to update unity, and unity is running, we don't need to continue
        if !is_full && self.unity_pid().is_some() {
            return;
        }

        // Slow path: full system scan (when we don't have cached PIDs or they became invalid)
        self.system
            .refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh_kind());
        let normalized_project_path = normalize_path(&self.target_project_path.as_str());

        if self.unity_pid().is_none() {
            self.detect_unity_process(&normalized_project_path);
        }

        if is_full && self.hot_reload_pid().is_none() {
            self.detect_hot_reload_process(normalized_project_path);
        }
    }

    fn detect_hot_reload_process(&mut self, normalized_project_path: String) {
        let mut found_hot_reload_pid = None::<Pid>;

        // Second pass: Only check for CodePatcherCLI processes if Unity exists for the target project
        if self.unity_pid().is_some() {
            for (pid, process) in self.system.processes() {
                if self.is_valid_hot_reload_process(process) {
                    if let Some(path) = extract_hot_reload_project_path(process) {
                        let normalized_path = normalize_path(path.as_str());
                        if normalized_project_path == normalized_path {
                            found_hot_reload_pid = Some(*pid);
                            break;
                        }
                    }
                }
            }
        }

        self.set_hot_reload_pid(found_hot_reload_pid);
    }

    fn detect_unity_process(&mut self, normalized_project_path: &String) {
        let mut found_unity_pid = None::<Pid>;
        for (pid, process) in self.system.processes() {
            if self.is_valid_unity_process(process) {
                if let Some(path) = extract_unity_project_path(process) {
                    let normalized_path = normalize_path(path.as_str());
                    if *normalized_project_path == normalized_path {
                        found_unity_pid = Some(*pid);
                        break;
                    }
                }
            }
        }

        self.set_unity_pid(found_unity_pid);
    }

    /**
     * if unity process exists, check whether if it is still valid and if not, we invalidate the cache
     */
    pub(crate) fn update_unity_process(&mut self) {
        if self.unity_pid().is_none() {
            return;
        }

        let unity_pid = self.unity_pid().unwrap();

        self
            .system
            .refresh_processes_specifics(ProcessesToUpdate::Some(&[unity_pid]), true, process_refresh_kind());
        if self.system.process(unity_pid).is_none() {
            self.set_unity_pid(None);
        }
    }

    /**
     * if hot_reload process exists, check whether if it is still valid and if not, we invalidate the cache
     */
    pub(crate) fn update_hot_reload_process(&mut self) {
        if self.hot_reload_pid().is_none() {
            return;
        }

        let hot_reload_pid = self.hot_reload_pid().unwrap();

        self
            .system
            .refresh_processes_specifics(ProcessesToUpdate::Some(&[hot_reload_pid]), true, process_refresh_kind());
        if self.system.process(hot_reload_pid).is_none() {
            self.set_hot_reload_pid(None);
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

    pub(crate) fn is_valid_hot_reload_process(&self, process: &sysinfo::Process) -> bool {
        process.name() == get_hot_reload_name()
    }

    pub(crate) fn unity_pid(&self) -> Option<Pid> {
        self.unity_pid
    }

    pub(crate) fn set_hot_reload_pid(&mut self, hot_reload_pid: Option<Pid>) {
        if self.hot_reload_pid == hot_reload_pid {
            return;
        }
        if hot_reload_pid.is_none() {
            info!("hot_reload process is closed");
        } else {
            info!("hot_reload process detected, id = {:?}", hot_reload_pid);
        }
        self.hot_reload_pid = hot_reload_pid;
    }

    pub(crate) fn hot_reload_pid(&self) -> Option<Pid> {
        self.hot_reload_pid
    }

    pub(crate) fn set_unity_pid(&mut self, unity_pid: Option<Pid>) {
        if self.unity_pid == unity_pid {
            return;
        }
        if unity_pid.is_none() {
            info!("Unity process is closed");
        } else {
            info!("Unity process detected, id = {:?}", unity_pid);
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
    // Use canonicalize for robust path normalization
    match std::fs::canonicalize(path) {
        Ok(canonical_path) => {
            canonical_path.to_string_lossy().to_string()
        }
        Err(_) => {
            // Fallback to original logic if canonicalize fails
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
    }
}

pub(crate) fn extract_unity_project_path(process: &sysinfo::Process) -> Option<String> {
    let cmd_args = process.cmd();

    for i in 0..cmd_args.len() {
        let arg = &cmd_args[i];

        // Check for -projectpath, -createproject, -createProject (case insensitive)
        if let Some(arg_str) = arg.to_str() {
            if arg_str.to_lowercase() == "-projectpath" || arg_str.to_lowercase() == "-createproject" {
                if i + 1 < cmd_args.len() {
                    if let Some(path_str) = cmd_args[i + 1].to_str() {
                        return Some(path_str.trim_matches('"').to_string());
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn extract_hot_reload_project_path(process: &sysinfo::Process) -> Option<String> {
    let cmd_args = process.cmd();

    for i in 0..cmd_args.len() {
        let arg = &cmd_args[i];

        // Check for -u option
        if let Some(arg_str) = arg.to_str() {
            if arg_str == "-u" {
                if i + 1 < cmd_args.len() {
                    if let Some(path_str) = cmd_args[i + 1].to_str() {
                        return Some(path_str.trim_matches('"').to_string());
                    }
                }
            }
        }
    }

    None
}

///  create process refresh kind
fn process_refresh_kind() -> ProcessRefreshKind {
    // we need nothing, no cpu, no memory, just basics
    ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet).with_cmd(UpdateKind::OnlyIfNotSet)
}