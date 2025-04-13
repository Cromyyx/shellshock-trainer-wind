///
/// A simple (non intrusive) trainer for http://www.shellshocklive.com/
///

// Use crate:: prefix for local modules (idiomatic in Rust 2018+)
mod platform;
mod math;

// Bring specific items into scope for easier use
use crate::platform::{Handle, VK, Rect, Cursor};
use crate::math::Hit;

use std::thread;
use std::time;
use std::collections::BTreeMap;
use std::io::{self, Write}; // For console input/output

const SHOW_MAX_HITS: usize = 5; // Max hits to show per category/best

#[derive(Debug, PartialEq, PartialOrd)]
enum Mode {
    ANGLE,
    VELOCITY,
}

fn main() {
    println!("[INFO] Searching for ShellShock Live window...");
    let handle = if cfg!(target_os = "windows") {
        // Use crate:: prefix here too
        crate::platform::windows::find_shellshock_handle()
    } else {
        // Provide a more informative panic message or implement for other platforms
        panic!("Platform not supported yet (only Windows is implemented).");
    };

    println!("[INFO] ShellShock found. Waiting for input...");
    println!("[INFO] Controls:");
    println!("  1: Set Source Position (Your Tank)");
    println!("  2: Set Target Position (Enemy Tank)");
    println!("  3: Calculate Hits (requires Source and Target)");
    println!("  4: Clear Positions");
    println!("  5: Switch Mode (Angle/Velocity)");
    start_event_loop(handle);
}

// Generic function over any type H that implements the Handle trait
fn start_event_loop<H: Handle>(handle: H) {
    let mut mode = Mode::VELOCITY;
    let mut source: Option<Cursor> = None; // Explicit type annotation
    let mut target: Option<Cursor> = None; // Explicit type annotation

    // State tracking for key presses to detect rising edge (press down)
    let mut vk1_state = false;
    let mut vk2_state = false;
    let mut vk3_state = false;
    let mut vk4_state = false;
    let mut vk5_state = false;

    loop {
        // Small delay to prevent high CPU usage
        thread::sleep(time::Duration::from_millis(10));

        // Check current key states
        let vk1_key_down = handle.is_key_pressed(VK::Key1);
        let vk2_key_down = handle.is_key_pressed(VK::Key2);
        let vk3_key_down = handle.is_key_pressed(VK::Key3);
        let vk4_key_down = handle.is_key_pressed(VK::Key4);
        let vk5_key_down = handle.is_key_pressed(VK::Key5);

        // --- Event Handling ---

        // Set position 1 (Source) on key press
        if vk1_key_down && !vk1_state {
            vk1_state = true; // Mark key as pressed

            let position = handle.get_mouse_position_in_window();
            println!("[INFO] Position 1 (Source) set to ({}, {}).", position.get_x(), position.get_y());
            source = Some(position);
        } else if !vk1_key_down {
            vk1_state = false // Reset state when key is released
        }

        // Set position 2 (Target) on key press
        if vk2_key_down && !vk2_state {
            vk2_state = true;

            let position = handle.get_mouse_position_in_window();
            println!("[INFO] Position 2 (Target) set to ({}, {}).", position.get_x(), position.get_y());
            target = Some(position);
        } else if !vk2_key_down {
            vk2_state = false
        }

        // Calculate hits on key press
        if vk3_key_down && !vk3_state {
            vk3_state = true;

            // Use 'if let' for cleaner check and unwrapping of Option values
            if let (Some(from), Some(to)) = (&source, &target) {
                let rect: Rect = handle.get_window_rect();

                // Get wind input from the user via console
                let wind_strength: f64 = get_wind_input();

                // Calculate relative target position in pixels
                let target_pos_pixels = crate::math::translate_target_position_relativ_to_origin(&rect, from, to);
                println!("[INFO] Relative target (pixels): ({:.2}, {:.2})", target_pos_pixels.0, target_pos_pixels.1);
                println!("[INFO] Calculating with Wind Strength: {:.1}", wind_strength);

                // Perform calculation based on current mode
                let hits: Vec<Hit> = match mode {
                    Mode::ANGLE => crate::math::calc_launch_angles_with_wind(target_pos_pixels.0, target_pos_pixels.1, wind_strength),
                    Mode::VELOCITY => crate::math::calc_launch_velocities_with_wind(target_pos_pixels.0, target_pos_pixels.1, wind_strength),
                };

                // Print results or indicate if no hits were found
                if hits.is_empty() {
                    println!("[INFO] No hits found for the given parameters.");
                } else {
                    print_hits(hits); // Pass the owned Vec<Hit>
                }
            } else {
                println!("[WARN] Source (1) and Target (2) positions must be set before calculating (3).");
            }
        } else if !vk3_key_down {
            vk3_state = false
        }

        // Clear positions on key press
        if vk4_key_down && !vk4_state {
            vk4_state = true;

            source = None; // Reset source to None
            target = None; // Reset target to None
            println!("[INFO] Positions cleared.");
        } else if !vk4_key_down {
            vk4_state = false
        }

        // Switch calculation mode on key press
        if vk5_key_down && !vk5_state {
            vk5_state = true;

            // Toggle between modes
            mode = if mode == Mode::ANGLE {
                Mode::VELOCITY
            } else {
                Mode::ANGLE
            };

            println!("[INFO] Mode changed to '{:?}'.", mode);
        } else if !vk5_key_down {
            vk5_state = false
        }
    } // End main loop
}

// Function to get wind input from the console
fn get_wind_input() -> f64 {
    loop { // Loop until valid input is received
        print!("[INPUT] Enter Wind (-100 Left to 100 Right, 0 for none): ");
        // Ensure the prompt is displayed before waiting for input
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Trim whitespace and attempt to parse as f64
                match input.trim().parse::<f64>() {
                    // Use modern inclusive range check
                    Ok(wind) if (-100.0..=100.0).contains(&wind) => {
                        return wind; // Valid input, return it
                    }
                    Ok(_) => {
                        // Parsed but outside the valid range
                        println!("[ERROR] Wind must be between -100 and 100.");
                    }
                    Err(_) => {
                        // Failed to parse as a number
                        println!("[ERROR] Invalid input. Please enter a number (e.g., -50, 0, 75).");
                    }
                }
            }
            Err(error) => {
                // Handle potential errors during input reading
                println!("[ERROR] Failed to read input: {}", error);
                // Decide on fallback behavior - returning 0 is a safe default
                return 0.0;
            }
        }
    } // Repeat loop if input was invalid
}


// Function to print the calculated hits, takes ownership of Vec<Hit>
fn print_hits(hits: Vec<Hit>) {
    println!("[INFO] Results (Velocity, Angle):");

    // Clone the hits vector to sort it without consuming the original
    let mut sorted_hits = hits; // hits is Vec<Hit>, not a reference

    // Sort all hits primarily by angle, secondarily by velocity
    sorted_hits.sort_by(|a, b| {
        a.get_angle().cmp(&b.get_angle())
            .then(a.get_velocity().cmp(&b.get_velocity())) // Use .then() for chaining Ordering
    });

    println!("Top {} Best -> {}",
             SHOW_MAX_HITS,
             // Create a slice of references for the top hits to pass to format_hits
             format_hits(&sorted_hits.iter().take(SHOW_MAX_HITS).collect::<Vec<_>>()));

    // Group hits into categories by angle
    // Pass a slice of the sorted owned Hits
    let categories = into_angle_categories(&sorted_hits);

    // Iterate over the angle categories (BTreeMap<i32, Vec<&Hit>>)
    for (category, category_hits) in &categories { // category_hits is &Vec<&Hit>

        // *** FIX Applied Here ***
        // Clone the references into a new, mutable Vec<&Hit> to allow sorting
        let mut sorted_category_hits: Vec<&Hit> = category_hits.to_vec();

        // Sort the hits within this category by velocity
        sorted_category_hits.sort_by(|a, b| a.get_velocity().cmp(&b.get_velocity()));
        // --- End Fix ---

        // Format and print the sorted hits for the current category
        // Pass a slice of references
        println!("Angle ~{} -> {}", category, format_hits(&sorted_category_hits));
    }
}

// Function to format a slice of Hit references into a String
fn format_hits(hits: &[&Hit]) -> String {
    hits.iter()
        .map(|hit| format!("{}", hit)) // Format each Hit reference
        .collect::<Vec<_>>() // Collect into a Vec<String>
        .join(" ") // Join the strings with spaces
}

// Function to group Hits into categories based on angle
// Takes a slice of owned Hits, returns a Map from angle category (i32) to Vec of references (&Hit)
fn into_angle_categories(hits: &[Hit]) -> BTreeMap<i32, Vec<&Hit>> {
    let mut map: BTreeMap<i32, Vec<&Hit>> = BTreeMap::new();

    // Iterate over the slice of owned Hit structs
    for hit in hits {
        let angle = hit.get_angle();
        // Determine the category (e.g., group by tens: 0-9, 10-19, -10 to -1)
        let category = (angle as f64 / 10.0).floor() as i32 * 10;

        // Insert a reference to the hit into the appropriate category vector in the map
        map.entry(category).or_insert_with(Vec::new).push(hit);
    }

    // Post-process the map: sort hits within each category and truncate if needed
    for hits_in_category in map.values_mut() { // Get mutable access to the Vec<&Hit>
        // Sort the Vec<&Hit> by velocity
        hits_in_category.sort_by(|a, b| a.get_velocity().cmp(&b.get_velocity()));
        // If there are more hits than the display limit, truncate the vector
        if hits_in_category.len() > SHOW_MAX_HITS {
            hits_in_category.truncate(SHOW_MAX_HITS);
        }
    }
    map // Return the processed map
}