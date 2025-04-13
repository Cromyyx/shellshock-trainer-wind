// src/math.rs

// Resources
// https://en.wikipedia.org/wiki/Trajectory_of_a_projectile
// Numerical simulation approach needed for wind integration.

// Use crate:: imports for local modules
use crate::platform::{Rect, Cursor};

use std::fmt; // Required for formatting Hit struct

// --- Core Game Physics / Scaling Constants ---
// Base resolution used for internal scaling calculations
const BASE_WINDOW_RESOLUTION: (u32, u32) = (1768, 992);
// Conversion factor: How many pixels (at base resolution) correspond to one internal "meter"
// CRITICAL for scaling - Needs tuning based on game testing
const BASE_METER_2_PIXEL: f64 = 2.271;
// Gravitational acceleration in internal "meters" per second squared
// CRITICAL for trajectory shape - Needs tuning based on game testing
const GRAVITY_MPSS: f64 = 9.81;

// --- Simulation Parameters ---
// Time step duration for physics simulation (seconds). Smaller = more accurate, slower.
const SIMULATION_DT: f64 = 0.01;
// Maximum number of simulation steps to run before giving up (prevents infinite loops).
const SIMULATION_MAX_STEPS: u32 = 2000;
// Radius around the target (in pixels) considered a "hit".
const HIT_TOLERANCE_PX: f64 = 3.0; // Needs tuning based on game's hit detection
// Conversion factor from user wind input (-100 to 100) to horizontal acceleration (m/s^2).
// CRITICAL for wind effect - Needs extensive tuning based on game testing
const WIND_SCALING_FACTOR: f64 = 0.0125; // Starting guess - **TUNE THIS**
// Buffer below the target (in pixels) used for simulation termination check.
const TERMINATION_Y_BUFFER_PX: f64 = 10.0; // Pixels below target's Y
// --- End Simulation Parameters ---


/// Represents a potential shot solution
#[derive(Debug, Clone)] // Clone needed for sorting/copying results
pub struct Hit {
    velocity: u32, // Initial launch velocity (1-100 m/s)
    angle: i32,    // Initial launch angle (-90 to 90 degrees)
}

impl Hit {
    /// Creates a new Hit instance
    fn new(velocity: u32, angle: i32) -> Self {
        // Use field init shorthand (Rust 2018+)
        Hit { velocity, angle }
    }

    /// Gets the velocity of the hit
    pub fn get_velocity(&self) -> u32 {
        self.velocity
    }

    /// Gets the angle of the hit
    pub fn get_angle(&self) -> i32 {
        self.angle
    }
}

/// How to display a Hit struct in the console output
impl fmt::Display for Hit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Format as (Velocity, Angle) pair
        write!(f, "({},{})", self.velocity, self.angle)
    }
}

/// Calculates the target's position relative to the source (launch point).
/// Takes screen coordinates and returns relative position in *pixels*.
/// Origin (0,0) for the returned tuple is the source/launch point.
pub fn translate_target_position_relativ_to_origin(rect: &Rect,
                                                   from: &Cursor, // Source position (screen coords)
                                                   to: &Cursor)   // Target position (screen coords)
                                                   -> (f64, f64) { // Returns (x_px, y_px) relative to source
    // Scale both points to the base resolution with (0,0) at bottom-left
    let from_scaled = scale_position(rect, from);
    let to_scaled = scale_position(rect, to);

    // Calculate relative position in scaled pixels
    // X positive is right, Y positive is up
    let x_px = to_scaled.0 - from_scaled.0;
    let y_px = to_scaled.1 - from_scaled.1;

    (x_px, y_px)
}

/// Helper function to scale absolute screen coordinates (0,0 top-left)
/// to the base resolution with origin (0,0) at the bottom-left.
fn scale_position(rect: &Rect, cursor: &Cursor) -> (f64, f64) {
    // Get current window dimensions
    let window_width = rect.get_width() as f64;
    let window_height = rect.get_height() as f64;

    // Calculate scaling factors based on base resolution
    let scalex = BASE_WINDOW_RESOLUTION.0 as f64 / window_width;
    let scaley = BASE_WINDOW_RESOLUTION.1 as f64 / window_height;

    // Scale cursor X coordinate
    let cx = cursor.get_x() as f64 * scalex;
    // Scale cursor Y coordinate and invert it to make (0,0) bottom-left
    let cy = (window_height - cursor.get_y() as f64) * scaley;

    (cx, cy)
}


/// Simulates a single projectile trajectory with given initial conditions and wind.
/// Returns `true` if the projectile hits the target within tolerance, `false` otherwise.
fn simulate_trajectory(
    initial_velocity_mps: f64, // Launch velocity (m/s)
    initial_angle_deg: f64,    // Launch angle (degrees)
    target_x_px: f64,          // Target X position relative to source (pixels)
    target_y_px: f64,          // Target Y position relative to source (pixels)
    wind_strength: f64         // User wind input (-100 to 100)
) -> bool {

    // Convert target pixel coordinates to internal "meters"
    let target_x_m = target_x_px / BASE_METER_2_PIXEL;
    let target_y_m = target_y_px / BASE_METER_2_PIXEL;

    // Convert the Y termination buffer from pixels to meters
    let termination_buffer_m = TERMINATION_Y_BUFFER_PX / BASE_METER_2_PIXEL;

    // Calculate initial velocity components in m/s
    let angle_rad = initial_angle_deg.to_radians();

    // *** FIX: Ensure initial horizontal velocity direction matches target direction ***
    // Use target_x_m.signum() to set the correct initial direction (+1.0 for right, -1.0 for left)
    // Handle the case where target_x_m is exactly 0 (straight up/down) - signum might be 0 or 1, default to 1.0
    let direction_sign = if target_x_m == 0.0 { 1.0 } else { target_x_m.signum() };
    let mut vel_x_mps = initial_velocity_mps * angle_rad.cos() * direction_sign;
    // *** END FIX ***

    let mut vel_y_mps = initial_velocity_mps * angle_rad.sin();

    // Calculate constant horizontal acceleration from wind in m/s^2
    let wind_accel_mpss = wind_strength * WIND_SCALING_FACTOR;

    // Initial position (meters, relative to launch point 0,0)
    let mut pos_x_m = 0.0;
    let mut pos_y_m = 0.0;

    // Run the simulation step-by-step
    for _step in 0..SIMULATION_MAX_STEPS {
        // 1. Update velocity components based on acceleration
        vel_x_mps += wind_accel_mpss * SIMULATION_DT; // Apply horizontal wind acceleration
        vel_y_mps -= GRAVITY_MPSS * SIMULATION_DT;   // Apply vertical gravity acceleration

        // 2. Update position based on new velocity
        pos_x_m += vel_x_mps * SIMULATION_DT;
        pos_y_m += vel_y_mps * SIMULATION_DT;

        // 3. Check for hit: Calculate squared distance to target
        let dist_sq_m = (pos_x_m - target_x_m).powi(2) + (pos_y_m - target_y_m).powi(2);
        let hit_tolerance_m = HIT_TOLERANCE_PX / BASE_METER_2_PIXEL;
        // Compare squared distance to squared tolerance (avoids sqrt)
        if dist_sq_m < hit_tolerance_m.powi(2) {
            return true; // Hit detected!
        }

        // 4. Termination Check (as corrected before)
        // Stop simulation if the projectile has fallen significantly below the target
        // AND is currently moving downwards (i.e., it has missed).
        if pos_y_m < (target_y_m - termination_buffer_m) && vel_y_mps < 0.0 {
            return false; // Definitively missed and passed below the target altitude
        }
    }

    // If loop finishes without hitting or terminating early, it's a miss
    false
}


/// Calculates possible launch angles for a fixed velocity range (1-100).
/// Iterates through velocities and angles, using simulation to check for hits.
pub fn calc_launch_angles_with_wind(target_x_px: f64, target_y_px: f64, wind_strength: f64) -> Vec<Hit> {
    let mut hits = Vec::new();
    // Iterate through possible velocities (1 to 100 m/s)
    for v in 1..=100 { // Use inclusive range '..='
        // For each velocity, iterate through possible angles
        let mut angle_deg = -90.0; // Start angle
        while angle_deg <= 90.0 { // End angle condition
            // Simulate this specific shot
            if simulate_trajectory(v as f64, angle_deg, target_x_px, target_y_px, wind_strength) {
                // If simulation results in a hit, record it
                hits.push(Hit::new(v, angle_deg.round() as i32));
            }
            // Increment angle for next test (adjust step for desired precision)
            angle_deg += 0.5; // Smaller step = more precise but slower
        }
    }
    // Sort the found hits primarily by angle, then by velocity
    hits.sort_by(|a, b| a.angle.cmp(&b.angle).then(a.velocity.cmp(&b.velocity)));
    hits
}


/// Calculates possible launch velocities for a fixed angle range (-90 to 90).
/// Iterates through angles and velocities, using simulation to check for hits.
pub fn calc_launch_velocities_with_wind(target_x_px: f64, target_y_px: f64, wind_strength: f64) -> Vec<Hit> {
    let mut hits = Vec::new();
    // Iterate through possible angles (-90 to 90 degrees)
    for angle_deg in -90..=90 { // Use inclusive range '..='
        // For each angle, iterate through possible velocities
        let mut v_mps = 1.0; // Start velocity
        while v_mps <= 100.0 { // End velocity condition
            // Simulate this specific shot
            if simulate_trajectory(v_mps, angle_deg as f64, target_x_px, target_y_px, wind_strength) {
                // If simulation results in a hit, record it after rounding velocity
                let rounded_v = v_mps.round() as u32;
                // Ensure the velocity is within the valid game range (1-100) before adding
                if (1..=100).contains(&rounded_v) {
                    // Avoid adding duplicate velocity entries for the same angle if rounding causes overlap
                    // Check if the last hit added for this angle has the same rounded velocity
                    // FIX for E0282: Added type annotation : &Hit to last_hit
                    if hits.last().map_or(true, |last_hit: &Hit| last_hit.angle != angle_deg || last_hit.velocity != rounded_v) {
                        hits.push(Hit::new(rounded_v, angle_deg));
                    }
                }
            }
            // Increment velocity for next test (adjust step for desired precision)
            v_mps += 0.1; // Smaller step = more precise but slower
        }
    }
    // Sort the found hits primarily by velocity, then by angle
    hits.sort_by(|a, b| a.velocity.cmp(&b.velocity).then(a.angle.cmp(&b.angle)));
    hits
}