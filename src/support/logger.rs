use crate::get_line;
use chrono::{DateTime, Utc};
use regex::Regex;
use std::fmt::Display;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Logger {
    root: PathBuf,
}

impl Logger {
    pub fn new(root: PathBuf) -> Logger {
        Logger { root }
    }

    pub fn log<T: Display>(&self, content: T) {
        let file = self.resolve_file();

        self.write(file, content);
    }

    fn write<T: Display>(&self, mut file: File, content: T) {
        match file.lock() {
            Ok(_) => {
                file.write_all(
                    format!(
                        "[{}] {}: {}\n",
                        Utc::now().format("%Y-%m-%d %H:%M:%S"),
                        get_line!(),
                        content
                    )
                        .as_bytes(),
                )
                    .unwrap();
            }
            Err(error) => {
                eprintln!("Failed to log panic.");
                dbg!(error);
            }
        };
    }

    fn resolve_file(&self) -> File {
        // pattern: app.{timestamp}.log
        let pattern = Regex::new(r"\Aapp\.\d{10}.log\z").unwrap();

        let mut paths = fs::read_dir(self.root.join("storage/logs"))
            .unwrap_or_else(|_| {
                fs::create_dir(self.root.join("storage/logs")).expect("failed to create dir");
                fs::read_dir(self.root.join("storage/logs")).unwrap()
            })
            .map(|p| String::from(p.unwrap().file_name().to_str().unwrap()))
            .filter(|n| pattern.is_match(n))
            .collect::<Vec<String>>();

        let mut file;

        if paths.len() == 0 {
            // Create log file if none exist
            file = self.create_log_file().unwrap();
        } else {
            // Get latest file from paths
            paths.sort();
            let file_name = paths.iter().last().unwrap();
            file = OpenOptions::new()
                .append(true)
                .open(format!("storage/logs/{file_name}"))
                .unwrap();

            // Create new log file if latest one >= 25mb
            let metadata = file.metadata().unwrap();
            let mb = metadata.len() / 1024 / 1024;
            if mb >= 25 {
                file = self.create_log_file().unwrap();
            }

            if paths.len() >= 10 {
                // Delete earliest log file
                let file_name = paths.iter().nth(0).unwrap();
                fs::remove_file(format!("storage/logs/{file_name}"))
                    .expect(format!("Failed to remove file {file_name}.").as_str());
            }
        }
        file
    }

    fn create_log_file(&self) -> Result<File, std::io::Error> {
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.format("%s");

        Ok(OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("storage/logs/app.{timestamp}.log"))?)
    }
}
