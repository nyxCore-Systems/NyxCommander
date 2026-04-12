use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub struct PluginProcess {
    child: Child,
    stdin: ChildStdin,
    /// Pending responses keyed by request id.
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    next_id: u64,
}

impl PluginProcess {
    /// Spawn a subprocess and start its stdout reader thread.
    pub fn spawn(executable: &str) -> std::io::Result<Self> {
        let mut child = std::process::Command::new(executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let pending_clone = Arc::clone(&pending);
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                let Ok(val) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                if let Some(id) = val.get("id").and_then(|v| v.as_u64()) {
                    if let Ok(mut map) = pending_clone.lock() {
                        if let Some(tx) = map.remove(&id) {
                            let _ = tx.send(val);
                        }
                    }
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            pending,
            next_id: 1,
        })
    }

    /// Send a JSON-RPC request and await a response (5 second timeout).
    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        let req = serde_json::json!({ "id": id, "method": method, "params": params });
        let line = format!("{req}\n");
        self.stdin
            .write_all(line.as_bytes())
            .map_err(|e| e.to_string())?;
        self.stdin.flush().map_err(|e| e.to_string())?;

        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);

        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .map_err(|_| "plugin timed out".to_string())?
            .map_err(|_| "plugin channel closed".to_string())
    }

    /// Returns true if the subprocess is still running.
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for PluginProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
