use std::env;
use std::process;
use std::time::Instant;
use sysinfo::ProcessRefreshKind;
use sysinfo::RefreshKind;
use sysinfo::{System, SystemExt, ProcessExt, PidExt};
use csv::Writer;
use std::io;

fn main() {
    let total_start = Instant::now();
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <process_name>", args[0]);
        eprintln!("Example: {} Unity.exe", args[0]);
        process::exit(1);
    }
    
    let target_process_name = &args[1];
    
    // Start timing for system initialization
    let init_start = Instant::now();
    
    // Initialize system information
    let system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    
    let init_duration = init_start.elapsed();
    eprintln!("System initialization took: {:.2?}", init_duration);
    
    // Create CSV writer
    let mut wtr = Writer::from_writer(io::stdout());
    
    // Write CSV header
    if let Err(e) = wtr.write_record(&["Process Name", "Process ID", "Parent ID", "Command Line"]) {
        eprintln!("Error writing CSV header: {}", e);
        process::exit(1);
    }
    
    let mut found_processes = false;
    let mut count = 0;
    
    // Start timing for process enumeration and search
    let search_start = Instant::now();

    // Iterate through all processes
    for (pid, process) in system.processes() {
        let process_name = process.name();
        count += 1;
        
        // Check if process name matches (case-insensitive)
        if process_name == target_process_name {
            found_processes = true;
            
            let process_id = pid.as_u32().to_string();
            let parent_id = match process.parent() {
                Some(ppid) => ppid.as_u32().to_string(),
                None => "0".to_string(),
            };
            
            // Get command line arguments with proper quoting
            let cmd_args = process.cmd();
            let cmd_line = if cmd_args.is_empty() {
                process.exe().to_string_lossy().to_string()
            } else {
                cmd_args.iter()
                    .map(|arg| {
                        if arg.contains(' ') || arg.contains('\t') {
                            format!("\"{}\"", arg)
                        } else {
                            arg.to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            };
            
            // Write process information to CSV
            if let Err(e) = wtr.write_record(&[process_name, &process_id, &parent_id, &cmd_line]) {
                eprintln!("Error writing CSV record: {}", e);
                process::exit(1);
            }
        }
    }
    
    let search_duration = search_start.elapsed();
    eprintln!("Process enumeration and search took: {:.2?}", search_duration);
    eprintln!("Total processes scanned: {}", count);
    eprintln!("Average time per process: {:.2?}", search_duration / count as u32);
    
    // Flush the writer
    if let Err(e) = wtr.flush() {
        eprintln!("Error flushing CSV output: {}", e);
        process::exit(1);
    }
    
    let total_duration = total_start.elapsed();
    eprintln!("Total execution time: {:.2?}", total_duration);
    eprintln!("Performance Summary:");
    eprintln!("  - System init: {:.2?} ({:.1}%)", init_duration, (init_duration.as_nanos() as f64 / total_duration.as_nanos() as f64) * 100.0);
    eprintln!("  - Process search: {:.2?} ({:.1}%)", search_duration, (search_duration.as_nanos() as f64 / total_duration.as_nanos() as f64) * 100.0);

    //system.refresh_process_specifics(pid, refresh_kind)

    if !found_processes {
        eprintln!("No processes found with name: {}", target_process_name);
        process::exit(1);
    }
}
