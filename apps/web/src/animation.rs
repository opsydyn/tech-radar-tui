#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    Running,
    Paused,
}

const ANIMATION_SWEEP_SPEED: f64 = 1.4;
const ANIMATION_MAX_FRAME_DELTA: f64 = 0.25;
const ANIMATION_FULL_ROTATION: f64 = 2.0 * std::f64::consts::PI;

pub fn advance_animation_counter(
    counter: f64,
    last_tick: Option<f64>,
    now_seconds: f64,
    mode: AnimationMode,
) -> (f64, Option<f64>) {
    let delta = last_tick
        .map(|last| (now_seconds - last).max(0.0).min(ANIMATION_MAX_FRAME_DELTA))
        .unwrap_or(0.0);

    let next_counter = match mode {
        AnimationMode::Running => {
            (counter + delta * ANIMATION_SWEEP_SPEED).rem_euclid(ANIMATION_FULL_ROTATION)
        }
        AnimationMode::Paused => counter.rem_euclid(ANIMATION_FULL_ROTATION),
    };

    (next_counter, Some(now_seconds))
}

#[cfg(test)]
mod tests {
    use super::{
        advance_animation_counter, AnimationMode, ANIMATION_FULL_ROTATION, ANIMATION_SWEEP_SPEED,
    };

    fn assert_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-9,
            "expected {expected}, got {actual}, diff {diff}"
        );
    }

    #[test]
    fn first_tick_initializes_time_without_advancing() {
        let start_counter = 1.2345;
        let (counter, last_tick) =
            advance_animation_counter(start_counter, None, 10.0, AnimationMode::Running);

        assert_close(counter, start_counter);
        assert_eq!(last_tick, Some(10.0));
    }

    #[test]
    fn running_mode_advances_counter_and_wraps() {
        let start_counter = ANIMATION_FULL_ROTATION - 0.1;
        let (counter, last_tick) =
            advance_animation_counter(start_counter, Some(4.0), 4.2, AnimationMode::Running);

        let expected =
            (start_counter + 0.2 * ANIMATION_SWEEP_SPEED).rem_euclid(ANIMATION_FULL_ROTATION);
        assert_close(counter, expected);
        assert_eq!(last_tick, Some(4.2));
    }

    #[test]
    fn paused_mode_keeps_counter_stable_but_updates_clock() {
        let start_counter = 2.25;
        let (counter, last_tick) =
            advance_animation_counter(start_counter, Some(1.0), 1.2, AnimationMode::Paused);

        assert_close(counter, start_counter);
        assert_eq!(last_tick, Some(1.2));
    }

    #[test]
    fn large_frame_gap_is_clamped() {
        let start_counter = 0.0;
        let (counter, _) =
            advance_animation_counter(start_counter, Some(3.0), 30.0, AnimationMode::Running);

        let expected = 0.25 * ANIMATION_SWEEP_SPEED;
        assert_close(counter, expected);
    }

    #[test]
    fn backwards_time_does_not_reverse_animation() {
        let start_counter = 3.5;
        let (counter, last_tick) =
            advance_animation_counter(start_counter, Some(10.0), 9.0, AnimationMode::Running);

        assert_close(counter, start_counter);
        assert_eq!(last_tick, Some(9.0));
    }
}
