//! `LivingEntity.readAdditionalSaveData`'s `last_hurt_by_mob` and
//! `ticks_since_last_hurt_by_mob`: a mob loaded with them (a summon's data, a
//! chunk) remembers who hurt it, as `persist::put_last_hurt_by_mob` writes it.
use super::super::Game;
use horizon_client::mobs::{Mob, Who};
use horizon_client::save::number;
use horizon_world::nbt::Tag;

impl Game<'_> {
    /// Adds a mob loaded from `tag` to the level, after reading who last hurt
    /// it: `lastHurtByMob` is the `EntityReference` in `last_hurt_by_mob`,
    /// here a mob already in the level, and its timestamp is
    /// `tickCount - ticks_since_last_hurt_by_mob` (0 when absent). A UUID no
    /// mob in the level has is left unread, as `put_last_hurt_by_mob` would
    /// not write it back either.
    pub(super) fn push_loaded_mob(&mut self, mut m: Mob, tag: &Tag) {
        // `EntityReference.read`: `UUIDUtil.CODEC`, an int array of four.
        if let Some(Tag::IntArray(v)) = tag.get("last_hurt_by_mob")
            && let [a, b, c, d] = v[..]
            && let Some(attacker) = self.mob_by_uuid([a, b, c, d])
        {
            let since = number(tag.get("ticks_since_last_hurt_by_mob")).unwrap_or(0);
            m.ai.last_hurt_by = Some(Who::Mob(attacker));
            m.ai.last_hurt_by_time = (i64::from(m.age) - since) as i32;
        }
        self.mobs.push(m);
    }
}
