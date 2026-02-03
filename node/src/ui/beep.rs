//! IBM POST beep sound for TUI transitions.
//! Zero external dependencies - embedded WAV played via system player.

use std::io::Write;
use std::process::{Command, Stdio};

/// IBM POST beep: 896.46 Hz square wave, ~230ms, embedded at compile time.
const POST_BEEP_WAV: &[u8] = include_bytes!("../../assets/post_beep.wav");

/// Play the IBM POST beep sound (non-blocking, spawns a thread).
pub fn play_post_beep() {
    std::thread::spawn(|| {
        #[cfg(target_os = "macos")]
        {
            // afplay needs a file, use temp
            let temp_path = std::env::temp_dir().join("shinod_beep.wav");
            if let Ok(mut file) = std::fs::File::create(&temp_path) {
                if file.write_all(POST_BEEP_WAV).is_ok() {
                    let _ = Command::new("afplay").arg(&temp_path).status();
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // aplay can read from stdin
            if let Ok(mut child) = Command::new("aplay")
                .args(["-q", "-"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(POST_BEEP_WAV);
                }
                let _ = child.wait();
            }
        }
    });
}
