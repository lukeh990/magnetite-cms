/*
 * util.rs
 * Copyright (c) 2024 Luke Harding
 * This code is licensed under a GNU GPL v3 license.
 * See the file "LICENSE" in the root of this project.
 */

pub mod println {
    use colored::Colorize;

    pub fn important<S: Into<String>>(msg: S) {
        println!("{}", msg.into().green().bold());
    }

    pub fn error<S: Into<String>>(msg: S) {
        eprintln!("{}", msg.into().red().bold());
    }

    pub fn warn<S: Into<String>>(msg: S) {
        eprintln!("{}", msg.into().yellow().bold());
    }

    pub fn info<S: Into<String>>(msg: S) {
        println!("{}", msg.into().italic());
    }
}
