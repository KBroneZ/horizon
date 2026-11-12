# Block experience and explosions

Attach a `BlockPart` to the blocks that own the behavior.

`exp_drop(level, event)` answers `Some(points)` to replace the vanilla experience
sample, before enchantments. `None` leaves the answer to another attached part
and then vanilla. The query receives the dimension, state, position, optional
block entity, breaker entity handle and tool. The level host reads without writing;
`level.block_entity_data(entity)` reads the supplied entity's saved data without
identity metadata and refuses stale handles. Answers are clamped to 0 through
32,767. Player breaks supply the part's answer to the global `break_block`
event's `exp_to_drop`, after Silk Touch suppresses the starting amount (except
for spawners). An explicit global override is applied as given. `block_drops` sees the resulting amount afterwards.
Level and mob breaks use the shared resource-drop path. A canceled player break
or a break that does not harvest produces no award.

`block_drops(level, event)` hears the drops after that. Besides the breaking
player's UUID (`breaker`) and the `tool`, it carries `breaker_entity`, the
entity that broke the block (the player, or the mob whose own break of it is
running: a ravager's), and `block_entity`, the block's block entity.
`breaker_entity` is not set when no entity broke the block (a piston, water, a
neighbour update, `/setblock destroy`) or for explosions. `block_entity` is set
whenever the block has one at that moment, whoever broke it, and not for
explosions, whose blocks are gone before the event. A player's break removes the
block entity from the world first, so its saved data, taken before the removal
(its `CustomName` too: copy a chest's name onto its drop), is held through the
event: `level.block_entity_data(entity)` reads it and, as with `exp_drop`, a
stale handle is refused. The data is released when the event ends, also if a
listener panics; keep nothing from it but what you copied.

`on_block_exploded(host, event)` runs before the engine removes each destroyed
block. The server host permits writes. `event.block_entity` is a live handle,
and `entity.data(host)` can read it while the block remains. The event borrows
the same `ExplosionStart` description as the global start event: dimension,
center, radius, fire, interaction, damage type, source and owner.

The engine still writes air after the callback, including when the callback
replaced the block. Vanilla's separate `wasExploded` behavior follows removal.
Air, spared blocks, canceled explosions and explosions that keep blocks produce
no callback. Explosions retain their existing zero initial XP behavior.

Both methods route through prepared per-block subscriptions. Unsubscribed
blocks build no callback context. Callback panics follow the existing native
hook unwinding behavior; the engine does not apply a returned XP answer after
a panic.
