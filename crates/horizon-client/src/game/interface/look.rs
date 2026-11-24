//! What a crate above may change in how the game draws (`horizon_hooks::look`): the engine reads
//! it here only, and each reading is vanilla's while nothing set a look. PvP & HUD sets it today
//! (`game/entities/pvp.rs`, whose `pvp_*` methods, the mod points mods know, read through these).
use super::Game;
use horizon_client::{chat::ChatQol, particles::ParticleType};
use horizon_hooks::look::{Chat, Frame, Look, engine};

impl Game<'_> {
    /// The look this `Game` draws with. The looks are kept per seat in a static that every thread
    /// shares, and a thread other than the window's is always seat 0: the server thread's `Game`
    /// (single player's, a LAN host's) draws nothing and keeps vanilla's, never P1's.
    fn look(&self) -> Look {
        if self.serving() {
            return Look::default();
        }
        engine::get()
    }
    /// Sets this window's look: `change` sets the fields its crate owns. The server thread's
    /// `Game` never does (see [`Game::look`]).
    pub(super) fn set_look(&self, change: impl FnOnce(&mut Look)) {
        if !self.serving() {
            horizon_hooks::look::update(change);
        }
    }
    /// 1.8.9's pictures of a fight: the sword block, the flat item pose and the hand swing.
    pub(super) fn legacy_pose(&self) -> bool {
        self.look().legacy_pose
    }
    pub(super) fn attack_indicator_hidden(&self) -> bool {
        self.look().hide_attack_indicator
    }
    /// A sound a server sent that this player doesn't hear (`Look::mutes_sound`).
    pub(super) fn sound_muted(&self, name: &str) -> bool {
        self.look().mutes_sound(name)
    }
    /// A particle a server sent that this player doesn't see: with the attack particles hidden,
    /// the sweep and the damage hearts, which 1.9 added to a hit (1.8.9's show crits only).
    pub(super) fn particle_hidden(&self, ty: ParticleType) -> bool {
        let attack = matches!(
            ty,
            ParticleType::SweepAttack | ParticleType::DamageIndicator
        );
        attack && self.look().hide_attack_particles
    }
    /// The block outline, hurt overlay and glint colours the renderer draws with.
    pub(super) fn look_frame(&self) -> Frame {
        self.look().frame
    }
    /// The enchantment glint's colour texel, 0 for vanilla's.
    pub(super) fn glint_texel(&self) -> u32 {
        self.look().frame.glint
    }
    /// The held shield's size in percent (100 is vanilla's).
    pub(super) fn shield_size(&self) -> u8 {
        self.look().shield_size
    }
    /// The totem animation's scale (1 is vanilla's).
    pub(super) fn totem_scale(&self) -> f32 {
        self.look().totem_scale
    }
    /// How far the first-person fire overlay is lowered, as a fraction (0 is vanilla's).
    pub(super) fn fire_lower(&self) -> f32 {
        self.look().fire_lower
    }
    /// Lava's fog as with fire resistance, and water's as fully adapted.
    pub(super) fn clear_fluids(&self) -> (bool, bool) {
        let look = self.look();
        (look.clear_lava, look.clear_water)
    }
    /// The chat's extras: Ctrl+F search, copy on click, timestamps, repeats counted.
    pub(super) fn chat_look(&self) -> Chat {
        self.look().chat
    }
    /// What the chat lays its lines out with.
    pub(super) fn chat_qol(&self) -> ChatQol {
        let chat = self.look().chat;
        ChatQol {
            timestamps: chat.timestamps,
            compact: chat.compact,
        }
    }
    /// Whether vanilla's status effect icons are left out (`Hud.extractEffects`).
    pub(super) fn effect_icons_hidden(&self) -> bool {
        self.look().hide_effect_icons
    }
    /// The day time the look shows while the world's is `world` (`Look::day_time`).
    pub(super) fn look_day_time(&self, world: i64) -> i64 {
        self.look().day_time(world)
    }
    /// The rain and thunder levels the look shows; `None` for the world's.
    pub(super) fn look_weather(&self) -> Option<(f32, f32)> {
        self.look().weather
    }
}
