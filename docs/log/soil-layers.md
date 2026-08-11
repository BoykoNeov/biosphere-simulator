## **Soil layers — the water side of root depth**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-11**, the successor the root-depth build named, and the first in that series
to **move a value**. ⚠ **It was priced as "the largest single piece the post-roadmap record
has considered" on a false premise.** That price assumed "layers" meant an N-layer
discretization; [F] Soltani & Sinclair opens its own soil-water chapter by settling the
resolution question — *"for models attempting to simulate crop growth and yield as is the
objective of this book, a **two-layered soil or even a one-layer soil seems satisfactory**
(Robertson and Fukai, 1994)"* — and names the two stores as the root zone and the water
below it. So the expensive design was refuted by reading one more sentence of a source
already on the shelf. **What shipped:** a `subsoil_water` POOL (`WSTORG`) holding
extractable water that is physically present below the rooted depth and unreachable, and
`RootZoneCapture` (`EWAT`, Eqn 14.10) moving it into `soil_water` as the roots arrive,
driven by the **same gated rate** the depth accumulator uses (one shared function — a
capture computed from an ungated rate would move water for depth the roots never gained).
Three cited additions rode along: the soil's own rooting cap `SOLDEP` (**discharging a
ceiling `root_depth.yaml` had itself recorded as deferred**), `EXTR = 0.13` ([F] Ch. 13),
and a **cited sowing rooting depth replacing an uncited 0.0** ([F] Ch. 14, *"normally
between 150 to 400 mm"*). **THE DIFF WAS PREDICTED IN THE PLAN DOC BEFORE REGENERATION AND
THE PREDICTION HELD**: `soil_water` up by the season's capture, `subsoil_water` down by the
same, `f_water` exactly 1.0 throughout, therefore **every carbon / nitrogen / oxygen stock
bit-identical at every horizon** — checked against all 25 goldens on disk (not the
manifest's 7; that roster error bit the previous build twice). 12 goldens changed, both
drift summaries byte-identical, and the shift is `149.5806 kg` against the geometric
`(1.30062 − 0.15) × 0.13 × 1000 = 149.58`. The 15-year runs land on the **same**
`soil_water` as the 5-year runs, which verifies the re-sow return (ours; [F] is
single-season and silent) as a closed cycle rather than a ratchet. **MEASURED EFFECT**,
against a control that removes only the water (`EXTR = 0`, so rooted depth still grows): the
deep-water crop peaks at **8.8398** leaf C and sets 3.69 grain against **3.5345** and
**0.0000** — a 2.5× canopy and the difference between setting grain and none, with the peak
matching the *fully irrigated* reference season. ⚠ The naive control (`subsoil = 0`) gives
the same numbers but does **not** license the claim: it also freezes rooted depth, hence the
nitrogen gate. The two controls agree stock-for-stock except `soil_n` at **1 ULP**, which is
what says the effect is water. **THREE FINDINGS NOT IN THE PLAN:** (1) two cited mechanisms
composed into a trap — with the old uncited `rooted_depth = 0`, [F]'s `WSTORG = 0 ⇒ GRTD =
0` freezes depth, makes `FROOT1 = 0`, and drives nitrogen uptake to **identically zero**;
the escape was also a citation. (2) The default profile does not weaken the drought cascade,
it **abolishes** it (`f_water` never leaves 1.0; end veg C 33.61 → 33.28 instead of 33.61 →
**12.68**), so `water_biting` and `drought` declare dry subsoils with the number recorded
rather than the decision asserted — a crop that can root into wet subsoil is
drought-defended, which is the mechanism working. (3) A test helper was **re-listing** the
aux keys instead of copying them, silently omitting `rooted_depth` — the very bug that
file's docstring warns about, moved one level up. **Cascade:** `flow_set` 20 → 21,
`param_files` unchanged (this is scenario/soil data), 12 goldens, both manifests, the Rust
mirror, 14 mutation-verified Python pins against 7 broken variants. ⚠ **A fourth finding,
from checking the port rather than trusting it:** `cargo test` green is not parity, and the
Rust mirror was measured **blind to its own transcription** — dropping the donor clamp, the
dry-subsoil stop, the soil cap, the re-sow return's area factor, or the capture's area
factor each left the *entire* Rust suite green, because no Rust scenario has a plot other
than 1 m² or a store that empties. Four Rust pins now construct those conditions. A flagged
hazard that did **not** survive checking is recorded too: a transposed `EXTR`/`ground_area`
argument is not a bug at all — they are symmetric factors of a product, so the transposition
is arithmetically identical for every input, and testing for it would pin nothing. **NOT
built, each now a named successor:** the `FTSW = ATSW/TTSW` stress conversion;
drainage/runoff/soil evaporation; and the real finding — **re-deriving `soil_water0` from
geometry**, because our root-zone bucket is not dimensionally a soil profile (1000 kg over 1
m² is 1000 mm of extractable water, needing a 7.7 m column at `EXTR = 0.13`), and deriving
it would collapse the store to ~19.5 kg at sowing and make **every frozen scenario
water-stressed**. That is a re-basing of the water regime, not an addition to it, and the
verdict is the user's. `docs/plans/post-roadmap-soil-layers.md`
