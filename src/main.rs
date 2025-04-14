///
/// A simple (non intrusive) trainer for http://www.shellshocklive.com/
///

mod platform;
mod math;

use crate::platform::{Handle, VK, Rect, Cursor};
use crate::math::Hit;

use std::thread;
use std::time;
use std::collections::BTreeMap;
use std::io::{self, Write};

// --- Updated WinAPI imports for v0.3 structure ---
#[cfg(target_os = "windows")]
use winapi::{
    // Use the 'um' (User Mode) submodules common in winapi 0.3
    um::wincon::FlushConsoleInputBuffer, // Function is in wincon
    um::processenv::GetStdHandle,        // Function is in processenv
    um::winbase::STD_INPUT_HANDLE,       // Constant is in winbase
    // Note: INVALID_HANDLE_VALUE is in um::handleapi if needed
};
// --- End WinAPI imports ---


const SHOW_MAX_HITS: usize = 5;

#[derive(Debug, PartialEq, PartialOrd)]
enum Mode {
    ANGLE,
    VELOCITY,
}

fn main() {
    println!("[INFO] Searching for ShellShock Live window...");
    let handle = if cfg!(target_os = "windows") {
        crate::platform::windows::find_shellshock_handle()
    } else {
        panic!("Platform not supported yet (only Windows is implemented).");
    };

    println!("[INFO] ShellShock found. Waiting for input...");
    println!("[INFO] Controls:");
    println!("  1: Set Source Position (Your Tank)");
    println!("  2: Set Target Position (Enemy Tank)");
    println!("  3: Set Wind Strength (via console input)");
    println!("  4: Calculate Hits (using stored wind & dimensions)");
    println!("  5: Clear Positions and Wind");
    println!("  6: Switch Mode (Angle/Velocity)");
    println!("  7: Cache Game Window Dimensions (Press while game is active)");
    start_event_loop(handle);
}

// Generic function over any type H that implements the Handle trait
fn start_event_loop<H: Handle>(handle: H) {
    let mut mode = Mode::VELOCITY;
    let mut source: Option<Cursor> = None;
    let mut target: Option<Cursor> = None;
    let mut current_wind_strength: f64 = 0.0;
    let mut cached_rect: Option<Rect> = None;

    let mut vk1_state = false;
    let mut vk2_state = false;
    let mut vk3_state = false;
    let mut vk4_state = false;
    let mut vk5_state = false;
    let mut vk6_state = false;
    let mut vk7_state = false;

    loop {
        thread::sleep(time::Duration::from_millis(10));

        let vk1_key_down = handle.is_key_pressed(VK::Key1);
        let vk2_key_down = handle.is_key_pressed(VK::Key2);
        let vk3_key_down = handle.is_key_pressed(VK::Key3);
        let vk4_key_down = handle.is_key_pressed(VK::Key4);
        let vk5_key_down = handle.is_key_pressed(VK::Key5);
        let vk6_key_down = handle.is_key_pressed(VK::Key6);
        let vk7_key_down = handle.is_key_pressed(VK::Key7);

        // --- Event Handling ---
        // (Key handler logic remains the same as the previous step)

        // Key 1: Set source position
        if vk1_key_down && !vk1_state {
            vk1_state = true;
            let position = handle.get_mouse_position_in_window();
            println!("[INFO] Position 1 (Source) set to ({}, {}).", position.get_x(), position.get_y());
            source = Some(position);
        } else if !vk1_key_down {
            vk1_state = false
        }

        // Key 2: Set target position
        if vk2_key_down && !vk2_state {
            vk2_state = true;
            let position = handle.get_mouse_position_in_window();
            println!("[INFO] Position 2 (Target) set to ({}, {}).", position.get_x(), position.get_y());
            target = Some(position);
        } else if !vk2_key_down {
            vk2_state = false
        }

        // Key 3: Get/Set Wind Input
        if vk3_key_down && !vk3_state {
            vk3_state = true;
            current_wind_strength = get_wind_input(); // Call the modified function
            println!("[INFO] Wind strength set to: {:.1}", current_wind_strength);
        } else if !vk3_key_down {
            vk3_state = false
        }

        // Key 4: Calculate Hits
        if vk4_key_down && !vk4_state {
            vk4_state = true;
            if let (Some(from), Some(to), Some(ref rect)) = (&source, &target, &cached_rect) {
                let target_pos_pixels = crate::math::translate_target_position_relativ_to_origin(rect, from, to);
                if target_pos_pixels.0.is_nan() || target_pos_pixels.1.is_nan() {
                    println!("[ERROR] Calculated relative position resulted in NaN. Check cached dimensions and coordinates.");
                } else {
                    println!("[INFO] Using cached dimensions: {}x{}", rect.get_width(), rect.get_height());
                    println!("[INFO] Relative target (pixels): ({:.2}, {:.2})", target_pos_pixels.0, target_pos_pixels.1);
                    println!("[INFO] Calculating with Stored Wind Strength: {:.1}", current_wind_strength);
                    let hits: Vec<Hit> = match mode {
                        Mode::ANGLE => crate::math::calc_launch_angles_with_wind(target_pos_pixels.0, target_pos_pixels.1, current_wind_strength),
                        Mode::VELOCITY => crate::math::calc_launch_velocities_with_wind(target_pos_pixels.0, target_pos_pixels.1, current_wind_strength),
                    };
                    if hits.is_empty() {
                        println!("[INFO] No hits found for the given parameters.");
                    } else {
                        print_hits(hits);
                    }
                }
            } else {
                if source.is_none() || target.is_none() {
                    println!("[WARN] Source (1) and Target (2) positions must be set before calculating (4).");
                }
                if cached_rect.is_none() {
                    println!("[WARN] Game window dimensions not cached. Press 7 while game window is active.");
                }
            }
        } else if !vk4_key_down {
            vk4_state = false
        }

        // Key 5: Clear Positions and Wind
        if vk5_key_down && !vk5_state {
            vk5_state = true;
            source = None;
            target = None;
            current_wind_strength = 0.0;
            println!("[INFO] Positions and wind cleared (Wind reset to 0). Cached dimensions remain.");
        } else if !vk5_key_down {
            vk5_state = false
        }

        // Key 6: Switch calculation mode
        if vk6_key_down && !vk6_state {
            vk6_state = true;
            mode = if mode == Mode::ANGLE { Mode::VELOCITY } else { Mode::ANGLE };
            println!("[INFO] Mode changed to '{:?}'.", mode);
        } else if !vk6_key_down {
            vk6_state = false
        }

        // Key 7: Cache Game Window Dimensions
        if vk7_key_down && !vk7_state {
            vk7_state = true;
            println!("[INFO] Attempting to cache game window dimensions...");
            let current_rect = handle.get_window_rect();
            if current_rect.get_width() > 0 && current_rect.get_height() > 0 {
                println!("[INFO] Game window dimensions cached: {}x{}",
                         current_rect.get_width(),
                         current_rect.get_height());
                cached_rect = Some(current_rect);
            } else {
                cached_rect = None;
                println!("[ERROR] Failed to get valid game window dimensions ({}x{}).", current_rect.get_width(), current_rect.get_height());
                println!("[ERROR] Please ensure ShellShock Live window is active/focused and press 7 again.");
            }
        } else if !vk7_key_down {
            vk7_state = false;
        }

    } // End main loop
}

// Function to get wind input from the console
// Uses the corrected imports for winapi 0.3 structures now
fn get_wind_input() -> f64 {
    // --- Flush stdin buffer on Windows before prompting ---
    #[cfg(target_os = "windows")]
    {
        // Use unsafe block for FFI calls
        unsafe {
            // Get the handle to the standard input device
            // Use the directly imported function name now
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            // Check if handle is valid (not NULL and not INVALID_HANDLE_VALUE)
            if !handle.is_null() && handle != winapi::um::handleapi::INVALID_HANDLE_VALUE {
                // Use the directly imported function name now
                if FlushConsoleInputBuffer(handle) == 0 { // Returns BOOL (non-zero on success)
                    // Flush failed - print an error (optional)
                    eprintln!("[WARN] Failed to flush console input buffer. Error code: {}", winapi::um::errhandlingapi::GetLastError());
                }
            } else {
                eprintln!("[WARN] Could not get standard input handle to flush buffer.");
            }
        }
    }
    // --- End flushing logic ---

    // Proceed with the input reading loop
    loop {
        print!("[INPUT] Enter Wind (-100 Left to 100 Right, 0 for none): ");
        io::stdout().flush().expect("Failed to flush stdout");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                match input.trim().parse::<f64>() {
                    Ok(wind) if (-100.0..=100.0).contains(&wind) => return wind,
                    Ok(_) => println!("[ERROR] Wind must be between -100 and 100."),
                    Err(_) => println!("[ERROR] Invalid input. Please enter a number (e.g., -50, 0, 75)."),
                }
            }
            Err(error) => {
                println!("[ERROR] Failed to read input: {}", error);
                return 0.0;
            }
        }
    }
}


// Function to print the calculated hits (Unchanged)
fn print_hits(hits: Vec<Hit>) {
    println!("[INFO] Results (Velocity, Angle):");
    let mut sorted_hits = hits;
    sorted_hits.sort_by(|a, b| {
        a.get_angle().cmp(&b.get_angle())
            .then(a.get_velocity().cmp(&b.get_velocity()))
    });
    println!("Top {} Best -> {}",
             SHOW_MAX_HITS,
             format_hits(&sorted_hits.iter().take(SHOW_MAX_HITS).collect::<Vec<_>>()));
    let categories = into_angle_categories(&sorted_hits);
    for (category, category_hits) in &categories {
        let mut sorted_category_hits: Vec<&Hit> = category_hits.to_vec();
        sorted_category_hits.sort_by(|a, b| a.get_velocity().cmp(&b.get_velocity()));
        println!("Angle ~{} -> {}", category, format_hits(&sorted_category_hits));
    }
}

// Function to format a slice of Hit references into a String (Unchanged)
fn format_hits(hits: &[&Hit]) -> String {
    hits.iter()
        .map(|hit| format!("{}", hit))
        .collect::<Vec<_>>()
        .join(" ")
}

// Function to group Hits into categories based on angle (Unchanged)
fn into_angle_categories(hits: &[Hit]) -> BTreeMap<i32, Vec<&Hit>> {
    let mut map: BTreeMap<i32, Vec<&Hit>> = BTreeMap::new();
    for hit in hits {
        let angle = hit.get_angle();
        let category = (angle as f64 / 10.0).floor() as i32 * 10;
        map.entry(category).or_insert_with(Vec::new).push(hit);
    }
    for hits_in_category in map.values_mut() {
        hits_in_category.sort_by(|a, b| a.get_velocity().cmp(&b.get_velocity()));
        if hits_in_category.len() > SHOW_MAX_HITS {
            hits_in_category.truncate(SHOW_MAX_HITS);
        }
    }
    map
}