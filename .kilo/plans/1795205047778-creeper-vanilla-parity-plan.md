# Creeper Vanilla Parity Implementation Plan

## Goal
Implement full vanilla Java Edition parity for the Creeper entity in PumpkinMC (SSIT2051/Pumpkin fork) based on [Issue #4252](https://github.com/Pumpkin-MC/Pumpkin/issues/4252), using Horizon's implementation as reference. Ensure crossplay compatibility (Java + Bedrock) and deploy via GitHub Actions to test VM.

---

## Context Summary

### Current State
- **PumpkinMC Core** (`pumpkin-core` crate): Has `creeper.rs` with partial implementation
- **Horizon** (`horizon-client/src/mobs/hostile.rs`): Complete vanilla-parity Creeper implementation with 281 differential harnesses
- **pumpkin-unified** (plugin): Uses `/spawnmob creeper` command via plugin API; needs updated WASM build
- **VM**: Runs PumpkinMC server at `/root/pumpkin-server`, plugins at `/root/pumpkin-server/plugins/pumpkin_unified_core.wasm`

### Key References
- **Issue #4252**: Vanilla parity tracker for Creeper
- **Horizon hostile.rs**: Reference implementation
- **PumpkinMC Entity AI Docs**: Goal system architecture
- **PumpkinMC Plugin API**: WASM plugin deployment

---

## Gap Analysis: Current vs Required

### ✅ Already Implemented in PumpkinMC (`crates/pumpkin-core/src/entity/mob/creeper.rs`)

| Feature | Status | Notes |
|---------|--------|-------|
| Entity struct with fuse/charged/ignited fields | ✅ | `fuse_speed`, `current_fuse_time`, `last_fuse_time`, `fuse_time`, `explosion_radius`, `ignited`, `charged` |
| SwimGoal (priority 1) | ✅ | |
| CreeperIgniteGoal (priority 2) | ✅ | Similar to vanilla's SwellGoal; handles fuse logic |
| Flee from Ocelot/Cat (priority 3) | ✅ | Uses `Flee` behavior with `TargetMatch::Type` |
| MeleeAttackGoal (priority 4) | ✅ | Speed 1.0, no memory |
| WanderAroundGoal (priority 5) | ✅ | Speed 0.8 |
| LookAtEntityGoal + RandomLookAroundGoal (priority 6) | ✅ | |
| Hunt Player target goal (priority 1) | ✅ | `mustSee=true` |
| RevengeGoal target goal (priority 2) | ✅ | |
| NBT read/write: powered, Fuse, ExplosionRadius, ignited | ✅ | |
| Fall damage increases swell | ✅ | In `cause_fall_damage` |
| Lightning strike charges creeper | ✅ | In `mob_on_lightning_strike` |
| Flint and steel ignition | ✅ | In `mob_interact` |
| Explosion with charged multiplier (2x) | ✅ | In `explode()` |
| Synced data for Bedrock: CHARGED, IS_IGNITED, FUSE_ID | ✅ | |
| Sound: CreeperPrimed when fuse starts | ✅ | In `mob_tick` |

### ❌ Missing per Issue #4252

| Feature | Priority | Location |
|---------|----------|----------|
| **SwellGoal** (vanilla name; current is CreeperIgniteGoal) | High | `creeper.rs` - rename/refactor goal |
| **AvoidEntityGoal** for Ocelot/Cat (not Flee) | High | `creeper.rs` - replace Flee with AvoidEntityGoal |
| **WaterAvoidingRandomStrollGoal** (not WanderAroundGoal) | Medium | `creeper.rs` - use `WanderAroundGoal::water_avoiding(0.8)` |
| **spawnLingeringCloud()** for area effect cloud | High | `creeper.rs` - TODO in explode() |
| **killedEntity** - charged creeper drops victim skull | High | `creeper.rs` - not implemented |
| **mobInteract** with fire_charge | Medium | `creeper.rs` - only flint_and_steel |
| **PRIME_FUSE** game event | Medium | `creeper.rs` - not sent; **GameEvent enum missing PRIME_FUSE** |
| **CREEPER_HURT** sound | Medium | `creeper.rs` - not played |
| **CREEPER_DEATH** sound | Medium | `creeper.rs` - not played |
| **ExplosionRadius** from NBT used in explosion | High | `creeper.rs` - uses default 3 only |
| **getSwelling(partialTick)** for client rendering | Medium | `creeper.rs` - not exposed |
| **Loot tables**: entities/creeper, charged_creeper | High | `pumpkin-data` / loot table JSON |
| **Spawn egg** registration | High | `pumpkin-data` (generated) |
| **Natural spawning rules**: biome weights, sulfur_caves | High | `pumpkin-data` (generated) |
| **Despawn rules**: removeWhenFarAway, distances 128/32 | High | `pumpkin-data` (generated) |
| **Size**: 0.6 x 1.7, clientTrackingRange(8) | High | `pumpkin-data` (generated) |
| **Attributes**: MAX_HEALTH 20, FOLLOW_RANGE 16, ATTACK_DAMAGE 2.0, MOVEMENT_SPEED 0.25 | High | `pumpkin-data` (generated) |

---

## Implementation Phases

### Phase 1: Audit & Data Verification (pumpkin-data)
- [ ] Verify entity type registration in `pumpkin-data` generated code includes:
  - `EntityType::CREEPER` with correct dimensions (0.6 x 1.7)
  - `MobCategory::MONSTER`, `notInPeaceful()`
  - `clientTrackingRange(8)`
  - Spawn egg: `Items.CREEPER_SPAWN_EGG`
  - Natural spawning: `ON_GROUND`, `MOTION_BLOCKING_NO_LEAVES`, `Monster::checkMonsterSpawnRules`
  - Biome weights: overworld monster biomes (weight 100, count 4), `sulfur_caves` (weight 50, count 2)
  - Despawn: `removeWhenFarAway=true`, distances 128/32
- [ ] Verify attributes in `pumpkin-data`:
  - MAX_HEALTH 20, FOLLOW_RANGE 16, ATTACK_DAMAGE 2.0, MOVEMENT_SPEED 0.25
- [ ] If missing, add to extractor/assets and regenerate via `pumpkin-codegen`

### Phase 2: Core Creeper Logic (pumpkin-core)

#### 2.1 AI Goals Refactor
- [ ] Rename `CreeperIgniteGoal` → `SwellGoal` (or create new `SwellGoal` matching vanilla exactly)
- [ ] Replace `Flee` with `AvoidEntityGoal` for Ocelot/Cat (params: flee_distance=6.0, slow_speed=1.0, fast_speed=1.2)
- [ ] Replace `WanderAroundGoal(0.8)` with `WanderAroundGoal::water_avoiding(0.8)`

#### 2.2 SwellGoal Enhancements
- [ ] Add `maxSwell` constant (default 30)
- [ ] Play `CREEPER_PRIMED` sound (vol 1.0, pitch 0.5) when swell goes from 0→>0
- [ ] Emit `PRIME_FUSE` game event when swell starts (requires adding to GameEvent enum)
- [ ] Implement `getSwelling(partialTick)` = lerp(oldSwell, swell) / (maxSwell - 2)
- [ ] Move explosion logic from `mob_tick` into `SwellGoal::tick` when swell >= maxSwell

#### 2.3 GameEvent Enum Extension (pumpkin-protocol)
- [ ] Add `PrimeFuse` variant to `GameEvent` enum in `crates/pumpkin-protocol/src/java/client/play/game_event.rs`
- [ ] Ensure `CGameEvent::new(GameEvent::PrimeFuse, 0.0)` works for PRIME_FUSE emission

#### 2.4 Explosion & Lingering Cloud
- [ ] Use `ExplosionInteraction::Mob` (exists in `crates/pumpkin-core/src/entity/tnt.rs`)
- [ ] Use `explosion_radius` from NBT (not hardcoded default)
- [ ] Implement `spawnLingeringCloud()` creating `AreaEffectCloud` entity:
  - Radius 2.5, radiusOnUse -0.5, waitTime 10, duration 300
  - potionDurationScale 0.25, radiusPerTick -radius/duration
  - Copies all active effects from creeper
- [ ] `AreaEffectCloud` entity EXISTS ✅ at `crates/pumpkin-core/src/entity/area_effect_cloud.rs`

#### 2.5 Charged Creeper Skull Drop
- [ ] Implement `killedEntity` hook: if powered and not `droppedSkulls`, drop `CHARGED_CREEPER` loot on victim

#### 2.6 Interactions
- [ ] Add `fire_charge` to `mob_interact` (check `ItemTags.CREEPER_IGNITERS`)
- [ ] Play `FIRECHARGE_USE` sound (vol 1.0, pitch random*0.4+0.8)

#### 2.7 Sounds
- [ ] Play `CREEPER_HURT` on damage
- [ ] Play `CREEPER_DEATH` on death (non-explosion)

#### 2.8 NBT & Metadata
- [ ] Ensure `ExplosionRadius` from NBT is used in explosion calculation
- [ ] Verify synced data IDs match Bedrock (CHARGED, IS_IGNITED, FUSE_ID)

### Phase 3: Crossplay Compatibility (Bedrock + Java)
- [ ] Verify Bedrock entity ID mapping for Creeper
- [ ] Ensure synced entity data works for both protocols
- [ ] Test explosion particles/sounds on Bedrock clients
- [ ] Verify spawn egg works in Bedrock creative inventory
- [ ] Test `getSwelling` data sync for Bedrock client rendering

### Phase 4: Update pumpkin-unified Plugin
- [ ] Verify `/spawnmob creeper` command works with new implementation
- [ ] Update any hardcoded entity type mappings if needed
- [ ] Ensure permission `pumpkin-unified-core:essentials.spawnmob` works

### Phase 5: CI/CD Pipeline (GitHub Actions)
- [ ] Configure `.github/workflows/wasm-build.yml` for `wasm32-wasip2` build
- [ ] Build `pumpkin-unified-core` crate as WASM artifact
- [ ] Upload artifact to `/tmp/artifact/pumpkin_unified_core.wasm`
- [ ] Trigger on push to SSIT2051/Pumpkin fork

### Phase 6: VM Deployment & Testing
- [ ] SSH to `pumpkin-vm` and verify connectivity
- [ ] SCP WASM plugin to `/root/pumpkin-server/plugins/`
- [ ] Restart PumpkinMC server cleanly
- [ ] Inspect `/tmp/pumpkin.log` for errors
- [ ] Test in-game: `/spawnmob creeper`, verify behavior (swell, explode, charged, drops)
- [ ] Run `safe-disk-guard.sh` and verify disk usage < 8.5GB

---

## Technical Considerations

### Architecture Constraints
- **No local `cargo build`** - All compilation via GitHub Actions
- **WASI 0.2** (`wasm32-wasip2` target) for plugin
- **Permission prefix**: `pumpkin-unified-core.` for all registered permissions
- **Crossplay**: Must work for both Java and Bedrock clients

### Key Files to Modify (PumpkinMC Core)
```
crates/pumpkin-core/src/entity/mob/creeper.rs                      # Main implementation
crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs             # Already exists, use it
crates/pumpkin-core/src/entity/ai/goal/wander_around.rs            # Already has water_avoiding()
crates/pumpkin-core/src/entity/area_effect_cloud.rs                # EXISTS - use for lingering cloud
crates/pumpkin-protocol/src/java/client/play/game_event.rs         # Add PRIME_FUSE to GameEvent enum
crates/pumpkin-data/...                                            # Regenerated via pumpkin-codegen
```

### Key Files to Modify (Plugin)
```
pumpkin-unified-core/src/utility/mod.rs                # spawn_mob mapping (verify only)
.github/workflows/wasm-build.yml                       # CI pipeline
```

---

## Validation Plan

### Unit Tests
- [ ] Creeper NBT round-trip (powered, Fuse, ExplosionRadius, ignited)
- [ ] SwellGoal state transitions (idle → swelling → exploding)
- [ ] Explosion radius calculation (normal vs powered vs custom ExplosionRadius)
- [ ] Charged creeper skull drop on kill
- [ ] `getSwelling(partialTick)` returns correct interpolation
- [ ] GameEvent::PrimeFuse serializes correctly in CGameEvent

### Integration Tests
- [ ] Spawn creeper via `/summon` and `/spawnmob`
- [ ] Verify natural spawning in correct biomes
- [ ] Test flint_and_steel / fire_charge ignition
- [ ] Test lightning charging
- [ ] Test cat/ocelot avoidance (uses AvoidEntityGoal)
- [ ] Test fall damage increases swell
- [ ] Test explosion particles/sounds on Java + Bedrock clients
- [ ] Test charged creeper kills drop victim head
- [ ] Test lingering cloud spawns with potion effects
- [ ] Test PRIME_FUSE game event emitted on swell start

### Performance
- [ ] Server tick time with multiple creepers
- [ ] Memory usage with entity tracking

---

## Rollback Plan
- Git revert on SSIT2051/Pumpkin fork
- Redeploy previous WASM plugin version to VM
- VM server restart with previous binary

---

## Resolved Questions

1. **AvoidEntityGoal exists?** ✅ Yes - `crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs`
2. **WaterAvoidingRandomStrollGoal?** ✅ Use `WanderAroundGoal::water_avoiding(0.8)` 
3. **ExplosionInteraction.MOB?** ✅ Used in TNT entity (`crates/pumpkin-core/src/entity/tnt.rs`)
4. **AreaEffectCloud entity?** ✅ EXISTS at `crates/pumpkin-core/src/entity/area_effect_cloud.rs`
5. **Loot table format?** JSON in `assets/datapacks/` consumed by `pumpkin-codegen`
6. **PRIME_FUSE game event?** ❌ MISSING - GameEvent enum in pumpkin-protocol only has 13 variants; must add `PrimeFuse`
7. **Horizon hostile.rs access?** Clone locally or use GitHub API for specific lines

---

## Next Steps
1. Clone SSIT2051/Pumpkin locally to inspect `creeper.rs` and pumpkin-data generated code
2. Begin Phase 2 implementation (core logic) starting with:
   - Add `PrimeFuse` to GameEvent enum
   - Refactor CreeperIgniteGoal → SwellGoal
   - Replace Flee with AvoidEntityGoal
   - Implement spawnLingeringCloud using AreaEffectCloud
3. Set up GitHub Actions workflow for WASM build
4. Test on VM

---

## References
- [Issue #4252](https://github.com/Pumpkin-MC/Pumpkin/issues/4252) - Vanilla parity tracker
- [Horizon hostile.rs](https://github.com/horizon-rust/horizon/blob/main/crates/horizon-client/src/mobs/hostile.rs) - Reference impl
- [PumpkinMC Entity AI Docs](https://docs.pumpkinmc.org/developer/entities/ai-and-mobs)
- [PumpkinMC Plugin Dev](https://docs.pumpkinmc.org/plugin-dev/introduction)
- [PumpkinMC Metadata Docs](https://docs.pumpkinmc.org/developer/networking/metadata)