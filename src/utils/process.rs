use std::{ffi::CString, path::Path, sync::LazyLock};
use regex::Regex;
use libc::{open, close, setsid, dup2, O_RDWR, STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};

static FIELD_CODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("%[fFuUdDnNiIcCkvVm]").expect("Failed to compile field code regex")
});

fn detach_child() {
    unsafe {
        // Just setsid alone is enough to completely detach the child process
        if setsid() < 0 {
            error!(error = %std::io::Error::last_os_error(), "Failed to create a new session");
        }

        // But we should also reopen stdin, stdout, and stderr to /dev/null, we won't
        // be needing them
        close(STDIN_FILENO);
        close(STDOUT_FILENO);
        close(STDERR_FILENO);

        let devnull = CString::new("/dev/null").unwrap();
        let null_desc = open(devnull.as_ptr(), O_RDWR);
        if null_desc >= 0 {
            dup2(null_desc, STDIN_FILENO);
            dup2(null_desc, STDOUT_FILENO);
            dup2(null_desc, STDERR_FILENO);

            if null_desc > STDERR_FILENO {
                close(null_desc);
            }
        }
    }
}

pub fn is_command_available(command: &str) -> bool {
    let paths = ["/bin", "/usr/bin", "/usr/local/bin"];
    for path in paths {
        let full_path = Path::new(path).join(command);
        if full_path.exists() && full_path.is_file() {
            return true;
        }
    }

    false
}

pub fn kill_task_if_any(command: &str) {
    let _ = std::process::Command::new("pkill")
        .arg("-f")
        .arg(command)
        .output();
}

pub fn launch(input: &str) {
    // Remove field codes from argv (including those that are deprecated), we won't be needing them...
    let argv: Vec<String> = if let Some(args) = shlex::split(input) {
        args.iter()
            .map(|s| FIELD_CODE_REGEX.replace_all(s, "").to_string())
            .collect()
    } else {
        error!(input, "Failed to parse command");
        return;
    };

    let binding = glib::environ();
    let envp: Vec<&Path> = binding
        .iter()
        .map(Path::new)
        .collect();

    if !argv.is_empty() {
        let argv_paths: Vec<&Path> = argv.iter()
            .map(AsRef::as_ref)
            .filter(|path: &&Path| !path.as_os_str().is_empty())
            .collect();

        if let Err(err) = glib::spawn_async(
            None::<&str>,
            &argv_paths,
            &envp,
            glib::SpawnFlags::SEARCH_PATH_FROM_ENVP | glib::SpawnFlags::SEARCH_PATH,
            Some(Box::new(detach_child)),
        ) {
            error!(input, %err, "Failed to launch command");
        }
    } else {
        warn!("No command to execute");
    }
}