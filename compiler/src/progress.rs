use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CYAN: &str = "\x1b[96m";
const BLUE: &str = "\x1b[94m";
const GREEN: &str = "\x1b[92m";
const RED: &str = "\x1b[91m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[90m";

const FRAMES: [&str; 4] = ["◢", "◣", "◤", "◥"];

pub struct Progress {
    total: usize,
    completed: usize,
    running: Arc<AtomicBool>,
    animation: Option<JoinHandle<()>>,
}

impl Progress {
    pub fn new(total: usize) -> Self {
        println!();
        println!("{CYAN}    A A A   E R O X A{RESET}");
        println!("{BLUE}    STRICT UNIVERSAL COMPILER{RESET}");
        println!("{DIM}    CODE BEYOND LIMITS{RESET}");
        println!();

        println!("{CYAN}  [ SYSTEM INITIALIZED ]{RESET}");
        println!("{DIM}  COMPILATION SEQUENCE STARTED{RESET}");
        println!();

        Self {
            total,
            completed: 0,
            running: Arc::new(AtomicBool::new(false)),
            animation: None,
        }
    }

    pub fn start(&mut self, stage: &str) {
        self.stop_animation();

        self.running.store(true, Ordering::Relaxed);

        // Display the first frame immediately.
        print!(
            "\r\x1b[2K{CYAN}  {}{RESET} {BLUE}SCANNING{RESET} {}",
            FRAMES[0],
            stage
        );

        let _ = io::stdout().flush();

        let running = Arc::clone(&self.running);
        let stage = stage.to_string();

        self.animation = Some(thread::spawn(move || {
            let mut frame = 1;

            while running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(80));

                if !running.load(Ordering::Relaxed) {
                    break;
                }

                print!(
                    "\r\x1b[2K{CYAN}  {}{RESET} {BLUE}SCANNING{RESET} {}",
                    FRAMES[frame],
                    stage
                );

                let _ = io::stdout().flush();

                frame = (frame + 1) % FRAMES.len();
            }
        }));
    }

    fn stop_animation(&mut self) {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.animation.take() {
            let _ = handle.join();
        }

        print!("\r\x1b[2K");
        let _ = io::stdout().flush();
    }

    pub fn complete(&mut self, stage: &str) {
        self.stop_animation();

        self.completed += 1;

        let percentage = if self.total == 0 {
            100
        } else {
            (self.completed * 100) / self.total
        };

        let width = 30;

        let filled = if self.total == 0 {
            width
        } else {
            (self.completed * width) / self.total
        };

        let bar = format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(width - filled)
        );

        println!(
            "{GREEN}  [ COMPLETE ]{RESET} {CYAN}{}{RESET}",
            stage
        );

        println!(
            "  {BLUE}[{}]{RESET} {CYAN}▶{RESET} {}%",
            bar,
            percentage
        );

        if self.completed == self.total {
            println!();
            println!(
                "{GREEN}  >>> BUILD SUCCESSFUL <<<{RESET}"
            );
            println!(
                "{CYAN}  NATIVE EXECUTABLE READY{RESET}"
            );
            println!();
        }
    }

    pub fn failed(&mut self, stage: &str) {
        self.stop_animation();

        println!();
        println!(
            "{RED}  [ FAILED ]{RESET} {}",
            stage
        );
        println!(
            "{RED}  COMPILATION ABORTED{RESET}"
        );
    }

    pub fn cache_hit() {
        println!();
        println!("{GREEN}  [ CACHE HIT ]{RESET}");
        println!("{CYAN}  BUILD ARTIFACTS REUSED{RESET}");
        println!();
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.stop_animation();
    }
}