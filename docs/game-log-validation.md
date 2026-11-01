# Current Game.log witness validation

> **Doc type:** Living reference — partial evidence; paired timing measured, mapping coverage remains open. Last reviewed: 2026-11-01.

This is the dated result of the local current-LIVE review for issue #107, R-OCR-20, A24 and O-101. It records observations, not a production format or mapping registry. The witness remains opt-in and default off; unknown formats and mappings supply no automatic resolving evidence. The implementation and its privacy tests belong to issue #108.

## Evidence boundary

Reviewed 2026-11-01: the user-authorised LIVE installation manifest and a current-build backup containing kiosk records. The manifest reports build `4.10.197.2878`, branch `sc-alpha-4.10.2`. Its `Data.Version` and `Data.Branch` are the relevant allowlisted fields. The current log and recent backups were surveyed locally; the initial 370-record survey below comes from one backup. The follow-up paired-capture review adds two backups from 2026-10-14 with 411 and 175 valid shop rows; both report the same build and branch. These row counts describe separate reviews and are not counts of distinct kiosk openings.

Raw logs remain local. No raw line, player handle, player or session identifier, endpoint, balance, personal path or complete reference-data dump is included here. Field names below describe the format; all example values in the synthetic specimens are invented. Reading and summarising selected fields is not proof that an eventual parser drops private fields: that requires the #108 privacy test through `GameLogSource`.

## Current format observations

Of 385 records containing `AddingCommodityBox`, 370 also contain `CEntityComponentCommodityUIProvider::LoadShopInventoryData` and `Available Box Sizes`. All 370 have a UTC timestamp shaped `<YYYY-MM-DDTHH:MM:SS.mmmZ>` and `[Notice]` severity. Their fields include `playerId[]`, `shopId[]`, `shopName[]`, `commodityName[ResourceType.<internal name>]` and repeated `boxSize[]` values. The source names the load of shop inventory; it is not a scroll event or proof of sell-side assortment.

The other 15 records come from `CEntityComponentCommodityUIProvider::AddPlayerCommodityItem` and lack `Available Box Sizes`. They describe player inventory, so matching `AddingCommodityBox` alone would admit the wrong source. They must never establish kiosk assortment or sell-side facts.

The valid records contain these size sets: `{8,16,24,32}`, `{16,24,32}`, `{1,2,4,8,16,24,32}`, `{24,32}` and `{4,8,16,24,32}`. No stock, quantity, `scu_buy`, `scu_sell` or price field was found on these 370 records. This confirms the current shape of the buy-side witness; the historical screen/UEX comparison in A24 remains the evidence for side interpretation and size agreement. Corrected 2026-11-01: the follow-up below compares selected current-version paired captures and size sets. No new current UEX per-row size comparison was performed.

Location records retain the historical `<RequestLocationInventory>` shape: `Player[] requested inventory for Location[]`. Current observations include `RR_P6_LEO`, `RR_P3_L3` and `RR_P2_L4`. The same `SCShop_Pyro_RestStop_Rund_Admin` template occurs alongside these locations. A shared template cannot resolve a terminal by itself, and temporal proximity alone has not validated each code-to-terminal pair.

Six `SendCommodityBuyRequest` records were also found. Transaction fields include `price`, `shopPricePerCentiSCU`, `quantity`, `boxSize` and `unitAmount`. Their values were neither retained nor used. Transaction records supply no price, SCU, status or sell-assortment witness (07 §2.5 item 6).

## Mapping evidence and unresolved candidates

The installed loose English `global.ini` supplies the labels below. Its provenance against the current packaged game data was not established: the packaged English entry was found, but the available ZIP reader rejected its compression method. Consequently these are corroborating names and candidates, not a validated current-build mapping registry.

| Internal name or code | Local label | Public UEX cross-check on 2026-11-01 | Status |
|---|---|---|---|
| `ResourceType.PlasmaFuel` | `items_commodities_type_plasmaFuel=Hydrogen Fuel` | Commodity 104, Hydrogen Fuel | Paired capture corroboration below; current packaged localisation not verified |
| `ResourceType.RMC` | `items_commodities_type_RMC=Recycled Material Composite` | Commodity 63, Recycled Material Composite | Paired capture corroboration below; current packaged localisation not verified |
| `ResourceType.QuantumFuel` | `items_commodities_type_quantumFuel=Quantum Fuel` | Commodity 87, Quantum Fuel | Paired capture corroboration below; packaged provenance unverified |
| `ResourceType.Waste` | `items_commodities_type_waste=Waste` | Commodity 79, Waste | Name agreement only; no independently inspected card in this review |
| `RR_P2_L4` | Checkmate | Commodity terminal 436, Admin - Checkmate, at Checkmate Station | Location candidate only; UEX also lists other terminal types there |
| `RR_P3_L3` | Patch City | Commodity terminal 453, Admin - Patch City, at Patch City | Location candidate only; the admin template does not prove identity |
| `RR_P6_LEO` | Ruin Station, visible in paired capture | Commodity terminal 466, Admin - Ruin Station | Corrected 2026-11-01: paired correspondence below |
| `ResourceType.AltruciaToxin_unprocessed` | Revenant Tree Pollen (follow-up below) | Commodity 66, Revenant Tree Pollen; distinct from commodity 4, Altruciatoxin | Corrected 2026-11-01: paired buy capture and explicit localisation key corroborate pollen; never discard the suffix |
| `ResourceType.ShipAmmoSize1` through `ResourceType.ShipAmmoSize7`, `ResourceType.ShipCountermeasuresDecoy`, `ResourceType.ShipCountermeasuresNoise` | Loose keys name Ship Ammunition - Size 1 through 7, Ship Decoy Countermeasures and Ship Noise Countermeasures | Selected UEX names include Ship Decoy Countermeasures 208 and Ship Noise Countermeasures 209 | Candidate labels; no complete independently inspected mapping validation |

Other observed internal commodity names are `AgriculturalSupplies`, `Carbon`, `DistilledSpirits`, `Dynaflex`, `Helium`, `Mercury`, `Potassium`, `Pressurized_Ice`, `ProcessedFood` and `Silicon` (all under `ResourceType`). Selected public UEX names have plausible counterparts, but removing punctuation or comparing similar strings does not validate their identities. No automatic mapping is enabled by this review.

Public credential-free sources: [UEX commodities](https://api.uexcorp.uk/2.0/commodities), [UEX terminals](https://api.uexcorp.uk/2.0/terminals) and [UEX game versions](https://api.uexcorp.uk/2.0/game_versions). These were GET-only spot checks; no credentials or API writes were used and no price table was retained.

## Version and capture matching

The public `game_versions` response on 2026-11-01 names LIVE `4.10.2` and PTU `4.10.3`. The local manifest branch agrees with LIVE on this occasion. That coincidence does not establish a stable conversion from the four-part build version or branch to the UEX patch string. The R-CAP-3b witness remains major/minor only; a differing major or minor version makes the capture uncertain and cannot make it certain.

Corrected 2026-11-01: the initial survey overlooked the existing public 4.10.2 captures dated 2026-10-14. Their `capturedAt` values agree with all 150 recorded original screenshot filenames; the metadata pins the original SHA-256 and describes lossless redaction. No capture time was inferred from a later file modification time. All four entries remain `verified: false`: these timing measurements and selected pixel inspections do not make their transcriptions golden data.

Offsets below use UTC capture time minus the latest preceding valid shop-inventory row in the reviewed logs. The row must contain all three markers: `LoadShopInventoryData`, `AddingCommodityBox` and `Available Box Sizes`. Taking the latest row of the preceding load avoids a future load and excludes player inventory. The latest location-inventory code before that row agrees with the latest code before the capture in every case. The measured row times retain milliseconds; capture metadata has only whole seconds, so the displayed decimal offsets do not imply subsecond capture accuracy.

| Corpus entry | Captures | Offset range (seconds) | Median (seconds) | Captures within 300 seconds |
|---|---:|---:|---:|---:|
| `ruin-station-01` | 27 | 7.958–88.958 | 52.958 | 27 |
| `patch-city-01` | 11 | 4.812–35.812 | 16.812 | 11 |
| `seraphim-station-01` | 26 | 9.723–69.226 | 39.475 | 26 |
| `levski-01` | 86 | 5.665–104.013 | 51.876 | 86 |
| Combined follow-up dataset | 150 | 4.812–104.013 | 45.120 | 150 |

These include buy and sell screenshots. For sell captures the match can support location only, never sell assortment or container sizes. Keep A24's separate 2026-10-09 dataset unchanged: 76 captures, 7–170 seconds, median 55 seconds. The new observations support retaining the existing five-minute starting window; they do not establish a new maximum or safety margin across patches. A future load or `CaptureTimeUncertain` capture remains ineligible under 07 §2.5 item 6.

## Paired location and commodity follow-up

The selected screenshots were inspected directly, independently of `expectedTerminalId` and card transcriptions. The table records observed correspondences for the reviewed 4.10.2 build, not an exhaustive mapping registry. Public UEX GETs on 2026-11-01 identify one commodity terminal at each of these named locations; other terminal types are not interchangeable with it.

| Log location and shop template | Independently visible evidence | Exact UEX commodity terminal | Limit |
|---|---|---|---|
| `RR_P6_LEO` + `SCShop_Pyro_RestStop_Rund_Admin` | `ruin-station-01/buy-1.jpg`: location field `RUIN STATION`; debug overlay also gives `RR_P6_LEO` | 466, Admin - Ruin Station, at Ruin Station | Supports this paired correspondence; the shared template alone does not identify it |
| `RR_CRU_LEO` + `SCShop_Admin_lt_base_g` | `seraphim-station-01/buy-1.jpg`: `SERAPHIM STATION` and buy panel | 259, Admin - Seraphim, at Seraphim Station | Supports this paired correspondence, not other users of the template |
| `Nyx_Levski` + `SCShop_Levski_CargoOffice_Commodities` | `levski-01/v1-buy-5.jpg`: `Levski` and buy panel | 778, Levski, at Levski | Supports this paired correspondence |
| `RR_P3_L3` + `SCShop_Pyro_RestStop_Rund_Admin` | `patch-city-01/buy-1.jpg`: overlay gives `RR_P3_L3`, but the location field says `SELECT LOCATION` | Candidate 453, Admin - Patch City | Exact identity remains unvalidated: the unverified expected terminal and loose localisation label do not replace an independently established station name |

The installed loose localisation file is not the file pinned by these captures: current SHA-256 `10f4636f5408fee3aabd780857907e4c3555aefeff5a14648b4e3e785b68625c`, 11,267,066 bytes, versus corpus SHA-256 `a9e6e4b126c61a97fd6ccbe7ad846f2d6522534eb347a90cab276853c36023ec`, 10,715,737 bytes. The original pinned localisation file was not available in this review. No claim of packaged or capture-time localisation provenance follows from the current loose file.

Selected aliases have stronger evidence than similar names: explicit loose localisation keys agree with independently visible buy cards, their paired internal names and size sets, and exact public UEX names. This corroborates the following correspondences for the reviewed captures; it does not validate unrelated aliases or establish packaged localisation provenance.

| Internal commodity | Independently inspected capture | Visible name / exact UEX commodity | Size set visible on screen and present in paired load |
|---|---|---|---|
| `ResourceType.PlasmaFuel` | `levski-01/v1-buy-5.jpg`, `seraphim-station-01/buy-7.jpg` | Hydrogen Fuel / 104 | `{1,2,4,8,16,24,32}` |
| `ResourceType.QuantumFuel` | Same captures | Quantum Fuel / 87 | `{1,2,4,8,16,24,32}` |
| `ResourceType.RMC` | `seraphim-station-01/buy-7.jpg` | Recycled Material Composite / 63 | `{1,2,4,8,16,24,32}` |
| `ResourceType.AltruciaToxin_unprocessed` | `ruin-station-01/buy-1.jpg` | `[!] Revenant Tree Pollen` / 66 | `{24,32}` |

The last correspondence is also named explicitly by `items_commodities_altruciatoxin_unprocessed` in the loose file. It is a concrete counterexample to removing `_unprocessed`: UEX's Altruciatoxin is commodity 4, a different name and ID. `ResourceType.Hydrogen` is also distinct from `PlasmaFuel`: the Seraphim buy screen shows Hydrogen with `{8,16,24,32}`, and UEX names Hydrogen as 41 and Hydrogen Fuel as 104. No price, quantity or status from the log was used to establish these correspondences.

## Synthetic format specimens

These are documentation fixtures for the observed payload shapes, not raw excerpts, executable parser tests or accepted real mappings. The omitted prefix and suffix are deliberate: they do not specify a complete grammar. Names, IDs, locations, timestamp and size combinations are synthetic. Production storage would contain only allowlisted witness facts, never the specimens' private-field sentinels.

Buy-side load candidate payload:

```text
<2000-01-01T12:00:00.000Z> [Notice] ... CEntityComponentCommodityUIProvider::LoadShopInventoryData ... AddingCommodityBox - playerId[SYNTHETIC_PLAYER_ID] shopId[SYNTHETIC_SHOP_ID] shopName[SCShop_SYNTHETIC_Admin] commodityName[ResourceType.SyntheticCommodity] Available Box Sizes:  boxSize[1] boxSize[2] ...
```

Expected interpretation: candidate buy-side internal commodity and sizes at the log time; no resolved terminal or UEX commodity until separately validated. Drop both identifier fields immediately. Nothing supplies a reported value.

Player-inventory exclusion payload:

```text
<2000-01-01T12:00:01.000Z> [Notice] ... CEntityComponentCommodityUIProvider::AddPlayerCommodityItem ... AddingCommodityBox ... commodityName[ResourceType.SyntheticCommodity] ...
```

Expected interpretation: reject as kiosk assortment despite the shared keyword. Do not infer sell assortment.

Location payload:

```text
<2000-01-01T11:59:59.000Z> [Notice] ... <RequestLocationInventory> Player[SYNTHETIC_HANDLE] requested inventory for Location[Synthetic_Location] ...
```

Expected interpretation: candidate location code at the log time; drop the handle immediately. The invented code has no mapping.

Synthetic matching examples under the existing rule: with a certain capture at `12:04:00Z` and the latest validated kiosk load at `12:00:00Z`, the four-minute offset is inside the default window. A capture at `12:06:00Z` is outside it; a load after the capture and a `CaptureTimeUncertain` capture are ineligible. These examples illustrate the requirement, not a newly measured offset.

## Remaining acceptance evidence

Issue #107 remains partial. The current format and dangerous keyword collision are directly observed; buy-side interpretation, unavailable sell-side facts and the limited version witness remain consistent with A24. Complete current-version validation still needs:

- Game-owned localisation or independently checked captures linking the remaining internal commodity names to exact UEX IDs, with unknown and unprocessed forms kept separate.
- Remaining location-code/template identities, especially an independent station-name link for the Patch City candidate; the three paired correspondences above do not validate every shared-template location.
- Broader per-version mapping coverage beyond the selected paired captures. Timing is now measured for 150 captures; the existing window is retained.
- The #108 parser's independent privacy checks before any persistent witness facts exist.

The M2 spike checkbox remains open. This document introduces no parser, persistent game-log data, normative change, verified corpus entry or API write.
