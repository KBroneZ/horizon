//! `LevelRef`'s fluid heights, replacement, tick and tick delay, and the fluid facts service,
//! against a level the test builds (#969).
use std::sync::Arc;

use horizon_client::chunk::Chunk;
use horizon_core::geometry::Direction;
use mod_api::{
    FluidFacts, FluidKey, Host, HostFluidStates, LevelRef, NativeWorldEpoch, ParticleType,
};

use super::native_entity_refs::tests::with_game;

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn fluid_heights_and_replacement_read_the_level_and_refuse_what_they_cannot_see() {
    with_game(|game| {
        let state = |name: &str| game.block_state_id(name).unwrap();
        let (source, shallow) = (state("minecraft:water"), state("minecraft:water[level=5]"));
        let (lava, thin_lava) = (state("minecraft:lava"), state("minecraft:lava[level=6]"));
        let stone = state("minecraft:stone");
        let mut chunk = Chunk::empty(0, 0);
        // A source under stone, shallow water under air, shallow water under a source, a lava
        // source, thin lava (2/9 high) and an empty block.
        for (x, at, above) in [
            (0, source, Some(stone)),
            (1, shallow, None),
            (2, shallow, Some(source)),
            (3, lava, None),
            (4, thin_lava, None),
        ] {
            chunk.set(x, 64, 0, at.0);
            if let Some(above) = above {
                chunk.set(x, 65, 0, above.0);
            }
        }
        game.world.chunks.insert((0, 0), Arc::new(chunk));
        let level = game.level(game.dim.id()).unwrap();
        let stale = LevelRef::new(
            NativeWorldEpoch::new(level.epoch().get() + 1),
            level.dimension(),
        );
        let host: &dyn Host = game;
        let key = |id| host.fluid_key(id).unwrap();
        let (water, lava_key) = (key("minecraft:water"), key("minecraft:flowing_lava"));
        let tank_only = key("horizon:definition_test");

        // FluidState.getHeight: own height (amount / 9) under anything but the same fluid.
        assert_eq!(level.fluid_height(host, [0, 64, 0]), Some(8.0 / 9.0));
        assert_eq!(level.fluid_height(host, [1, 64, 0]), Some(3.0 / 9.0));
        assert_eq!(level.fluid_height(host, [2, 64, 0]), Some(1.0));
        assert_eq!(level.fluid_height(host, [5, 64, 0]), Some(0.0));
        assert_eq!(
            level.fluid_shape(host, [1, 64, 0]),
            Some(vec![[0.0, 0.0, 0.0, 1.0, f64::from(3.0f32 / 9.0), 1.0]])
        );
        assert_eq!(level.fluid_shape(host, [5, 64, 0]), Some(Vec::new()));
        // getHeightForCamera: a source under a sturdy bottom face fills the block.
        assert_eq!(level.fluid_height_for_camera(host, [0, 64, 0]), Some(1.0));
        assert_eq!(
            level.fluid_height_for_camera(host, [1, 64, 0]),
            Some(3.0 / 9.0)
        );
        assert_eq!(
            level.fluid_height_for_camera(host, [3, 64, 0]),
            Some(8.0 / 9.0)
        );

        let replaced = |pos, fluid, direction| {
            level.fluid_can_be_replaced_with(host, pos, fluid, direction)
        };
        assert_eq!(replaced([0, 64, 0], lava_key, Direction::Down), Some(true));
        assert_eq!(replaced([0, 64, 0], lava_key, Direction::North), Some(false));
        assert_eq!(replaced([1, 64, 0], water, Direction::Down), Some(false));
        assert_eq!(replaced([3, 64, 0], water, Direction::Up), Some(true));
        assert_eq!(
            replaced([4, 64, 0], water, Direction::Up),
            Some(false),
            "lava under 4/9 high"
        );
        assert_eq!(replaced([5, 64, 0], lava_key, Direction::East), Some(true));
        assert_eq!(replaced([5, 64, 0], tank_only, Direction::Down), None);

        for pos in [[16, 64, 0], [0, 4000, 0]] {
            assert_eq!(level.fluid_height(host, pos), None);
            assert_eq!(level.fluid_height_for_camera(host, pos), None);
            assert_eq!(replaced(pos, water, Direction::Down), None);
        }
        assert_eq!(stale.fluid_height(host, [0, 64, 0]), None);
    });
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_fluid_tick_runs_now_and_its_delay_follows_the_level() {
    with_game(|game| {
        let trickle = game.block_state_id("minecraft:water[level=7]").unwrap();
        let air = game.block_state_id("minecraft:air").unwrap();
        let mut chunk = Chunk::empty(0, 0);
        chunk.set(8, 64, 8, trickle.0);
        game.world.chunks.insert((0, 0), Arc::new(chunk));
        let level = game.level(game.dim.id()).unwrap();
        let stale = LevelRef::new(
            NativeWorldEpoch::new(level.epoch().get() + 1),
            level.dimension(),
        );
        let nether = LevelRef::new(level.epoch(), mod_api::DimensionKey::NETHER);
        let key = |game: &crate::game::Game<'_>, id| game.fluid_key(id).unwrap();
        let (water, lava, empty) = (
            key(game, "minecraft:flowing_water"),
            key(game, "minecraft:lava"),
            key(game, "minecraft:empty"),
        );
        let tank_only = key(game, "horizon:definition_test");

        // Fluid.getTickDelay in the Overworld, where lava is slow.
        let delay = |game: &mut crate::game::Game<'_>, level: LevelRef, fluid: FluidKey| {
            level.fluid_tick_delay(game, [8, 64, 8], fluid)
        };
        assert_eq!(delay(game, level, water), Some(5));
        assert_eq!(delay(game, level, lava), Some(30));
        assert_eq!(delay(game, level, empty), Some(0));
        assert_eq!(delay(game, level, tank_only), None);
        assert_eq!(delay(game, stale, water), None);
        assert_eq!(delay(game, nether, lava), None, "a level not in place");

        // FluidState.tick: flowing water nothing feeds dries up at once.
        assert!(level.fluid_tick(game, [8, 64, 8]));
        assert_eq!(game.world.block(8, 64, 8), air);
        assert!(level.fluid_tick(game, [8, 64, 8]), "no fluid: nothing to do");
        assert!(!level.fluid_tick(game, [16, 64, 8]), "unloaded");
        assert!(!stale.fluid_tick(game, [8, 64, 8]));
        assert!(!nether.fluid_tick(game, [8, 64, 8]));
    });
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn fluid_facts_and_legacy_blocks_follow_vanilla() {
    with_game(|game| {
        let state = |name: &str| game.block_state_id(name).unwrap();
        let (source, falling) = (state("minecraft:water"), state("minecraft:water[level=8]"));
        let shallow = state("minecraft:lava[level=5]");
        let slab = state("minecraft:oak_slab[waterlogged=true]");
        let air = state("minecraft:air");
        let fluid_of = |state| game.state_fluid_state(state).unwrap();
        let host: &dyn Host = game;
        let service = host.fluid_states();
        let facts = |id| service.facts(host.fluid_key(id).unwrap()).unwrap();

        let water = FluidFacts {
            explosion_resistance: 100.0,
            randomly_ticks: false,
            drip_particle: Some(ParticleType::DrippingWater),
        };
        let inert = FluidFacts {
            explosion_resistance: 0.0,
            randomly_ticks: false,
            drip_particle: None,
        };
        assert_eq!(facts("minecraft:water"), water);
        assert_eq!(facts("minecraft:flowing_water"), water);
        for lava in ["minecraft:lava", "minecraft:flowing_lava"] {
            assert_eq!(
                facts(lava),
                FluidFacts {
                    explosion_resistance: 100.0,
                    randomly_ticks: true,
                    drip_particle: Some(ParticleType::DrippingLava),
                }
            );
        }
        assert_eq!(facts("minecraft:empty"), inert);
        assert_eq!(facts("horizon:definition_test"), inert);

        // createLegacyBlock: the liquid block at the state's level, never the block it came from.
        for liquid in [source, falling, shallow] {
            assert_eq!(service.legacy_block(fluid_of(liquid)), Some(liquid));
        }
        assert_eq!(service.legacy_block(fluid_of(slab)), Some(source));
        assert_eq!(service.legacy_block(fluid_of(air)), Some(air));
        let mut drained = fluid_of(source);
        drained.amount = 3;
        assert_eq!(service.legacy_block(drained), None, "a source is full");
        let mut tank = fluid_of(source);
        tank.fluid = host.fluid_key("horizon:definition_test").unwrap();
        assert_eq!(service.legacy_block(tank), None);
    });
}
