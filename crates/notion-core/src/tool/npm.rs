use std::env::{args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::io;
use std::process::{Command, ExitStatus};

use super::{
    arg0, command_for, display_error, intercept_global_installs, NoGlobalInstallError,
    NoSuchToolError, Tool,
};
use session::{ActivityKind, Session};

use notion_fail::Fallible;

/// Represents a `npm` executable.
pub struct Npm(Command);

impl Tool for Npm {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Npm);

        let mut args = args_os();
        let exe = arg0(&mut args)?;

        if intercept_global_installs() && is_global_npm_install() {
            throw!(NoGlobalInstallError);
        }

        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;
            Ok(Self::from_components(&exe, args, &image.path()?))
        } else {
            // Using 'Node' as the tool name since the npm version is derived from the Node version
            // This way the error message will prompt the user to add 'Node' to their toolchain, instead of 'npm'
            throw!(NoSuchToolError {
                tool: "Node".to_string()
            });
        }
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Npm(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }

    fn finalize(session: &Session, maybe_status: &io::Result<ExitStatus>) {
        if let Ok(_) = maybe_status {
            if let Ok(Some(project)) = session.project() {
                let errors = project.autoshim();

                for error in errors {
                    display_error(&error);
                }
            }
        }
    }
}

fn is_global_npm_install() -> bool {
    let command = args_os()
        .skip(1)
        .skip_while(|arg| match arg.to_str() {
            Some(arg) => arg.starts_with("-"),
            None => false,
        })
        .next();

    // npm global installs will have the command `i`, `install`, `add` or `isntall`
    // See https://github.com/npm/cli/blob/latest/lib/config/cmd-list.js
    // Additionally, they will have `-g` or `--global` somewhere in the argument list
    if command == Some(OsString::from("install"))
        || command == Some(OsString::from("i"))
        || command == Some(OsString::from("isntall"))
        || command == Some(OsString::from("add"))
    {
        args_os().any(|arg| arg == OsString::from("-g") || arg == OsString::from("--global"))
    } else {
        false
    }
}
