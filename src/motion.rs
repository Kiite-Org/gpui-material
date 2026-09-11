//! Material 3 Expressive motion engine.
//!
//! Provides harmonic oscillator spring physics with expressive (zeta ≈ 0.56)
//! and standard (zeta ≈ 0.8) damping profiles.

/// Spring parameters for thumb motion: stiffness k, damping c, mass m.
/// Expressive is underdamped for visible overshoot, standard is calmer.
/// Mirrors the intent of `MotionScheme.expressive()` vs `standard()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl SpringParams {
    /// Bouncy hero motion, zeta near 0.56.
    #[must_use]
    pub fn expressive() -> Self {
        Self {
            stiffness: 320.0,
            damping: 20.0,
            mass: 1.0,
        }
    }

    /// Calm motion, zeta near 0.8.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            stiffness: 400.0,
            damping: 32.0,
            mass: 1.0,
        }
    }
}

/// Spring state for one animated axis.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SpringState {
    pub pos: f32,
    pub vel: f32,
}

/// Advance a damped spring toward target by dt seconds.
/// Semi-implicit Euler, stable for our stiffness at 60Hz.
#[inline]
#[must_use]
pub fn spring_step(params: SpringParams, state: SpringState, target: f32, dt: f32) -> SpringState {
    let mass = params.mass.max(1e-4);
    let force = -params.stiffness * (state.pos - target) - params.damping * state.vel;
    let vel = state.vel + force / mass * dt;
    let pos = state.pos + vel * dt;
    SpringState { pos, vel }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_step_advances_toward_target() {
        let params = SpringParams::expressive();
        let state = SpringState { pos: 0.0, vel: 0.0 };
        let next = spring_step(params, state, 1.0, 1.0 / 60.0);
        assert!(next.pos > 0.0);
        assert!(next.vel > 0.0);
    }
}
