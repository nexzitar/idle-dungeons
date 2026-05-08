# Itemization & save JSON migration notes

## `ItemRarity` / `ItemAffix` variants

Save files store `profile.json` (`SaveProfile` in `src/save.rs`) including `ItemInstance` payloads under the hero and inventory.

- **Adding new enum variants** (e.g. `Epic`, `Legendary`, `Devourer`, `TitansFury`) is **backward compatible** for readers: older saves only serialized earlier variants; serde deserializes them as before.
- **Removing or renaming** a serde variant breaks old saves unless you add custom deserialization or a one-time migration layer. Prefer **append-only** variant names in Rust; if you must rename, keep the JSON name stable with `#[serde(rename = "OldName")]`.
- **Defaulting missing fields** on `ItemInstance` is not used today; new optional fields on structs should use `#[serde(default)]` if you need older files to load without the key.

## Legendary-only affixes

`Devourer` and `TitansFury` are **loot-only** for normal play: the legendary weighted table in `src/domain/loot.rs` is the only place they are rolled. If a save is hand-edited or migrated, equipping them on a hero still works; combat reads them via `has_affix`.

## Verification after domain changes

1. Run `cargo test` (covers round-trip save tests and combat).
2. Keep a **fixture** save from the previous build if you change serialization shape; load it once after edits.
