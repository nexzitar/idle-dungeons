# Delvers — Visual Bible (Foundation v1)

## Core visual identity

Delvers should feel like:

**A firelit expedition journal from a doomed but determined dungeon company.**

The visuals are not meant to overwhelm the player with effects or realism.  
They are meant to:

- communicate combat clearly,
- reinforce atmosphere,
- support buildcraft readability,
- and make the dungeon feel ancient, dangerous, and alive.

The game should feel **warm against darkness**, **readable against chaos**, **restrained instead of noisy**, **tactical instead of flashy**.

### Visual pillars

#### 1. Readability first

Every visual decision must support gameplay clarity.

Combat should always answer:

- Who attacked?
- What kind of attack was it?
- Who is in danger?
- What triggered?
- Who holds threat?
- What is happening RIGHT NOW?

The player should never lose information because of particles, bloom, excessive motion, overlapping effects, or visual clutter.

**If readability conflicts with spectacle: readability wins.**

#### 2. Warm firelight vs cold darkness

The entire game should revolve around this contrast.

**Warm tones** represent the party, safety, civilization, the campfire, hope, preparation.

**Cold/dark tones** represent the depths, monsters, forgotten ruins, danger, corruption, pressure.

This contrast should exist everywhere: UI, backgrounds, combat, VFX, menus, progression.

**The campfire is the emotional center of the game.**

#### 3. Stylized painterly minimalism

The art style should **not** chase realism.

Instead: simplified shapes, strong silhouettes, painterly texture, limited detail density, atmospheric shading, readable forms.

The game should look handcrafted, moody, intentional, timeless.

**Avoid:** hyper realism, excessive tiny details, over-rendering, visual noise.

#### 4. Combat rhythm over visual chaos

Combat is cadence-driven.

Visuals should emphasize timing, impact, pressure, attack rhythm, recovery windows, ability bursts. The game should visually communicate **“tempo.”**

- A **Heavy Strike** should feel slower, weightier, committed.
- **Swift Strikes** should feel rapid, fluid, relentless.

This is more important than raw animation count.

---

## Color language

### Primary palette

**Backgrounds:** charcoal black, deep navy, muted brown-black, dark desaturated stone.

**Primary accent:** warm amber, fire gold, muted brass.

**Secondary accent:** desaturated crimson, ember orange, worn copper.

**Text:** parchment, warm ivory, faded gray.

### Combat color language

| Role | Meaning |
|------|---------|
| **White / ivory** | Basic weapon strikes |
| **Gold / amber** | Abilities / empowered attacks |
| **Green** | Healing / sustain / recovery |
| **Purple** | Arcane / corruption / poison mastery |
| **Red** | Danger / enemy attacks / lethal effects |
| **Blue-gray** | Defensive states / barriers / guarded effects |

### Rarity language

Rarity should feel ancient, material, grounded.

**Avoid:** neon MMO rainbow explosions.

| Tier | Feel |
|------|------|
| **Common** | Worn iron / dull leather |
| **Uncommon** | Slight glow / refined material |
| **Rare** | Distinct visual identity begins |
| **Epic** | Noticeable aura / silhouette enhancement |
| **Legendary** | Immediately recognizable from silhouette and lighting alone |

Legendary should feel **mythic**, not colorful.

---

## UI philosophy

### Panels

Panels should resemble **expedition ledgers**, **old reinforced frames**, **dark iron and brass**, **candlelit interfaces**.

**Not:** futuristic windows, glossy MMO panels, clean sci-fi overlays.

### Borders

Thin but deliberate. Warm metallic tones. Subtle texture.

**Avoid:** giant ornate fantasy clutter, overdecorated corners, thick noisy borders.

### Buttons

Buttons should feel **physical**, **tactile**, **weighty**.

- **Hover:** slight illumination, subtle warmth increase, maybe faint ember glow.
- **Pressed:** darker, slightly inset.

### Typography

Typography should feel **readable**, **grounded**, **expeditionary**, **old-world**.

**Avoid:** highly decorative fantasy fonts, difficult-to-read gothic fonts.

**Use:** clean serif or restrained typewriter-inspired fonts, with one slightly stylized display font for titles.

---

## Combat theater philosophy

Combat should resemble **observing a dangerous expedition unfold**.

**Not:** arcade chaos.

The battlefield should prioritize:

- current action,
- timing,
- HP states,
- role clarity,
- status readability.

---

## Animation philosophy

**Style:** restrained, purposeful, timing-driven, silhouette-first.

**Avoid:** endless idle motion, giant screen shake, excessive particles, flashy spam.

### Combat animation priorities

| Priority | Content |
|----------|---------|
| **Highest** | Attacks, impacts, casts, crits, deaths |
| **Medium** | Buffs, poison, barriers, taunts |
| **Low** | Passive procs, ambient motion, cosmetic particles |

---

## Floating combat text rules

Combat text should support readability, never flood the screen, communicate hierarchy.

| Color | Role |
|-------|------|
| **White** | Normal hits |
| **Gold** | Ability hits |
| **Larger + “!”** | Crits |
| **Green** | Healing |
| **Red** | Enemy damage |

Combat text should fade quickly, stagger naturally, avoid overlapping walls of text.

The player should feel **impact**, not **spreadsheet overload**.

---

## Character presentation

Characters should be identifiable by **silhouette**, **posture**, **timing**, **weapon style**, **role stance**—**not** by excessive detail.

Eventually: tanks should look **planted**, swift attackers should look **agile**, support heroes should look **composed**.

---

## The campfire

The campfire is **the soul of Delvers**.

It represents preparation, companionship, survival, continuity, progression.

As the player advances: more heroes appear, gear visually changes, trophies appear, atmosphere evolves.

**The fire should become emotionally familiar.**

---

## Audio philosophy

Sound should reinforce **rhythm**.

Most important: attack cadence, impact timing, heavy vs fast distinction, cooldown readiness, pressure escalation.

Music should support tension, isolation, descent, survival.

**Avoid:** over-orchestrated bombast.

---

## Long-term visual goal

The final visual impression should feel like:

> *“Watching a desperate but disciplined dungeon expedition illuminated by firelight in ancient darkness.”*

The player should feel **tension**, **mastery**, **momentum**, **danger**, **rhythm**, **progression**, **attachment to their party**—not sensory overload.

---

## Visual anti-goals

Delvers should **not** become:

- VFX spam,
- neon fantasy,
- noisy MMO UI,
- overanimated chaos,
- hyperrealistic grimdark,
- meme-heavy idle clutter,
- mobile-game reward explosion design.

The game’s strength is **tactical readability** and **emotional atmosphere through restraint**.

---

## UI implementation

For spacing, panel recipes, inspect regions, tooltip policy, density zones, and primitive catalog, see **[`ui-design-system.md`](ui-design-system.md)**. The Party Buildcraft sheet is the reference implementation.
