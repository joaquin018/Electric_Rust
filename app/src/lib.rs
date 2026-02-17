mod android_utils;

slint::include_modules!();
use slint::ComponentHandle;
use slint::Model;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
struct Project {
    id: String,
    name: String,
    inv_vals: Vec<String>,
    inv_lengths: Vec<i32>,
    last_modified: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct AppState {
    projects: Vec<Project>,
    active_project_id: Option<String>,
}

// Fallback for migration: old structure
#[derive(Serialize, Deserialize)]
struct OldAppState {
    inv_vals: Vec<String>,
    inv_lengths: Vec<i32>,
}

fn save_state(state: &AppState) {
    let dir = android_utils::get_app_files_dir();
    let path = format!("{}/construct_data_v2.json", dir);
    
    if let Ok(json) = serde_json::to_string(state) {
        let _ = std::fs::write(path, json);
    }
}

fn load_state() -> AppState {
    let dir = android_utils::get_app_files_dir();
    let path_v2 = format!("{}/construct_data_v2.json", dir);
    
    // Try loading v2
    if let Ok(content) = std::fs::read_to_string(&path_v2) {
        if let Ok(state) = serde_json::from_str::<AppState>(&content) {
            return state;
        }
    }
    
    // Migration: Check for v1
    let path_v1 = format!("{}/construct_data.json", dir);
    if let Ok(content) = std::fs::read_to_string(&path_v1) {
        if let Ok(old_state) = serde_json::from_str::<OldAppState>(&content) {
            // Create default project from old state
            let default_project = Project {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Proyecto Principal".to_string(),
                inv_vals: old_state.inv_vals,
                inv_lengths: old_state.inv_lengths,
                last_modified: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            };
            
            let new_state = AppState {
                projects: vec![default_project.clone()],
                active_project_id: Some(default_project.id),
            };
            save_state(&new_state); // Save as v2
            return new_state;
        }
    }

    // Default if nothing found: Create one empty project so user lands on it?
    // User wants "create menu". Let's return empty state.
    AppState::default()
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

    // Scroll Wake-Up Monitor (120Hz Fluidity)
    let scroll_timer = slint::Timer::default();
    {
        let ui_weak = ui_handle.clone();
        let mut last_y = ui.get_viewport_y_tracker();
        let mut frames_unchanged = 0;
        
        scroll_timer.start(slint::TimerMode::Repeated, Duration::from_millis(16), move || {
             if let Some(ui) = ui_weak.upgrade() {
                 let current_y = ui.get_viewport_y_tracker();
                 
                 // If scroll position changed (scrolling active)
                 if current_y != last_y {
                     ui.window().request_redraw();
                     frames_unchanged = 0;
                 } else {
                     frames_unchanged += 1;
                     // Keep waking up for ~300ms (20 frames) after scroll stops for smooth settling
                     if frames_unchanged < 20 {
                         ui.window().request_redraw();
                     }
                 }
                 last_y = current_y;
             }
        });
    }

    // Load State
    let app_state = load_state();
    let state_rc = std::rc::Rc::new(std::cell::RefCell::new(app_state));

    // Callbacks
    
    // Create Project
    let state_weak_create = state_rc.clone();
    let ui_weak_create = ui_handle.clone();
    ui.on_request_create_project(move |name| {
        let mut state = state_weak_create.borrow_mut();
        
        // Use uuid v4
        let new_project = Project {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            inv_vals: vec!["".to_string(); 35],
            inv_lengths: vec![0; 35],
            last_modified: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        };

        state.projects.push(new_project);
        save_state(&state);

        // Update UI List
        if let Some(ui) = ui_weak_create.upgrade() {
             let names: Vec<slint::SharedString> = state.projects.iter().map(|p| p.name.clone().into()).collect();
             ui.set_project_names(std::rc::Rc::new(slint::VecModel::from(names)).into());
        }
    });

    // Delete Project
    let state_weak_del = state_rc.clone();
    let ui_weak_del = ui_handle.clone();
    ui.on_request_delete_project(move |idx| {
         let mut state = state_weak_del.borrow_mut();
         let i = idx as usize;
         if i < state.projects.len() {
             let deleted_id = state.projects[i].id.clone();
             state.projects.remove(i);
             if state.active_project_id.as_ref() == Some(&deleted_id) {
                 state.active_project_id = None;
             }
             save_state(&state);

             if let Some(ui) = ui_weak_del.upgrade() {
                 let names: Vec<slint::SharedString> = state.projects.iter().map(|p| p.name.clone().into()).collect();
                 ui.set_project_names(std::rc::Rc::new(slint::VecModel::from(names)).into());
             }
         }
    });

    // Open Project
    let state_weak_open = state_rc.clone();
    let ui_weak_open = ui_handle.clone();
    ui.on_request_open_project(move |idx| {
         let mut state = state_weak_open.borrow_mut();
         let i = idx as usize;
         if i < state.projects.len() {
             // 1. Get ID and Data efficiently
             let new_active_id = state.projects[i].id.clone();
             let mut iv = state.projects[i].inv_vals.clone();
             let mut il = state.projects[i].inv_lengths.clone();
             
             // 2. Update State
             state.active_project_id = Some(new_active_id);
             save_state(&state);
             
             // 3. Prepare UI Data
             if iv.len() < 35 { iv.resize(35, "".to_string()); }
             let inv_data: Vec<slint::SharedString> = iv.iter().map(|s| s.into()).collect();
             
             if il.len() < 35 { il.resize(35, 0); }
             let il_data: Vec<i32> = il;

             if let Some(ui) = ui_weak_open.upgrade() {
                  ui.set_inv_vals(std::rc::Rc::new(slint::VecModel::from(inv_data)).into());
                  ui.set_inv_lengths(std::rc::Rc::new(slint::VecModel::from(il_data)).into());
                  
                  // Transition to Editor
                  ui.set_view_mode(1);
             }
         }
    });

    // Rename Project
    let state_weak_rename = state_rc.clone();
    let ui_weak_rename = ui_handle.clone();
    ui.on_request_rename_project(move |idx, new_name| {
        let mut state = state_weak_rename.borrow_mut();
        let i = idx as usize;
        if i < state.projects.len() {
            state.projects[i].name = new_name.to_string();
            save_state(&state);
             if let Some(ui) = ui_weak_rename.upgrade() {
                 let names: Vec<slint::SharedString> = state.projects.iter().map(|p| p.name.clone().into()).collect();
                 ui.set_project_names(std::rc::Rc::new(slint::VecModel::from(names)).into());
             }
        }
    });

    // Append Digit
    let state_weak_append = state_rc.clone();
    let ui_weak_append = ui_handle.clone();
    ui.on_request_append_digit(move |digit| {
        if let Some(ui) = ui_weak_append.upgrade() {
             let idx = ui.get_active_idx();
             if idx >= 0 {
                 let i = idx as usize;
                 let model = ui.get_inv_vals(); // Get current model
                 if let Some(val) = model.row_data(i) {
                     let s = val.as_str();
                     if s.len() < 7 {
                         let new_val = format!("{}{}", s, digit);
                         model.set_row_data(i, new_val.clone().into());
                         
                         let mut state = state_weak_append.borrow_mut();
                         let active_id = state.active_project_id.clone(); // Clone ID to avoid borrow conflict
                         if let Some(active_id) = active_id {
                             if let Some(proj) = state.projects.iter_mut().find(|p| p.id == active_id) {
                                 if i < proj.inv_vals.len() {
                                     proj.inv_vals[i] = new_val;
                                 }
                             }
                             save_state(&state); // Save after mutable borrow of proj ends
                         }
                     }
                 }
             }
        }
    });

    // Backspace
    let state_weak_bs = state_rc.clone();
    let ui_weak_bs = ui_handle.clone();
    ui.on_request_backspace(move || {
        #[cfg(target_os = "android")]
        android_utils::trigger_haptic_feedback();
        
        if let Some(ui) = ui_weak_bs.upgrade() {
             let idx = ui.get_active_idx();
             if idx >= 0 {
                 let i = idx as usize;
                 let model = ui.get_inv_vals();
                 if let Some(val) = model.row_data(i) {
                     let s = val.as_str();
                     if !s.is_empty() {
                         let mut chars = s.chars();
                         chars.next_back();
                         let new_val = chars.as_str().to_string();
                         model.set_row_data(i, new_val.clone().into());
                         
                         let mut state = state_weak_bs.borrow_mut();
                         let active_id = state.active_project_id.clone();
                         if let Some(active_id) = active_id {
                             if let Some(proj) = state.projects.iter_mut().find(|p| p.id == active_id) {
                                  if i < proj.inv_vals.len() {
                                      proj.inv_vals[i] = new_val;
                                  }
                             }
                             save_state(&state);
                         }
                     }
                 }
             }
        }
    });

    // Set Length
    let state_weak_len = state_rc.clone();
    let ui_weak_len = ui_handle.clone();
    ui.on_request_set_length(move |idx, len_idx| {
        #[cfg(target_os = "android")]
        android_utils::trigger_haptic_feedback();

        if let Some(ui) = ui_weak_len.upgrade() {
             let i = idx as usize;
             let model = ui.get_inv_lengths();
             model.set_row_data(i, len_idx);
             
             let mut state = state_weak_len.borrow_mut();
             let active_id = state.active_project_id.clone();
             if let Some(active_id) = active_id {
                 if let Some(proj) = state.projects.iter_mut().find(|p| p.id == active_id) {
                      if i < proj.inv_lengths.len() {
                          proj.inv_lengths[i] = len_idx;
                      }
                 }
                 save_state(&state);
             }
        }
    });

    // Selective Share Handler
    let ui_weak_share_sel = ui_handle.clone();
    ui.on_request_selective_share(move |base, door, roof, insulation| {
        if let Some(ui) = ui_weak_share_sel.upgrade() {
            let data = format_inventory(ui.get_inv_vals(), ui.get_inv_lengths(), base, door, roof, insulation);
            android_utils::share_text(&data);
            
            // WAKE-UP after share return (restore fluidity)
            for i in 0..15 {
                let ui_weak = ui_weak_share_sel.clone();
                slint::Timer::single_shot(Duration::from_millis(i * 10), move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.window().request_redraw();
                    }
                });
            }
        }
    });

    // Share Picker Open Handler
    let ui_weak_share_open = ui_handle.clone();
    ui.on_request_share_picker_open(move || {
        android_utils::trigger_haptic_feedback();
        
        if let Some(_) = ui_weak_share_open.upgrade() {
            for i in 0..60 {
                let ui_weak = ui_weak_share_open.clone();
                slint::Timer::single_shot(Duration::from_millis(i * 8), move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.window().request_redraw();
                    }
                });
            }
        }
    });

    // Menu Open Handler (Sidebar Gesture -> Project List)
    let ui_weak_menu = ui_handle.clone();
    ui.on_request_menu_open(move || {
         android_utils::trigger_haptic_feedback(); 
         if let Some(ui) = ui_weak_menu.upgrade() {
             // Close sidebar (visual drawer) if it was somehow open
             ui.set_show_sidebar(false);
             // Switch to Project List (View Mode 0)
             ui.set_view_mode(0);
             
             // Wake up for transition
             ui.window().request_redraw();
         }
    });

    // Keypad Open Handler
    let ui_weak_keypad = ui_handle.clone();
    ui.on_request_activate_item(move |idx| {
         android_utils::trigger_haptic_feedback();
         if let Some(ui) = ui_weak_keypad.upgrade() {
             ui.set_active_idx(idx);
             for i in 0..60 {
                 let ui_weak = ui_weak_keypad.clone();
                 slint::Timer::single_shot(Duration::from_millis(i * 8), move || {
                     if let Some(ui) = ui_weak.upgrade() {
                         ui.window().request_redraw();
                     }
                 });
             }
         }
    });
    
    // Initial Load - Populate Project List
    {
        let state = state_rc.borrow();
        let names: Vec<slint::SharedString> = state.projects.iter().map(|p| p.name.clone().into()).collect();
        ui.set_project_names(std::rc::Rc::new(slint::VecModel::from(names)).into());
    }

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
