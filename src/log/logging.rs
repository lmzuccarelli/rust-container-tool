// module logging

// logging convenience functions
pub fn log_info(msg: &str) {
    println!("\x1b[1;94m {} \x1b[0m {}","INFO: ",msg);
}

pub fn log_hi(msg: &str) {
    println!("\x1b[1;94m {} \x1b[0m \x1b[1;95m{} \x1b[0m","INFO: ",msg);
}

pub fn log_warn(msg: &str) {
    println!("\x1b[1;93m {} \x1b[0m {}","WARN: ",msg);
}

pub fn log_error(msg: &str) {
    println!("\x1b[1;91m {} \x1b[0m {}","ERROR:",msg);
}
