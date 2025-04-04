use std::env;
use tirra::ui::command_line;

pub fn main() -> () {
    let args: Vec<String> = env::args().collect();
    let args_count = args.len();

    command_line::process(&args, args_count);
}
