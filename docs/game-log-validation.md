# Current Game.log witness validation

> **Doc type:** Living reference — partial evidence; mappings and paired captures remain open. Last reviewed: 2026-11-01.

This is the dated result of the local current-LIVE review for issue #107, R-OCR-20, A24 and O-101. It records observations, not a production format or mapping registry. The witness remains opt-in and default off; unknown formats and mappings supply no automatic resolving evidence. The implementation and its privacy tests belong to issue #108.

## Evidence boundary

Reviewed 2026-11-01: the user-authorised LIVE installation manifest and a current-build backup containing kiosk records. The manifest reports build `4.10.197.2878`, branch `sc-alpha-4.10.2`. Its `Data.Version` and `Data.Branch` are the relevant allowlisted fields. The current log and recent backups were surveyed locally; the useful kiosk observations below come from one backup, not from a representative set of terminals or patches.

Raw logs remain local. No raw line, player handle, player or session identifier, endpoint, balance, personal path or complete reference-data dump is included here. Field names below describe the format; all example values in the synthetic specimens are invented. Reading and summarising selected fields is not proof that an eventual parser drops private fields: that requires the #108 privacy test through `GameLogSource`.

## Current format observations

Of 385 records containing `AddingCommodityBox`, 370 also contain `CEntityComponentCommodityUIProvider::LoadShopInventoryData` and `Available Box Sizes`. All 370 have a UTC timestamp shaped `<YYYY-MM-DDTHH:MM:SS.mmmZ>` and `[Notice]` severity. Their fields include `playerId[]`, `shopId[]`, `shopName[]`, `commodityName[ResourceType.<internal name>]` and repeated `boxSize[]` values. The source names the load of shop inventory; it is not a scroll event or proof of sell-side assortment.

The other 15 records come from `CEntityComponentCommodityUIProvider::AddPlayerCommodityItem` and lack `Available Box Sizes`. They describe player inventory, so matching `AddingCommodityBox` alone would admit the wrong source. They must never establish kiosk assortment or sell-side facts.

The valid records contain these size sets: `{8,16,24,32}`, `{16,24,32}`, `{1,2,4,8,16,24,32}`, `{24,32}` and `{4,8,16,24,32}`. No stock, quantity, `scu_buy`, `scu_sell` or price field was found on these 370 records. This confirms the current shape of the buy-side witness; the historical screen/UEX comparison in A24 remains the evidence for side interpretation and size agreement. A new comparison against current paired captures or current UEX per-row size sets was not performed.

Location records retain the historical `<RequestLocationInventory>` shape: `Player[] requested inventory for Location[]`. Current observations include `RR_P6_LEO`, `RR_P3_L3` and `RR_P2_L4`. The same `SCShop_Pyro_RestStop_Rund_Admin` template occurs alongside these locations. A shared template cannot resolve a terminal by itself, and temporal proximity alone has not validated each code-to-terminal pair.

Six `SendCommodityBuyRequest` records were also found. Transaction fields include `price`, `shopPricePerCentiSCU`, `quantity`, `boxSize` and `unitAmount`. Their values were neither retained nor used. Transaction records supply no price, SCU, status or sell-assortment witness (07 §2.5 item 6).

## Mapping evidence and unresolved candidates

The installed loose English `global.ini` supplies the labels below. Its provenance against the current packaged game data was not established: the packaged English entry was found, but the available ZIP reader rejected its compression method. Consequently these are corroborating names and candidates, not a validated current-build mapping registry.

| Internal name or code | Local label | Public UEX cross-check on 2026-11-01 | Status |
|---|---|---|---|
| `ResourceType.PlasmaFuel` | `items_commodities_type_plasmaFuel=Hydrogen Fuel` | Commodity 104, Hydrogen Fuel | Corroborates the historical A24 alias; current packaged localisation not verified |
| `ResourceType.RMC` | `items_commodities_type_RMC=Recycled Material Composite` | Commodity 63, Recycled Material Composite | Corroborates the historical A24 alias; current packaged localisation not verified |
| `ResourceType.QuantumFuel` | `items_commodities_type_quantumFuel=Quantum Fuel` | Commodity 87, Quantum Fuel | Name agreement only; no current capture comparison |
| `ResourceType.Waste` | `items_commodities_type_waste=Waste` | Commodity 79, Waste | Name agreement only; no current capture comparison |
| `RR_P2_L4` | Checkmate | Commodity terminal 436, Admin - Checkmate, at Checkmate Station | Location candidate only; UEX also lists other terminal types there |
| `RR_P3_L3` | Patch City | Commodity terminal 453, Admin - Patch City, at Patch City | Location candidate only; the admin template does not prove identity |
| `RR_P6_LEO` | Not established by this review | No validated terminal pair | Unmapped |
| `ResourceType.AltruciaToxin_unprocessed` | Not established by this review | Commodity 4 is named Altruciatoxin | Unmapped; the unprocessed suffix must not be discarded to force a match |
| `ResourceType.ShipAmmoSize1` through `ResourceType.ShipAmmoSize7`, `ResourceType.ShipCountermeasuresDecoy`, `ResourceType.ShipCountermeasuresNoise` | Not established by this review | No validated commodity pair | Unmapped |

Other observed internal commodity names are `AgriculturalSupplies`, `Carbon`, `DistilledSpirits`, `Dynaflex`, `Helium`, `Mercury`, `Potassium`, `Pressurized_Ice`, `ProcessedFood` and `Silicon` (all under `ResourceType`). Selected public UEX names have plausible counterparts, but removing punctuation or comparing similar strings does not validate their identities. No automatic mapping is enabled by this review.

Public credential-free sources: [UEX commodities](https://api.uexcorp.uk/2.0/commodities), [UEX terminals](https://api.uexcorp.uk/2.0/terminals) and [UEX game versions](https://api.uexcorp.uk/2.0/game_versions). These were GET-only spot checks; no credentials or API writes were used and no price table was retained.

## Version and capture matching

The public `game_versions` response on 2026-11-01 names LIVE `4.10.2` and PTU `4.10.3`. The local manifest branch agrees with LIVE on this occasion. That coincidence does not establish a stable conversion from the four-part build version or branch to the UEX patch string. The R-CAP-3b witness remains major/minor only; a differing major or minor version makes the capture uncertain and cannot make it certain.

No current-build paired captures were inspected in this review. There is no new measured capture offset. Keep A24's 2026-10-09 result separate: 76 captures, 7–170 seconds after the latest kiosk load, median 55 seconds. The existing five-minute starting window remains unchanged. Under 07 §2.5 item 6 the applicable load must be the latest one before a certain capture time and inside that window; future loads and uncertain capture times cannot witness a capture.

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
- Current paired captures establishing each location-code/template-to-terminal identity, including locations sharing the same template.
- Current paired capture times measuring offsets against the latest preceding kiosk load; no new window can be claimed from this review.
- The #108 parser's independent privacy checks before any persistent witness facts exist.

The M2 spike checkbox remains open. This document introduces no parser, persistent game-log data, normative change, verified corpus entry or API write.
