//! Where sleeping through the night moves the clock: vanilla's morning, or the mods'
//! `ClockAdjustment` from `sleep_finished_time` (NeoForge's `SleepFinishedTimeEvent`).
use super::Game;
use mod_api::{ClockAdjustment, Identifier};

/// Vanilla's wake-up time marker, where sleeping through the night moves the clock
/// (`ClockTimeMarkers.WAKE_UP_FROM_SLEEP`).
const WAKE_UP_FROM_SLEEP: &str = "minecraft:wake_up_from_sleep";

impl Game<'_> {
    /// The time sleeping moves the clock to from `current`, and whether a mod canceled the move.
    /// The mods may move the wake-up time; never before now (`setTimeAddition` refuses a lower
    /// one), and a marker the clock lacks keeps the morning.
    pub(super) fn sleep_finished_target(&mut self, current: i64) -> (i64, bool) {
        let morning = current + 24000 - current.rem_euclid(24000);
        let mut adjustment =
            ClockAdjustment::Marker(Identifier::parse(WAKE_UP_FROM_SLEEP).expect("a valid id"));
        let mut cancel = false;
        let dimension = self.dim.id();
        self.native_sleep_finished_time(dimension, current, morning, &mut adjustment, &mut cancel);
        let clock = self.level_clock_name(self.dim);
        let target = wake_time(current, morning, &adjustment, |marker| {
            self.clock_marker_time(&clock, marker, current)
        });
        (target, cancel)
    }
}

/// Where sleeping through the night moves the clock from `current`: the mods' `adjustment`
/// resolved, or `morning` (vanilla's) when it lands before `current` or names a marker `marker`
/// doesn't know. Vanilla's own marker is `morning` itself.
fn wake_time(
    current: i64,
    morning: i64,
    adjustment: &ClockAdjustment,
    marker: impl FnOnce(&str) -> Option<i64>,
) -> i64 {
    adjustment
        .resolve(current, |id| {
            let id = format!("{}:{}", id.namespace(), id.path());
            if id == WAKE_UP_FROM_SLEEP {
                return Some(morning);
            }
            marker(&id)
        })
        .filter(|time| *time >= current)
        .unwrap_or(morning)
}

#[cfg(test)]
mod tests {
    use super::{WAKE_UP_FROM_SLEEP, wake_time};
    use mod_api::{ClockAdjustment, Identifier};

    const NOW: i64 = 13_000;
    const MORNING: i64 = 24_000;

    fn marker(id: &str) -> ClockAdjustment {
        ClockAdjustment::Marker(Identifier::parse(id).unwrap())
    }
    /// The Overworld's markers as a test knows them: only `minecraft:noon`, next at 30000.
    fn known(id: &str) -> Option<i64> {
        (id == "minecraft:noon").then_some(30_000)
    }

    #[test]
    fn vanillas_marker_is_the_morning_as_before() {
        assert_eq!(
            wake_time(NOW, MORNING, &marker(WAKE_UP_FROM_SLEEP), |_| None),
            MORNING
        );
    }

    #[test]
    fn a_mods_adjustment_moves_the_clock_where_it_says() {
        assert_eq!(
            wake_time(NOW, MORNING, &ClockAdjustment::Relative(1_000), known),
            14_000
        );
        assert_eq!(
            wake_time(NOW, MORNING, &ClockAdjustment::Absolute(20_000), known),
            20_000
        );
        assert_eq!(
            wake_time(NOW, MORNING, &marker("minecraft:noon"), known),
            30_000
        );
    }

    #[test]
    fn a_time_before_now_or_an_unknown_marker_keeps_the_morning() {
        assert_eq!(
            wake_time(NOW, MORNING, &ClockAdjustment::Absolute(NOW - 1), known),
            MORNING
        );
        assert_eq!(
            wake_time(NOW, MORNING, &ClockAdjustment::Relative(-5), known),
            MORNING
        );
        assert_eq!(
            wake_time(NOW, MORNING, &marker("example:mod_dawn"), known),
            MORNING
        );
    }
}
