//! Mods disconnecting a player (server) or leaving the world (client), NeoForge's
//! `ICommonPacketListener.disconnect` and `IClientCommonPacketListenerExtension.disconnect`.
use super::Game;
use horizon_client::{Result, lan::data::component_nbt, text::Text};
use mod_api::{DisconnectError, PlayerRef, WorldEpoch};

/// A client mod's request to leave, carried out at the end of the frame.
pub(super) struct LeaveRequest {
    /// The world it was made in: a request made while that world closes is dropped.
    epoch: WorldEpoch,
    reason: Option<Text>,
}

impl Game<'_> {
    /// Kicks a connected player with `reason` through the hub's own kick (`Hub::kick`, as bans
    /// and `/kick` do): the disconnect packet goes out and the kick is recorded now, and the
    /// hub's next poll fires `player_kicked` and removes the player. Once a kick is under way,
    /// a further call changes nothing.
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
        if lan.hub.singleplayer_owner == Some(uuid) {
            return false;
        }
        let Some(guest) = lan.hub.online_players().find(|g| g.uuid == uuid) else {
            return false;
        };
        if !guest.kicked {
            self.kick_uuid(uuid, component_nbt(&reason.component()));
        }
        true
    }

    /// Asks to leave the joined server or world at the end of this frame.
    /// Unavailable with no world joined, or while the world is already closing.
    pub(super) fn disconnect(
        &mut self,
        reason: Option<Text>,
    ) -> std::result::Result<(), DisconnectError> {
        let epoch = self.native_lifecycle.epoch();
        let (Some(epoch), Some(_), None) = (epoch, self.remote.as_ref(), &self.pending_save) else {
            return Err(DisconnectError::Unavailable);
        };
        self.native_net.leave = Some(LeaveRequest { epoch, reason });
        Ok(())
    }

    /// The end of the frame: a mod's `disconnect` leaves now, outside every hook. With a
    /// reason, the disconnect screen shows it; with none, the pause menu's Disconnect runs. A
    /// request from a world that has closed since is dropped.
    pub(in crate::game) fn native_leave_if_asked(&mut self) -> Result<()> {
        let Some(request) = self.native_net.leave.take() else {
            return Ok(());
        };
        if self.remote.is_none() || self.native_lifecycle.epoch() != Some(request.epoch) {
            return Ok(());
        }
        let left = match request.reason {
            Some(reason) => self.leave_server(Some(reason.plain())),
            None => self.close_world(),
        };
        // A hook the leaving fired (a disconnect or unload listener) may have asked again: that
        // request belongs to the world just closed.
        self.native_net.leave = None;
        left
    }
}
