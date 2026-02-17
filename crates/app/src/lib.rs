mod android_utils;

slint::include_modules!();
use slint::ComponentHandle;
use slint::Model;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct AppState {
    inv_vals: Vec<String>,
    inv_lengths: Vec<i32>,
}

fn save_state(vals: &slint::ModelRc<slint::SharedString>, lengths: &slint::ModelRc<i32>) {
    let vals_vec: Vec<String> = vals.iter().map(|s| s.to_string()).collect();
    let lengths_vec: Vec<i32> = lengths.iter().collect();

    let state = AppState {
        inv_vals: vals_vec,
        inv_lengths: lengths_vec,
    };
    
    let dir = android_utils::get_app_files_dir();
    let path = format!("{}/construct_data.json", dir);
    
    if let Ok(json) = serde_json::to_string(&state) {
        let _ = std::fs::write(path, json);
    }
}

fn load_state() -> Option<AppState> {
    let dir = android_utils::get_app_files_dir();
    let path = format!("{}/construct_data.json", dir);
    
    if let Ok(content) = std::fs::read_to_string(path) {
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

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

    // Full Share Handler
    let ui_weak_share_full = ui_handle.clone();
    ui.on_request_full_share(move || {
        if let Some(ui) = ui_weak_share_full.upgrade() {
            let data = format_inventory(ui.get_inv_vals(), ui.get_inv_lengths(), true, true, true, true);
            android_utils::share_text(&data);
        }
    });

    // Selective Share Handler
    let ui_weak_share_sel = ui_handle.clone();
    ui.on_request_selective_share(move |base, door, roof, insulation| {
        if let Some(ui) = ui_weak_share_sel.upgrade() {
            let data = format_inventory(ui.get_inv_vals(), ui.get_inv_lengths(), base, door, roof, insulation);
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
    let loaded_state = load_state().unwrap_or_default();
    
    // Ensure we have correct size (35) even if loaded state is different/empty
    let mut iv = loaded_state.inv_vals;
    if iv.len() < 35 { iv.resize(35, "".to_string()); }
    
    let inv_data: Vec<slint::SharedString> = iv.iter().map(|s| s.into()).collect();
    let inv_model = std::rc::Rc::new(slint::VecModel::from(inv_data));
    ui.set_inv_vals(inv_model.clone().into());

    // Data Model for Lengths
    let mut il = loaded_state.inv_lengths;
    if il.len() < 35 { il.resize(35, 0); }
    
    let length_data: Vec<i32> = il;
    let length_model = std::rc::Rc::new(slint::VecModel::from(length_data));
    ui.set_inv_lengths(length_model.clone().into());

    // Length Set Handler
    let length_model_weak = length_model.clone();
    let inv_model_weak_for_save = inv_model.clone();
    let ui_weak_len = ui_handle.clone();
    ui.on_request_set_length(move |idx, len_idx| {
        #[cfg(target_os = "android")]
        android_utils::trigger_haptic_feedback();

        if let Some(_ui) = ui_weak_len.upgrade() {
            let i = idx as usize;
            if i < length_model_weak.row_count() {
                length_model_weak.set_row_data(i, len_idx);
                save_state(&inv_model_weak_for_save.clone().into(), &length_model_weak.clone().into());
            }
        }
    });

    // Append Digit Handler (7-char Limit)
    let model_weak = inv_model.clone();
    let len_model_weak_for_save = length_model.clone();
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
                         save_state(&model_weak.clone().into(), &len_model_weak_for_save.clone().into());
                     }
                 }
             }
        }
    });
    
    // Backspace Handler
    let model_weak_bs = inv_model.clone();
    let len_model_weak_bs = length_model.clone();
    let ui_weak_bs = ui_handle.clone();
    ui.on_request_backspace(move || {
        #[cfg(target_os = "android")]
        android_utils::trigger_haptic_feedback();
        
        if let Some(ui) = ui_weak_bs.upgrade() {
             let idx = ui.get_active_idx();
             if idx >= 0 {
                 let i = idx as usize;
                 if let Some(val) = model_weak_bs.row_data(i) {
                     let s = val.as_str();
                     if !s.is_empty() {
                         let mut chars = s.chars();
                         chars.next_back();
                         let new_val = chars.as_str();
                         model_weak_bs.set_row_data(i, new_val.into());
                         save_state(&model_weak_bs.clone().into(), &len_model_weak_bs.clone().into());
                     }
                 }
             }
        }
    });

    ui.run()
}

fn format_inventory(
    model: slint::ModelRc<slint::SharedString>,
    length_model: slint::ModelRc<i32>,
    inc_base: bool,
    inc_door: bool,
    inc_roof: bool,
    inc_insul: bool,
) -> String {
    struct Section<'a> {
        header: &'a str,
        range: std::ops::Range<usize>,
        labels: &'a [&'a str],
        category: i32, // 0: base, 1: door, 2: roof, 3: insul
    }

    let sections = [
        Section {
            header: "Apoyos",
            range: 0..8,
            labels: &["30 cm", "40 cm", "50 cm", "60 cm", "70 cm", "80 cm", "90 cm", "1 mt"],
            category: 0,
        },
        Section {
            header: "Vigas y Madera",
            range: 8..14,
            labels: &["2x3\"", "2x4\"", "2x5\"", "2x6\"", "2x8\"", "2x10\""],
            category: 0,
        },
        Section {
            header: "Clavos",
            range: 14..18,
            labels: &["3\"", "3 1/2\"", "4\"", "Techo 2 1/2\""],
            category: 0,
        },
        Section {
            header: "Cemento",
            range: 18..19,
            labels: &["Bolsas"],
            category: 0,
        },
        Section {
            header: "Puerta",
            range: 19..23,
            labels: &["60m", "70m", "80m", "90m"],
            category: 1,
        },
        Section {
            header: "Zinc Acanalado",
            range: 23..27,
            labels: &["2.5m", "3.6m", "4m", "6m"],
            category: 2,
        },
        Section {
            header: "Zinc en V",
            range: 27..30,
            labels: &["2.5m", "3.66m", "4m"],
            category: 2,
        },
        Section {
            header: "Full Tech",
            range: 30..31,
            labels: &["3.80m"],
            category: 2,
        },
        Section {
            header: "Lana Vidrio",
            range: 31..32,
            labels: &["Lana vidrio"],
            category: 3,
        },
        Section {
            header: "Rollo Hidrofuga",
            range: 32..33,
            labels: &["Rollo hidrofuga"],
            category: 3,
        },
    ];

    let mut result = String::new();
    let mut is_first_section = true;

    for section in sections.iter() {
        let should_include = match section.category {
            0 => inc_base,
            1 => inc_door,
            2 => inc_roof,
            3 => inc_insul,
            _ => false,
        };
        
        if !should_include { continue; }

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
