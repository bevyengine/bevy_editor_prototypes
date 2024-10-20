//! Provides the [`Momentum`] settings.

use std::time::Duration;

use bevy::math::{DVec2, DVec3};
use bevy::reflect::prelude::*;

/// Defines momentum behavior of this [`super::component::EditorCam`].
#[derive(Debug, Clone, Copy, Reflect)]
pub struct Momentum {
    /// Momentum decay scales with velocity.
    pub pan_damping: u8,
    /// Momentum decay is constant.
    pub pan_friction: f64,
    /// The sampling window to use when a movement ends to determine the velocity of the camera when
    /// momentum decay begins. The higher this value, the easier it is to "flick" the camera, but
    /// the more of a velocity discontinuity will be present when momentum starts.
    pub init_pan: Duration,
    /// Momentum decay scales with velocity.
    pub orbit_damping: u8,
    /// Momentum decay is constant.
    pub orbit_friction: f64,
    /// The sampling window to use when a movement ends to determine the velocity of the camera when
    /// momentum decay begins. The higher this value, the easier it is to "flick" the camera, but
    /// the more of a velocity discontinuity will be present when momentum starts.
    pub init_orbit: Duration,
}

impl Default for Momentum {
    fn default() -> Self {
        Self {
            pan_damping: 160,
            pan_friction: 0.2,
            init_pan: Duration::from_millis(40),
            orbit_damping: 160,
            orbit_friction: 0.2,
            init_orbit: Duration::from_millis(60),
        }
    }
}

impl Momentum {
    fn decay_velocity_orbit(self, velocity: DVec2, delta_time: Duration) -> DVec2 {
        let velocity =
            velocity * (self.orbit_damping as f64 / 256.0).powf(delta_time.as_secs_f64() * 10.0);
        let static_decay =
            velocity.normalize() * self.orbit_friction * delta_time.as_secs_f64() * 120.0;
        let static_decay_clamped = static_decay.abs().min(velocity.abs()) * velocity.signum();
        velocity - static_decay_clamped
    }

    fn decay_velocity_pan(self, velocity: DVec2, delta_time: Duration) -> DVec2 {
        let velocity =
            velocity * (self.pan_damping as f64 / 256.0).powf(delta_time.as_secs_f64() * 10.0);
        let static_decay =
            velocity.normalize() * self.pan_friction * delta_time.as_secs_f64() * 120.0;
        let static_decay_clamped = static_decay.abs().min(velocity.abs()) * velocity.signum();
        velocity - static_decay_clamped
    }
}

/// The velocity of the camera.
#[derive(Debug, Clone, Copy, Default, Reflect)]
pub enum Velocity {
    /// The velocity is zero and the camera will transition into the Stationary state.
    #[default]
    None,
    ///Camera is spinning.
    Orbit {
        /// The anchor of rotation being orbited about.
        anchor: DVec3,
        /// The current velocity of the camera about the anchor.
        velocity: DVec2,
    },
    /// Camera is sliding.
    Pan {
        /// The anchor point that should stick to the pointer during panning.
        anchor: DVec3,
        /// The current panning velocity of the camera.
        velocity: DVec2,
    },
}

impl Velocity {
    const DECAY_THRESHOLD: f64 = 1e-3;
    /// Decay the velocity based on the momentum setting.
    pub fn decay(&mut self, momentum: Momentum, delta_time: Duration) {
        let is_none = match self {
            Velocity::None => true,
            Velocity::Orbit {
                ref mut velocity, ..
            } => {
                *velocity = momentum.decay_velocity_orbit(*velocity, delta_time);
                velocity.length() <= Self::DECAY_THRESHOLD
            }
            Velocity::Pan {
                ref mut velocity, ..
            } => {
                *velocity = momentum.decay_velocity_pan(*velocity, delta_time);
                velocity.length() <= Self::DECAY_THRESHOLD
            }
        };

        if is_none {
            *self = Velocity::None;
        }
    }
}
