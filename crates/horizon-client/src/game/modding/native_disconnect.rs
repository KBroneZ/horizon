//! Mods disconnecting a player (server) or leaving the world (client), NeoForge's
//! `ICommonPacketListener.disconnect` and `IClientCommonPacketListenerExtension.disconnect`.
use super::Game;
use horizon_client::{Result, lan::data::component_nbt, text::Text};
use mod_api::PlayerRef;

impl Game<'_> {
    /// Kicks a connected player with `reason`, as `kick_uuid` does for a ban: the disconnect
    /// packet goes out now, the hub closes the connection when it next handles kicks.
    pub(super) fn player_disconnect(&mut self, player: PlayerRef, reason: Text) -> bool {
        let acting = self.native_acting_player() == Some(player) && !self.lan_acting_fake;
        if !acting && self.online_player_record(player).is_none() {
            return false;
        }
        let uuid = player.uuid();
        let Some(lan) = self.lan.as_ref() else {
            return false;
        };
        // KickCommand: an integrated server can't kick its own host.
        if lan.hub.singleplayer_owner == Some(uuid)
            || !lan.hub.online_players().any(|g| g.uuid == uuid)
        {
            return false;
        }
        self.kick_uuid(uuid, component_nbt(&reason.component()));
        true
    }

    /// Asks to leave the joined server or world at the end of this frame; false with none.
    pub(super) fn disconnect(&mut self, reason: Option<Text>) -> bool {
        if self.remote.is_none() {
            return false;
        }
        self.native_net.leave = Some(reason);
        true
    }

    /// The end of the frame: a mod's `disconnect` leaves now, outside every hook. With a
    /// reason, the disconnect screen shows it; with none, the pause menu's Disconnect runs.
    pub(in crate::game) fn native_leave_if_asked(&mut self) -> Result<()> {
        let Some(reason) = self.native_net.leave.take() else {
            return Ok(());
        };
        if self.remote.is_none() {
            return Ok(());
        }
        match reason {
            Some(reason) => self.leave_server(Some(reason.plain())),
            None => self.close_world(),
        }
    }
}
