//! `LivingEntity.readAdditionalSaveData`'s `last_hurt_by_mob` and
//! `ticks_since_last_hurt_by_mob`: a mob loaded with them (a summon's data, a
//! chunk) remembers who hurt it, as `persist::put_last_hurt_by_mob` writes it.
use super::super::Game;
use horizon_client::mobs::{Mob, Who};
use horizon_client::save::number;
use horizon_world::nbt::Tag;

impl Game<'_> {
    /// Adds a mob loaded from the compound `remember` kept for it to the
    /// level, after reading who last hurt it: `lastHurtByMob` is the
    /// `EntityReference` in `last_hurt_by_mob`, here a mob already in the
    /// level, and its timestamp is `tickCount - ticks_since_last_hurt_by_mob`
    /// (0 when absent; a long narrows as `getIntOr` does).
    ///
    /// PARTIAL: vanilla's reference resolves lazily, so an attacker loaded
    /// after this mob (later in the same chunk) still counts. Here a UUID no
    /// mob in the level has yet reads as no record, which
    /// `put_last_hurt_by_mob` then doesn't write back.
    pub(super) fn push_loaded_mob(&mut self, mut m: Mob) {
        let tag = self.persist.raw.get(&m.id);
        // `EntityReference.read`: `UUIDUtil.CODEC`, an int array of four.
        let uuid = match tag.and_then(|t| t.get("last_hurt_by_mob")) {
            Some(Tag::IntArray(v)) if v.len() == 4 => Some([v[0], v[1], v[2], v[3]]),
            _ => None,
        };
        let since = tag
            .and_then(|t| number(t.get("ticks_since_last_hurt_by_mob")))
            .unwrap_or(0) as i32;
        if let Some(attacker) = uuid.and_then(|u| self.mob_by_uuid(u)) {
            m.ai.last_hurt_by = Some(Who::Mob(attacker));
            m.ai.last_hurt_by_time = (m.age as i32).wrapping_sub(since);
        }
        self.mobs.push(m);
    }
}
