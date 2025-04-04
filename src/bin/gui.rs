use std::env;
use tirra::{gui_iced::app, storage::db, ui::command_line};

pub fn main() -> () {
    let mut is_gui = true;
    let mut db_to_use = db::default_user_db_path();
    // check arguments
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let first_arg = &args[1];
        if first_arg.len() == 5 && first_arg[0..5].eq("--cli") {
            is_gui = false;
        } else {
            db_to_use = String::from(first_arg);
        }
    }

    /////

    if is_gui {
        let _result: iced::Result = app::app_iced(db_to_use);
    } else {
        //cli
        let cli_args = &args[1..].to_vec();
        command_line::process(cli_args, cli_args.len());
    }
}
