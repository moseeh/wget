#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum OutputLevel {
    Quiet = 0,
    Normal = 1,
    Verbose = 2,
    Debug = 3,
}

pub struct OutputLogger {
    level: OutputLevel,
}

impl OutputLogger {
    pub fn new(level: OutputLevel) -> Self {
        Self { level }
    }

    pub fn info(&self, msg: &str) {
        if self.level >= OutputLevel::Normal {
            println!("{}", msg);
        }
    }

    pub fn verbose(&self, msg: &str) {
        if self.level >= OutputLevel::Verbose {
            println!("{}", msg);
        }
    }

    pub fn debug(&self, msg: &str) {
        if self.level >= OutputLevel::Debug {
            println!("DEBUG: {}", msg);
        }
    }

    pub fn error(&self, msg: &str) {
        eprintln!("{}", msg);
    }
}