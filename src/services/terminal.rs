use pty_process::blocking::{Command as PtyCommand, Pty};
use pty_process::Size;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Sessions = Arc<Mutex<HashMap<String, PtySession>>>;

pub struct PtySession {
    pub id: String,
    pub child: Child,
    pub pty_master: Pty,
}

impl PtySession {
    pub fn new(id: String, working_dir: PathBuf, command: &str, env: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let pty = Pty::new()?;
        let pts = pty.pts()?;
        let mut cmd = PtyCommand::new(command);
        cmd.current_dir(working_dir);
        cmd.env("TERM", "xterm-256color");

        for (key, value) in env {
            cmd.env(key, value);
        }

        let child = cmd.spawn(&pts)?;

        Ok(Self {
            id,
            child,
            pty_master: pty,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        self.pty_master.write_all(data)?;
        Ok(data.len())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self.pty_master.read(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(e),
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.pty_master.resize(Size::new(rows, cols))?;
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        // Try to get process status
        true // Simplified for now
    }
}
