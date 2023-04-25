// module logging

// logging convenience functions
pub fn log_info(msg: &str) {
    println!("\x1b[1;94m {} \x1b[0m  : {}", "INFO", msg);
}

pub fn log_debug(msg: &str) {
    println!("\x1b[1;92m {} \x1b[0m : {}", "DEBUG", msg);
}

pub fn log_hi(msg: &str) {
    println!("\x1b[1;94m {}  \x1b[0m : \x1b[1;95m{} \x1b[0m", "INFO", msg);
}

pub fn log_mid(msg: &str) {
    println!("\x1b[1;94m {}  \x1b[0m : \x1b[1;96m{} \x1b[0m", "INFO", msg);
}

pub fn log_lo(msg: &str) {
    println!("\x1b[1;94m {}  \x1b[0m : \x1b[1;93m{} \x1b[0m", "INFO", msg);
}

pub fn log_ex(msg: &str) {
    println!("\x1b[1;94m {}  \x1b[0m : \x1b[1;98m{} \x1b[0m", "INFO", msg);
}

pub fn log_warn(msg: &str) {
    println!("\x1b[1;93m {} \x1b[0m  : {}", "WARN", msg);
}

pub fn log_error(msg: &str) {
    println!("\x1b[1;91m {} \x1b[0m : {}", "ERROR", msg);
}
