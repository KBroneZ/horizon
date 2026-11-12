//! A mod disconnects a player (server) or leaves the world (client) (#3451).
use super::native_entity_refs::tests::with_client_game;
use super::native_guest_turn_tests::with_guests;
use horizon_client::title::Mode;
use mod_api::{ClientHost, ServerHost, Text};

fn kicked() -> Text {
    Text::translate("mymod.kicked", vec![])
}

fn left_reason(game: &super::Game<'_>, conn: u32) -> String {
    let hub = &game.lan.as_ref().unwrap().hub;
    hub.guest(conn).unwrap().left_reason.clone()
}

/// The guest gets the reason, the other guest stays, and the host's owner can't be kicked.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_mod_disconnects_a_guest_with_its_reason() {
    with_guests(|game| {
        let guest = game.player(12).unwrap();
        let other = game.player(13).unwrap();
        let host: &mut dyn ServerHost = game;
        assert_eq!(guest.disconnect(host, kicked()), Ok(true));
        assert_eq!(left_reason(game, 12), "mymod.kicked");
        assert_eq!(left_reason(game, 13), "");
        // KickCommand: the single-player world's owner stays.
        game.lan.as_mut().unwrap().hub.singleplayer_owner = Some(13);
        let host: &mut dyn ServerHost = game;
        assert_eq!(other.disconnect(host, kicked()), Ok(false));
        assert_eq!(left_reason(game, 13), "");
    });
}

/// Inside the guest's own turn (a `net_from_client` handler, say), the acting player can be
/// disconnected too; a handle from another world can't.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn the_acting_player_can_be_disconnected_and_a_stale_handle_cannot() {
    with_guests(|game| {
        let guest = game.player(12).unwrap();
        let stale = mod_api::__private::game::player_ref(
            mod_api::EntityRef::new(
                mod_api::WorldEpoch::new(guest.entity().epoch().get() + 1),
                guest.entity().id(),
            ),
            guest.uuid(),
        );
        assert!(!game.player_disconnect(stale, horizon_client::text::Text::literal("x")));
        assert_eq!(left_reason(game, 12), "");
        let done = game
            .as_guest(12, |game| {
                let host: &mut dyn ServerHost = game;
                guest.disconnect(host, kicked())
            })
            .unwrap();
        assert_eq!(done, Ok(true));
        assert_eq!(left_reason(game, 12), "mymod.kicked");
    });
}

/// The client leaves at the end of the frame, not inside the call, and the disconnect screen
/// shows the reason. With no world joined, nothing happens.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_client_mod_leaves_at_the_end_of_the_frame_with_its_reason() {
    with_client_game(|game| {
        game.title = None;
        let host: &mut dyn ClientHost = game;
        assert_eq!(host.disconnect(Some(kicked())), Ok(false));
        game.native_leave_if_asked().unwrap();
        let (pipe, _server) = horizon_protocol::connection::Pipe::pair();
        game.connect_local(pipe, "Steve").unwrap();
        let host: &mut dyn ClientHost = game;
        assert_eq!(host.disconnect(Some(kicked())), Ok(true));
        assert!(
            game.remote.is_some(),
            "still connected until the frame ends"
        );
        game.native_leave_if_asked().unwrap();
        assert!(game.remote.is_none());
        let mode = &game.title.as_ref().expect("the title screen").mode;
        assert!(
            matches!(
                mode,
                Mode::Multiplayer(horizon_client::multiplayer::Screen::Disconnected { reason, .. })
                    if reason == "mymod.kicked"
            ),
            "the disconnect screen with the reason"
        );
    });
}
