# Supplied design references and component adoption

> **Doc type:** Living reference — partial egui adoption. Last reviewed: 2026-11-03.

The owner-supplied archives guide the Tallyline rendition. Accepted requirements, ADRs and owner decisions remain normative. This checkpoint adopts typography, existing widget geometry and theme states, and exact manual-review number presentation. The shell, review table and M1–M5 remain incomplete.

## Source identities and precedence

| Archive | SHA-256 | Reference paths |
|---|---|---|
| `Tallyline-Design-System.zip` | `ac6f69b6ecaf22535978eec678de598f82c1db46ac1729fba82e4de3baf50207` | `tallyline-design-system/{README.md,platform-rules.md,state-language.md,tokens.json,components/<name>/README.md}` |
| `tallyline-egui.zip` | `787c87a0ac63b0a7a393a8fa8aadcc499026c778de3a2e0d1cef99ca6b66b90f` | `tallyline-egui/{README.md,src/fonts.rs,src/theme.rs,src/widgets/}` |

Corrected 2026-11-03: both archives are available as references; earlier absence statements describe the 2026-11-01 recreation. Archive scripts/code are not executed or imported wholesale, and these are not build/runtime inputs.

- Preserve Tallyline, GPL-3.0-or-later, separate-file font delivery, egui/eframe 0.37.0 and current application/command boundaries. Archive titles, placeholder licences and 0.36.2 examples do not supersede those choices.
- `ui/design-tokens/` remains authoritative; the different archive schema does not replace the reconciled DTCG documents.
- O-83/O-112 retain D1–D11, including concept (c), Line. The archive's unselected caption is stale. AppShell specifies no logo inside the application shell; native icon and external brand exports are delivered separately in [branding](../../branding/README.md).
- Preserve the Fan Kit unit, exact notice bytes, placements, original files and pinning tests.
- The five archive TTFs match existing approved hashes in NOTICE §2.12. Reuse the repository's unchanged font files and source-specific OFL texts; the differing archive JetBrains licence file is not adopted.
- O-112 already approves the modified Material Symbols approach. The supplied binary still needs immutable upstream provenance, exact mapping checks, notices and packaging/rendering acceptance. No icon-font bytes are adopted here.

## Delivered and remaining components

| Reference area | This checkpoint | Remaining work |
|---|---|---|
| Typography and tokens | Five exact font families and all 20 registered styles; explicit semantic face/size/line-height roles in reference-browser headings, manual review/confirmation, action labels and existing widgets; source scrim alpha in the actual confirmation modal | Specialised screen composition, native scale/text-size acceptance |
| Chip, DeltaBadge, Field, Panel, Toast, IconButton | Small radii, strong field outline, raised panel with 16×12 padding, minor deviation outline, closed semantic message tones using valid foreground/background/border tokens | All icon-plus-text, environment/source/gate variants; Toast's unused string tone becomes `SemanticTone`, while existing application struct callers remain compatible |
| Button, Dialog, Menu, TextField | Token button states, 2 px keyboard button focus, dialog/menu surfaces, muted text, separate text-selection background, no high-contrast shadows | Pinned egui's non-experimental TextEdit shares its focus-border stroke with selected-text foreground; independent text-edit focus treatment remains open |
| NumericField, ConfirmationControl | Exact current price/SCU values use strong Mono, confirmation values use large Mono, and references/deltas use regular Mono, English grouping, explicit positive delta signs, U+2212 and adjacent percent sign; precision and zero-reference handling retained | Remaining numerical displays and complete editable field |
| AppShell, TopBar, StatusBar, Tabs, SettingsSection | Existing actions receive weighted labels | Persistent topbar/sidebar-or-rail/content/status shell and responsive navigation |
| ReviewTable, TableRow, TableCell, SummaryHeader, DetailStrip, ScreenshotPane | Current manual review commands, focus and explicit confirmations retained | Fixed-height table, sticky summary/header, separate selection/gate slots, cell navigation/editing, readable crops and overlays |
| Icon, AppIconConcepts, FanKitUnit, AboutDialog, OpenSourceLicencesView | Existing primitive state icons and legal placements retained | Material Symbols evidence and rendering, and full reference-screen composition; Line branding/native icons are delivered separately in [branding](../../branding/README.md) |
| QueueRow, HistoryRow, StateChip, Banner and workflow components | Existing behaviour retained | Component-specific layouts/state matrices; no unfinished OCR/VLM capability is implied |

Every component name below resolves to `tallyline-design-system/components/<name>/README.md` in the hash-bound archive. This inventory is a reference set, not evidence that these components exist in the app.
`AboutDialog`, `AccountStateLine`, `AffinityField`, `AiJobChip`, `AiRunningIndicator`, `AppIconConcepts`, `AppShell`, `AppTokenField`, `Banner`, `BlockReasonList`, `Button`, `CaptureCheckResult`, `Checkbox`, `Chip`, `CombinationMatrix`, `ConfirmationControl`, `ConnectionTestResult`, `ContainerSizeChips`, `ContextHeader`, `ContrastMatrix`, `CountBadge`, `Cover`, `CropThumbnail`, `DeltaBadge`, `DetailStrip`, `Dialog`, `DropTarget`, `EmptyState`, `EnvironmentChip`, `EnvironmentSelector`, `FanKitUnit`, `Field`, `FindingsList`, `FolderPathPicker`, `FolderStatusRow`, `GameLogWitnessSetting`, `GameVersionCertainty`, `GameVersionChoice`, `HistoryRow`, `Icon`, `IconButton`, `ImportResultLine`, `IntakeSetAsideLine`, `LicenceTextViewer`, `LogViewer`, `Menu`, `MissingRow`, `NotificationPanel`, `NotObservedRow`, `NumericField`, `ObservationTimeField`, `OnboardingStepper`, `OpenSourceLicencesView`, `Panel`, `ProgressBar`, `QueueHoldNotice`, `QueueRow`, `RadioButton`, `ReleaseSplitNotice`, `ReviewTable`, `RowWithdrawControl`, `ScreenshotPane`, `Scrollbar`, `SecretKeyField`, `SessionRow`, `SettingsErrorNotice`, `SettingsSection`, `ShortcutHint`, `SideSelector`, `SourceChip`, `SplitPane`, `StaffNoticeBanner`, `StateChip`, `StatusBar`, `StatusDot`, `StatusSelector`, `SummaryHeader`, `SupersededChoice`, `Switch`, `TableCell`, `TableRow`, `Tabs`, `TerminalCombobox`, `TextField`, `ThresholdField`, `Toast`, `ToggleButton`, `Tooltip`, `TopBar`, `UexRowLifecycle`, `UnexpectedCommodityRow`, `UploadPreviewConfirmation`.

## Verification boundary

Drawn tests inspect actual reference-browser, manual-review and confirmation role faces, sizes and explicit line heights, including current price/SCU values, alongside installed font bytes, widget rectangles, semantic colours, keyboard button focus and the requested modal scrim. Public manual-review tests include exact values beyond binary floating-point precision and explicit confirmation commands. They do not establish native Windows rendering, 100/125/150% display scaling, 200% text, rendered-pixel contrast or Linux acceptance. Fan Kit pinning tests remain unchanged.