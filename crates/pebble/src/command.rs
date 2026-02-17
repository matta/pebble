use std::io::{self, Error, ErrorKind};
use std::process::{Command, ExitStatus, Stdio};

// ============================================================================
// Layer 1: The "Polyfill"
// Adds `.exit_ok()` to ExitStatus (similar to unstable #![feature(exit_status_error)])
// ============================================================================

pub trait ExitStatusExt {
    /// Returns `Ok(())` if the exit code is 0.
    /// Returns `Err(io::Error)` otherwise.
    fn exit_ok(&self) -> io::Result<()>;

    /// A variant that allows attaching a custom error message context
    fn exit_ok_with_context(&self, context: &str) -> io::Result<()>;
}

impl ExitStatusExt for ExitStatus {
    fn exit_ok(&self) -> io::Result<()> {
        if self.success() {
            Ok(())
        } else {
            let msg = if let Some(code) = self.code() {
                format!("Process exited with non-zero status code: {}", code)
            } else {
                "Process terminated by signal".to_string()
            };
            Err(Error::other(msg))
        }
    }

    fn exit_ok_with_context(&self, context: &str) -> io::Result<()> {
        #[allow(unstable_name_collisions)]
        self.exit_ok()
            .map_err(|e| Error::other(format!("{}: {}", context, e)))
    }
}

// ============================================================================
// Layer 2: The "Python Subprocess" Wrapper
// Adds `.check_output()` and `.check_run()` directly to Command builders.
// ============================================================================

pub trait CommandExt {
    /// Analog to Python's `subprocess.check_output(..., text=True)`.
    /// Runs the command, captures stdout/stderr, checks for success.
    /// Returns stdout as a String on success.
    /// Returns an Error containing stderr info on failure.
    fn check_output_utf8(&mut self) -> io::Result<String>;

    /// Analog to Python's `subprocess.check_call(...)`.
    /// Runs the command, inheriting stdout/stderr (unless configured otherwise),
    /// and checks for success.
    fn check_run(&mut self) -> io::Result<()>;
}

impl CommandExt for Command {
    fn check_output_utf8(&mut self) -> io::Result<String> {
        // Ensure we capture output to process it
        self.stdout(Stdio::piped());
        self.stderr(Stdio::piped());

        let output = self.output()?;

        // Use our Layer 1 extension to check status
        #[allow(unstable_name_collisions)]
        if let Err(e) = output.status.exit_ok() {
            // Enhancement: Include stderr in the error message for debugging
            let stderr_preview = String::from_utf8_lossy(&output.stderr);
            let rich_error = format!("{}; stderr: {}", e, stderr_preview.trim());
            return Err(Error::other(rich_error));
        }

        // Decode stdout
        String::from_utf8(output.stdout).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    fn check_run(&mut self) -> io::Result<()> {
        // We use .status() so streams are inherited (printed to terminal)
        // unless the user manually overrode them on the Command struct previously.
        let status = self.status()?;
        #[allow(unstable_name_collisions)]
        status.exit_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_output_utf8_success() {
        let output = Command::new("echo")
            .arg("hello world")
            .check_output_utf8()
            .expect("Failed to run echo");
        assert_eq!(output.trim(), "hello world");
    }

    #[test]
    fn test_check_output_utf8_failure() {
        // Use a command that is guaranteed to fail and output to stderr
        let result = Command::new("git")
            .arg("this-is-an-invalid-command")
            .check_output_utf8();

        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("stderr:"));
        // Check for non-zero exit code message
        assert!(
            msg.contains("Process exited with non-zero status code")
                || msg.contains("Process terminated")
        );
    }

    #[test]
    fn test_check_run_success() {
        Command::new("true")
            .check_run()
            .expect("true command should succeed");
    }

    #[test]
    fn test_check_run_failure() {
        let result = Command::new("false").check_run();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string()
                .contains("Process exited with non-zero status code")
        );
    }
}
