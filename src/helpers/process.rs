use std::{ffi::CString, path::Path};
use once_cell::sync::Lazy;
use regex::Regex;
use libc::{open, close, setsid, dup2, O_RDWR, STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};

static FIELD_CODE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("%[fFuUdDnNiIcCkvVm]").expect("Failed to compile field code regex")
});

fn detach_child() {
    unsafe {
        // Just setsid alone is enough to completely detach the child process
        if setsid() < 0 {
            eprintln!("Failed to create a new session: {}", std::io::Error::last_os_error());
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

pub fn launch(input: &str) {
    // Remove field codes from argv (including those that are deprecated), we won't be needing them...
    let argv: Vec<String> = if let Some(args) = shlex::split(input) {
        args.iter()
            .map(|s| FIELD_CODE_REGEX.replace_all(s, "").to_string())
            .collect()
    } else {
        eprintln!("Failed to parse command: {}", input);
        return;
    };

    let binding = gtk4::glib::environ();
    let envp: Vec<&Path> = binding
        .iter()
        .map(Path::new)
        .collect();

    if !argv.is_empty() {
        let argv_paths: Vec<&Path> = argv.iter().map(AsRef::as_ref).collect();

        if let Err(err) = gtk4::glib::spawn_async(
            None::<&str>,
            &argv_paths,
            &envp,
            gtk4::glib::SpawnFlags::SEARCH_PATH_FROM_ENVP | gtk4::glib::SpawnFlags::SEARCH_PATH,
            Some(Box::new(detach_child)),
        ) {
            eprintln!("Failed to launch command: {}: {}", input, err);
        }
    } else {
        eprintln!("No command to execute.");
    }
}