use crate::command::CommandExt;
use color_eyre::Result;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

pub trait GitProvider {
    fn run(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()>;
    fn run_quiet(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()>;
    fn run_silent(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()>;
    fn output(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<String>;
    fn status(
        &self,
        args: &[&dyn AsRef<OsStr>],
        current_dir: &Path,
    ) -> Result<std::process::ExitStatus>;
    fn status_silent(
        &self,
        args: &[&dyn AsRef<OsStr>],
        current_dir: &Path,
    ) -> Result<std::process::ExitStatus>;
}

pub struct RealGit;

impl RealGit {
    fn command(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Command {
        let mut cmd = Command::new("git");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.current_dir(current_dir);
        cmd
    }
}

impl GitProvider for RealGit {
    fn run(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()> {
        self.command(args, current_dir)
            .check_run()
            .map_err(Into::into)
    }

    fn run_quiet(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()> {
        self.command(args, current_dir)
            .stdout(std::process::Stdio::null())
            .check_run()
            .map_err(Into::into)
    }

    fn run_silent(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<()> {
        self.command(args, current_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .check_run()
            .map_err(Into::into)
    }

    fn output(&self, args: &[&dyn AsRef<OsStr>], current_dir: &Path) -> Result<String> {
        self.command(args, current_dir)
            .check_output()
            .map_err(Into::into)
    }

    fn status(
        &self,
        args: &[&dyn AsRef<OsStr>],
        current_dir: &Path,
    ) -> Result<std::process::ExitStatus> {
        self.command(args, current_dir).status().map_err(Into::into)
    }

    fn status_silent(
        &self,
        args: &[&dyn AsRef<OsStr>],
        current_dir: &Path,
    ) -> Result<std::process::ExitStatus> {
        self.command(args, current_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(Into::into)
    }
}
