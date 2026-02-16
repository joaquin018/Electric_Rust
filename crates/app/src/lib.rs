mod android_utils;

slint::include_modules!();
use slint::ComponentHandle;
use slint::Model;
use std::time::Duration;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    run().unwrap();
}

pub fn run() -> Result<(), slint::PlatformError> {
    // Initialize Tokio runtime for async tasks if needed
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    // Init Haptics (Platform Agnostic)
    android_utils::init_haptics();

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // Copy Handler
    let ui_weak_copy = ui_handle.clone();
    ui.on_request_copy(move || {
        if let Some(ui) = ui_weak_copy.upgrade() {
            let data = format_inventory(ui.get_inv_vals(), ui.get_inv_lengths());
            android_utils::copy_to_clipboard(&data);
            
            let next_id = ui.get_toast_request_id() + 1;
            ui.set_toast_request_id(next_id);
            ui.set_show_copy_toast(true);
            
            let ui_weak_thread = ui_weak_copy.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(2500));
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak_thread.upgrade() {
                        if ui.get_toast_request_id() == next_id {
                            ui.set_show_copy_toast(false);
                        }
                    }
                }).unwrap();
            });
        }
    });

    // Share Handler
    let ui_weak_share = ui_handle.clone();
    ui.on_request_share(move || {
        if let Some(ui) = ui_weak_share.upgrade() {
            let data = format_inventory(ui.get_inv_vals(), ui.get_inv_lengths());
            android_utils::share_text(&data);
        }
    });

    // Menu Open Handler (Non-Blocking Haptic + Aggressive Redraw)
    let ui_weak_menu = ui_handle.clone();
    ui.on_request_menu_open(move || {
         android_utils::trigger_haptic_feedback(); 
         
         if let Some(ui) = ui_weak_menu.upgrade() {
             ui.set_show_sidebar(true);
             
             // AGGRESSIVE WAKE-UP Sidebar
             for i in 0..12 {
                 let ui_weak = ui_weak_menu.clone();
                 slint::Timer::single_shot(Duration::from_millis(i * 8), move || {
                     if let Some(ui) = ui_weak.upgrade() {
                         ui.window().request_redraw();
                     }
                 });
             }
         }
    });

    // Keypad Open Handler (Same WAKE-UP Logic)
    let ui_weak_keypad = ui_handle.clone();
    ui.on_request_activate_item(move |idx| {
         android_utils::trigger_haptic_feedback();
         
         if let Some(ui) = ui_weak_keypad.upgrade() {
             ui.set_active_idx(idx);
             
             // AGGRESSIVE WAKE-UP Keypad
             for i in 0..12 {
                 let ui_weak = ui_weak_keypad.clone();
                 slint::Timer::single_shot(Duration::from_millis(i * 8), move || {
                     if let Some(ui) = ui_weak.upgrade() {
                         ui.window().request_redraw();
                     }
                 });
             }
         }
    });

    // Data Model for Inventory (Mutable from Rust)
    let inv_data: Vec<slint::SharedString> = vec!["".into(); 19];
    let inv_model = std::rc::Rc::new(slint::VecModel::from(inv_data));
    ui.set_inv_vals(inv_model.clone().into());

    // Data Model for Lengths
    let length_data: Vec<i32> = vec![0; 19];
    let length_model = std::rc::Rc::new(slint::VecModel::from(length_data));
    ui.set_inv_lengths(length_model.clone().into());

    // Length Set Handler
    let length_model_weak = length_model.clone();
    let ui_weak_len = ui_handle.clone();
    ui.on_request_set_length(move |idx, len_idx| {
        #[cfg(target_os = "android")]
        android_utils::trigger_haptic_feedback();

        if let Some(_ui) = ui_weak_len.upgrade() {
            let i = idx as usize;
            if i < length_model_weak.row_count() {
                length_model_weak.set_row_data(i, len_idx);
            }
        }
    });

    // Append Digit Handler (7-char Limit)
    let model_weak = inv_model.clone();
    let ui_weak_input = ui_handle.clone();
    ui.on_request_append_digit(move |digit| {
        if let Some(ui) = ui_weak_input.upgrade() {
             let idx = ui.get_active_idx();
             if idx >= 0 {
                 let i = idx as usize;
                 // Row data access from VecModel is cheap
                 if let Some(val) = model_weak.row_data(i) {
                     let s = val.as_str();
                     if s.len() < 7 {
                         let new_val = format!("{}{}", s, digit);
                         model_weak.set_row_data(i, new_val.into());
                     }
                 }
             }
        }
    });

    ui.run()
}

fn format_inventory(
    model: slint::ModelRc<slint::SharedString>,
    length_model: slint::ModelRc<i32>
) -> String {
    struct Section<'a> {
        header: &'a str,
        range: std::ops::Range<usize>,
        labels: &'a [&'a str],
    }

    let sections = [
        Section {
            header: "Apoyos",
            range: 0..8,
            labels: &["30 cm", "40 cm", "50 cm", "60 cm", "70 cm", "80 cm", "90 cm", "1 mt"],
        },
        Section {
            header: "Vigas y Madera",
            range: 8..14,
            labels: &["2x3\"", "2x4\"", "2x5\"", "2x6\"", "2x8\"", "2x10\""],
        },
        Section {
            header: "Clavos",
            range: 14..18,
            labels: &["3\"", "3 1/2\"", "4\"", "Techo 2 1/2\""],
        },
        Section {
            header: "Cemento",
            range: 18..19,
            labels: &["Bolsas"],
        },
    ];

    let mut result = String::new();
    let mut is_first_section = true;

    for section in sections.iter() {
        let mut section_lines = Vec::new();
        for (i, label) in section.range.clone().zip(section.labels.iter()) {
            if let Some(val) = model.row_data(i) {
                let val_str = val.as_str();
                if !val_str.is_empty() && val_str != "0" {
                     if section.header == "Cemento" {
                          section_lines.push(format!("- {} bolsas", val_str));
                     } else if section.header == "Vigas y Madera" {
                          let len_idx = length_model.row_data(i).unwrap_or(0);
                          let len_str = match len_idx {
                              0 => "3.2m",
                              1 => "4m",
                              2 => "5m",
                              3 => "6m",
                              _ => "3.2m",
                          };
                          section_lines.push(format!("- {} de {} de {}", val_str, label, len_str));
                     } else {
                          section_lines.push(format!("- {} de {}", val_str, label));
                     }
                }
            }
        }
        if !section_lines.is_empty() {
            if !is_first_section { result.push_str("\n"); }
            result.push_str(section.header); result.push_str("\n");
            for line in section_lines { result.push_str(&line); result.push_str("\n"); }
            is_first_section = false;
        }
    }
    if result.is_empty() { return String::from("(Sin items seleccionados)"); }
    result
}
