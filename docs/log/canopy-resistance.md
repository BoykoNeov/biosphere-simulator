## **A canopy surface resistance that reads leaf area** (opened from Step 6's water-gap split — the frozen 70 s/m ignores the canopy; two cited forms, measured in the lab before any decision)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-07** on the user's call (*"Leaf-area resistance"*), after Step 6 found the crop's
surface resistance is a constant 70 s/m whatever its leaf area. The order the user set: a source
first, a lab measurement, then the user's decision. Plan: `docs/plans/post-roadmap-canopy-resistance.md`.
Nothing frozen or adopted.

* **FAO-56** (Allen et al. 1998), Chapter 2, read from the raw page: Eq. 5, `rs = rl / LAI_active`,
  is *"an acceptable approximation … of dense full cover vegetation"*; partial cover should include
  soil evaporation. The 0.5 active-leaf factor and the ~100 s/m leaf value are stated in the grass
  reference box. A summarising fetch had reported "no caveats"; the raw page has them.
* **Teh**, Eq. 4.80 after Szeicz & Long (1969), on the shelf: `rc = rst / L` up to half a threshold
  LAI, constant above it; Teh takes the threshold as 4.0. His `rst` is light-dependent (Jarvis),
  with no wheat coefficients on the shelf.
* The two forms differ by a factor of 2 below the plateau and diverge at full cover (32 against 50
  s/m at LAI 6.2). Only Teh's addresses a seedling; plugging FAO's 100 into it is a composite.
* Measured first: the sealed chambers' LAI never passes 1.02, so either form raises their resistance
  on every step; the open-field crop sits below LAI 0.5 for 41 % of its season.
* Ten predictions written before any code.

**BUILT lab-only and measured 2026-10-07** (plan §7). Two forms behind a lab switch; the loader
keeps the frozen constant, and the frozen path is the old code path (asserted bit-identical).

* Each form reaches the water flow to 1e-12; forcing it back to constant turns that check red.
* Water moves, carbon does not, until drought. The seedling barely drinks (winter use −82 % FAO,
  −68 % Szeicz–Long); the closed canopy drinks more (+20 % / +11 %); the season −5 %.
* The deep-water rescue collapses to ~1 (grain 1.036 / 1.089 against the pinned 1.22): it was
  mostly the seedling drought the constant resistance invents. A summer drought now bites sooner.
* The sealed chambers drink 29–65 % less; their canopies never close.
* TM 102788's early water curve fits Szeicz–Long closely (0.17 / 0.40 / 0.74 / 1.06 against
  0.21 / 0.47 / 0.71 / 1.18 on days 5–14); FAO rises too slowly; the constant is flat. One trial.
* Neither closes the trial's level (2.6–2.9 against ~6). Of 10 predictions 7 held, 2 in part, 1
  failed: drainage stayed 0 (irrigation is deficit-driven), and the dark share was 5.7 %, not 8 %.
* Decisions owed to the user: which form to price for adoption, a constant or light-dependent leaf
  resistance, and the threshold LAI.
