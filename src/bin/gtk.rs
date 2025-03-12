use std::{cell::RefCell, rc::Rc};

use crate::glib::clone;
use gtk::{
    glib::{self},
    prelude::*,
};
use tirra::ui::ui::{self, TirraInterface, WriterUi};

fn main() -> glib::ExitCode {
    let application = gtk::Application::builder()
        .application_id("com.tirra.gtk")
        .build();
    application.connect_activate(build_ui);
    application.run_with_args(&["dd"])
}

fn build_ui(application: &gtk::Application) {
    let db_to_use = ui::parse_args();

    //let login_ui = ui::LoginUi::new(&db_to_use);
    let login_ui = Rc::new(RefCell::new(ui::LoginUi::new(&db_to_use)));
    let title = login_ui.borrow().title();
    let window = gtk::ApplicationWindow::new(application);
    window.set_default_size(800, 800);
    window.set_title(Some(&title));

    let login_page_box = build_interface(db_to_use, login_ui);
    window.set_child(Some(&login_page_box));
    window.present();
}

fn build_interface(db_to_use: String, login: Rc<RefCell<ui::LoginUi>>) -> gtk::Box {
    let writer_ui = Rc::new(RefCell::new(WriterUi::new()));
    // Login Box
    let container_app = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    container_app.set_homogeneous(false);
    container_app.set_margin_start(100);
    container_app.set_margin_end(100);

    let container_login = gtk::Box::new(gtk::Orientation::Vertical, 6);
    container_login.set_margin_top(60);
    container_login.set_homogeneous(false);
    container_app.append(&container_login);

    //label : database path
    let label_db = gtk::Label::builder().label(&db_to_use).build();
    container_login.append(&label_db);

    // Password Entry
    let password_entry = gtk::PasswordEntry::builder().width_chars(54).build();
    password_entry.set_placeholder_text(Some(ui::LoginUi::UI_LOGIN_PWDINPUT_PLACEHOLDER));
    container_login.append(&password_entry);
    // Decrypt Button
    let decrypt_button = gtk::Button::with_label(ui::LoginUi::UI_LOGIN_BUTTON_TEXT_DECRYPT);
    container_login.append(&decrypt_button);
    //label info
    let label_info = gtk::Label::builder().label("").build();
    container_login.append(&label_info);

    // Editor Container
    let container_editor = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    container_editor.set_margin_top(60);
    container_editor.set_homogeneous(false);
    container_editor.set_visible(false);
    container_app.append(&container_editor);

    // Entries List
    let list_box = gtk::ListBox::new();
    container_editor.append(&list_box);

    //reader sub-container
    let container_viewer = gtk::Box::new(gtk::Orientation::Vertical, 6);
    container_viewer.set_homogeneous(false);
    container_viewer.set_hexpand(true);
    container_viewer.set_vexpand(true);
    container_editor.append(&container_viewer);

    //reader
    let reader = gtk::TextView::builder()
        .hexpand(true)
        .vexpand(true)
        .wrap_mode(gtk::WrapMode::Word)
        .build();
    container_viewer.append(&reader);
    reader.buffer().set_text("Dissociated self");
    // status bar
    let status_bar = gtk::Label::builder().label("status bar").build();
    container_viewer.append(&status_bar);

    // actions
    decrypt_button.connect_clicked(clone!(
        #[weak]
        password_entry,
        move |_| {
            password_entry.emit_activate();
        }
    ));

    password_entry.connect_activate(clone!(
        #[weak]
        password_entry,
        #[weak]
        container_app,
        #[weak]
        label_info,
        #[strong]
        writer_ui,
        #[weak]
        list_box,
        #[weak]
        reader,
        move |_| {
            let pwd = password_entry.text();

            login.borrow_mut().on_pwd(pwd.to_string());
            login.borrow_mut().on_login();

            if login.borrow_mut().is_logged_in() {
                //show Writer page
                container_editor.set_visible(true);
                container_login.set_visible(false);
                container_app.remove(&container_login);
                // connect the Writer UI
                writer_ui
                    .borrow_mut()
                    .connect(&db_to_use, pwd.to_string().as_bytes());

                let _ = match writer_ui.borrow_mut().current_entry() {
                    Some(entry) => reader.buffer().set_text(&entry.text),
                    _ => reader.buffer().set_text(&"no entry"),
                };

                // Fill entries list
                let _ = writer_ui
                    .borrow_mut()
                    .entry_view_iter()
                    .for_each(|entry_view| {
                        let label = gtk::Label::new(Some(&entry_view.title));

                        list_box.append(&label);
                    });
            } else {
                label_info.set_text(&login.borrow_mut().info_text);
            }
        }
    ));

    //

    list_box.connect_row_selected(clone!(
        #[weak]
        reader,
        #[weak]
        writer_ui,
        move |_, row| {
            if let Some(row) = row {
                writer_ui
                    .borrow_mut()
                    .on_entry_selected_by_index(row.index() as usize);
                // current_entry text
                let _ = match writer_ui.borrow_mut().current_entry() {
                    Some(entry) => reader.buffer().set_text(&entry.text),
                    _ => reader.buffer().set_text(&"no entry"),
                };
                //current_entry date_times
                let status_text;
                match writer_ui.borrow_mut().view_status_current_entry_date() {
                    Some((id, dt_create, tm_create, dt_modify, tm_modify)) => {
                        status_text = format!(
                            "{} {} - {} {} - [{}]",
                            dt_create, tm_create, dt_modify, tm_modify, id
                        );
                    }
                    None => {
                        status_text = "".to_string();
                    }
                }

                status_bar.set_text(&status_text);
            }
        }
    ));

    //

    return container_app;
}
