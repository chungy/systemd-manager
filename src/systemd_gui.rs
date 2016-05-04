extern crate pango;  // Allows manipulating font styles
use systemd_dbus;    // The dbus-based backend for systemd
use gtk;
use gtk::prelude::*;
use gdk::enums::key;

pub fn launch() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let unit_files = systemd_dbus::list_unit_files(systemd_dbus::SortMethod::Name);
    let container = gtk::ScrolledWindow::new(None, None);
    generate_services(&container, &unit_files);

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    configure_main_window(&window);

    window.add(&container);
    window.show_all();

    // Define action on key press
    window.connect_key_press_event(move |_, key| {
        if let key::Escape = key.get_keyval() { gtk::main_quit(); }
        gtk::Inhibit(false)
    });

    gtk::main();
}

// create_list_widget! creates the widgets for each section
macro_rules! create_list_widget {
    ($label:expr, $label_font:expr, $top:expr) => {{
        let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        if !$top { list.add(&gtk::Separator::new(gtk::Orientation::Horizontal)); }
        let label = gtk::Label::new(Some($label));
        label.override_font(&$label_font);
        list.pack_start(&label, true, true, 0);
        list
    }};
}

// collect_units performs a loop over a list of units and creates a widget from each.
macro_rules! collect_units {
    ($filter_function:ident, $list:expr, $units:expr) => {
        for unit in systemd_dbus::$filter_function($units) {
            $list.add(&gtk::Separator::new(gtk::Orientation::Horizontal));
            $list.pack_start(&get_unit_widget(unit), false, false, 3);
        }
    }
}

// rm_directory_path takes either a &str or String and returns a String with directory paths removed
macro_rules! rm_directory_path {
    ($input:expr) => {{
        let temp = $input;
        let mut split: Vec<&str> = temp.split('/').collect();
        String::from(split.pop().unwrap())
    }}
}

/// Configures all of the options for the main window
fn configure_main_window(window: &gtk::Window) {
    window.set_title("System Services");
    window.set_default_size(500,500);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::Inhibit(true)
    });
}

// generate_services() creates a gtk::ScrolledWindow widget containing the list of units available
// on the system. Each individual unit is created by get_unit_widget() and added to their respective
// gtk::Box.
fn generate_services(container: &gtk::ScrolledWindow, unit_files: &[systemd_dbus::SystemdUnit]) {
    let mut label_font = pango::FontDescription::new();
    label_font.set_weight(pango::Weight::Medium);

    let service_list = create_list_widget!("Services (Activate on Startup)", label_font, true);
    let socket_list  = create_list_widget!("Sockets (Activate On Use)", label_font, false);
    let timer_list   = create_list_widget!("Timers (Activate Periodically)", label_font, false);

    collect_units!(collect_togglable_services, service_list, unit_files.clone());
    collect_units!(collect_togglable_sockets, socket_list, unit_files.clone());
    collect_units!(collect_togglable_timers, timer_list, unit_files.clone());

    service_list.add(&socket_list);
    service_list.add(&timer_list);
    container.add(&service_list);
}

// Removes the directory path and extension from the unit name
fn get_unit_name(x: &str) -> String {
    let mut output = rm_directory_path!(x);
    let mut last_occurrence: usize = 0;
    for (index, value) in output.chars().enumerate() {
        if value == '.' { last_occurrence = index; }
    }
    output.truncate(last_occurrence);
    output
}

// get_unit_widget() takes a SystemdUnit and generates a gtk::Box widget from that information.
fn get_unit_widget(unit: systemd_dbus::SystemdUnit) -> gtk::Box {
    let switch = match unit.state {
        systemd_dbus::UnitState::Disabled => gtk::Button::new_with_label(" Enable"),
        systemd_dbus::UnitState::Enabled  => gtk::Button::new_with_label("Disable"),
        _ => unreachable!(), // This program currently only collects units that fit the above.
    };

    { // Defines action when clicking on the {en/dis}able toggle switch.
        let service = unit.name.clone();
        switch.connect_clicked(move |switch| {
            let filename = rm_directory_path!(&service);
            if &switch.get_label().unwrap() == "Disable" {
                match systemd_dbus::disable(&filename) {
                    Some(error) => print_dialog(&error),
                    None => switch.set_label(" Enable")
                }
            } else {
                match systemd_dbus::enable(&filename) {
                    Some(error) => print_dialog(&error),
                    None => switch.set_label("Disable")
                }
            }
        });
    }

    // Start Button
    let start_button = gtk::Button::new_with_label("Start"); {
        let unit = rm_directory_path!(unit.name.clone());
        start_button.connect_clicked(move |_| {
            if let Some(error) = systemd_dbus::start(&unit) {
                print_dialog(&error);
            }
        });
    }

    // Stop Button
    let stop_button = gtk::Button::new_with_label("Stop"); {
        let unit = rm_directory_path!(unit.name.clone());
        stop_button.connect_clicked(move |_| {
            if let Some(error) = systemd_dbus::stop(&unit) {
                print_dialog(&error);
            }
        });
    }

    let mut label_font = pango::FontDescription::new();
    label_font.set_weight(pango::Weight::Medium);
    let label = gtk::Label::new(Some(&get_unit_name(&unit.name)));
    label.override_font(&label_font);

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    button_box.pack_start(&switch, false, false, 1);
    button_box.pack_start(&start_button, false, false, 1);
    button_box.pack_start(&stop_button, false, false, 1);
    button_box.set_halign(gtk::Align::End);

    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    layout.pack_start(&label, false, false, 5);
    layout.pack_start(&button_box, true, true, 15);

    layout
}

fn print_dialog(message: &str) {
    let dialog = gtk::Dialog::new();
    dialog.set_title("Systemd Error");
    let content = dialog.get_content_area();
    let text = gtk::TextView::new();
    text.get_buffer().unwrap().set_text(message);
    text.set_left_margin(5);
    text.set_right_margin(5);
    content.add(&text);
    dialog.show_all();
}
