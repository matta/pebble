use crate::command::CommandExt;
use color_eyre::Result;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

pub trait GitProvider {
    fn run(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()>;
    fn run_quiet(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()> {
        self.run(args, current_dir)
    }
    fn output(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<String>;
    fn status(
        &self,
        args: &[&dyn AsRef<OsStr>],
        current_dir: &Path,
    ) -> Result<std::process::ExitStatus>;
}

pub struct RealGit;

impl GitProvider for RealGit {
    fn run(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()> {
        let mut cmd = Command::new("git");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.current_dir(current_dir).check_run().map_err(Into::into)
    }

    fn run_quiet(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()> {
        let mut cmd = Command::new("git");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.current_dir(current_dir)
            .stdout(std::process::Stdio::null())
            .check_run()
            .map_err(Into::into)
    }

    fn output(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<String> {
        let mut cmd = Command::new("git");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.current_dir(current_dir)
            .check_output()
            .map_err(Into::into)
    }

    fn status(
        &self,
        args: &[&dyn AsRef<OsStr>],
        current_dir: &Path,
    ) -> Result<std::process::ExitStatus> {
        let mut cmd = Command::new("git");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.current_dir(current_dir).status().map_err(Into::into)
    }
}
