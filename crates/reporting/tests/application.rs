// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Manual application behaviour at its public persistence and reference boundaries.

use tallyline_game::api::model::{EnvironmentMapping, GameEnvironment};
use tallyline_reference_data::api::model::{
    CachedReference, Commodity, CommodityAverages, EndpointData, GameVersions, PricePrior,
    PriorProvenance, PriorValues, ReferenceSnapshot, StatusLevel, StatusLevels, Terminal,
    TerminalPriors, TerminalText, UexEnvironment,
};
use tallyline_reference_data::api::{ReferenceApplication, ReferenceDataCache};
use tallyline_reporting::api::model::{Report, ReportId, ReportKey, RowId};
use tallyline_reporting::api::{
    ManualEdit, ReportRepository, ReportingApplication, RepositoryError,
};
use tallyline_shared_kernel::api::Clock;
use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp, TradeSide};

struct Fixed;
impl Clock for Fixed {
    fn now(&self) -> Timestamp {
        Timestamp::from_seconds(1000)
    }
}
#[derive(Default)]
struct Repository(Vec<Report>);
impl ReportRepository for Repository {
    fn create_manual(
        &mut self,
        key: ReportKey,
        at: Timestamp,
        version: Option<&str>,
    ) -> Result<Report, RepositoryError> {
        let id = self.0.iter().map(|report| report.id().0).max().unwrap_or(0) + 1;
        let report =
            Report::manual(ReportId(id), key, at, version).map_err(|_| RepositoryError::Invalid)?;
        self.0.push(report.clone());
        Ok(report)
    }
    fn load_draft(&mut self, id: ReportId) -> Result<Option<Report>, RepositoryError> {
        Ok(self.0.iter().find(|report| report.id() == id).cloned())
    }
    fn drafts(&mut self) -> Result<Vec<Report>, RepositoryError> {
        Ok(self.0.clone())
    }
    fn save_draft(&mut self, report: &Report, expected: u64) -> Result<(), RepositoryError> {
        let stored = self
            .0
            .iter_mut()
            .find(|stored| stored.id() == report.id())
            .ok_or(RepositoryError::Invalid)?;
        if report.version() <= expected {
            return Err(RepositoryError::Invalid);
        }
        if stored.version() != expected {
            return Err(RepositoryError::Conflict);
        }
        *stored = report.clone();
        Ok(())
    }
}
struct Cache(Vec<CachedReference>);
impl ReferenceDataCache for Cache {
    type Error = std::convert::Infallible;
    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
        Ok(self.0.clone())
    }
    fn replace(&mut self, _: &CachedReference) -> Result<(), Self::Error> {
        Ok(())
    }
}
fn snapshot(environment: UexEnvironment) -> Result<ReferenceSnapshot, Box<dyn std::error::Error>> {
    let terminal = Terminal::new(
        TerminalId(11),
        TerminalText {
            name: "Depot".to_owned(),
            nickname: "DP".to_owned(),
            display_name: "Shop".to_owned(),
            location: "Pyro".to_owned(),
        },
        true,
    );
    let prior = PricePrior::new(
        CommodityId(42),
        TradeSide::Buy,
        PriorValues {
            price: "123.45".parse()?,
            stock: "0".parse()?,
            status: std::num::NonZeroU8::new(3),
            container_sizes: vec![],
        },
        PriorProvenance {
            game_version: "4.10.2".to_owned(),
            modified_at: Timestamp::from_seconds(900),
        },
    )?;
    let level = StatusLevel {
        code: std::num::NonZeroU8::new(3).ok_or("invalid fixture status")?,
        name: "Medium".to_owned(),
        short_name: "Med".to_owned(),
        abbreviation: "M".to_owned(),
    };
    Ok(ReferenceApplication::load(
        Cache(
            vec![
                EndpointData::Terminals(vec![terminal]),
                EndpointData::Commodities(vec![Commodity::new(
                    CommodityId(42),
                    "Gold",
                    "GOLD",
                    true,
                    CommodityAverages::new(None, None),
                )]),
                EndpointData::TerminalPriors(TerminalPriors::new(TerminalId(11), vec![prior])?),
                EndpointData::GameVersions(GameVersions {
                    live: "4.10.2".to_owned(),
                    ptu: None,
                }),
                EndpointData::StatusLevels(StatusLevels {
                    buy: vec![level],
                    sell: vec![],
                }),
            ]
            .into_iter()
            .map(|data| CachedReference::new(Timestamp::from_seconds(950), data))
            .collect(),
        ),
        Fixed,
    )?
    .snapshot(environment))
}
fn key() -> Result<ReportKey, tallyline_reporting::api::model::InvalidReport> {
    ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live)
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_man_6__typed_fields_and_takeover_are_durable_user_choices_without_credentials() {
    let snapshot = snapshot(UexEnvironment::Live).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let mut report = app.create(key().unwrap(), Some("4.10.2")).unwrap();
    assert_eq!(report.observed_at(), Timestamp::from_seconds(1000));
    report = app
        .edit(
            report.id(),
            0,
            ManualEdit::AddCommodity(CommodityId(42)),
            &snapshot,
        )
        .unwrap();
    let row = report.rows().first().unwrap().id();
    for edit in [
        ManualEdit::Price(row, Some("999".parse().unwrap())),
        ManualEdit::Stock(row, Some("1.25".parse().unwrap())),
        ManualEdit::Status(row, std::num::NonZeroU8::new(3)),
    ] {
        report = app
            .edit(report.id(), report.version(), edit, &snapshot)
            .unwrap();
    }
    assert_eq!(
        app.drafts()
            .unwrap()
            .first()
            .unwrap()
            .rows()
            .first()
            .unwrap()
            .stock()
            .unwrap()
            .to_string(),
        "1.25"
    );
    report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::TakeOver(row),
            &snapshot,
        )
        .unwrap();
    assert_eq!(
        report.rows().first().unwrap().price().unwrap().to_string(),
        "123.45"
    );
    assert_eq!(
        report.rows().first().unwrap().stock().unwrap().to_string(),
        "0"
    );
    assert_eq!(app.drafts().unwrap(), vec![report]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_man_6__stale_edits_invalid_vocabulary_and_wrong_environment_never_replace_a_draft() {
    let snapshot = snapshot(UexEnvironment::Live).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let report = app.create(key().unwrap(), None).unwrap();
    assert_eq!(
        app.edit(
            report.id(),
            1,
            ManualEdit::AddCommodity(CommodityId(42)),
            &snapshot
        ),
        Err(RepositoryError::Conflict)
    );
    assert_eq!(
        app.edit(
            report.id(),
            0,
            ManualEdit::AddCommodity(CommodityId(999)),
            &snapshot
        ),
        Err(RepositoryError::Invalid)
    );
    let report = app
        .edit(
            report.id(),
            0,
            ManualEdit::AddCommodity(CommodityId(42)),
            &snapshot,
        )
        .unwrap();
    let row = report.rows().first().unwrap().id();
    assert_eq!(
        app.edit(
            report.id(),
            report.version(),
            ManualEdit::Status(row, std::num::NonZeroU8::new(7)),
            &snapshot
        ),
        Err(RepositoryError::Invalid)
    );
    assert_eq!(
        app.edit(
            report.id(),
            report.version(),
            ManualEdit::Price(RowId(99), Some("10".parse().unwrap())),
            &snapshot
        ),
        Err(RepositoryError::Invalid)
    );
    assert_eq!(app.drafts().unwrap(), vec![report]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_man_6__changed_side_and_mapping_reassess_status_and_takeover_without_erasing_user_values()
 {
    let live = snapshot(UexEnvironment::Live).unwrap();
    let ptu = snapshot(UexEnvironment::Ptu).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let mut report = app.create(key().unwrap(), Some("4.10.2")).unwrap();
    report = app
        .edit(
            report.id(),
            0,
            ManualEdit::AddCommodity(CommodityId(42)),
            &live,
        )
        .unwrap();
    let row = report.rows().first().unwrap().id();
    report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::TakeOver(row),
            &live,
        )
        .unwrap();
    report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::ChangeKey(
                ReportKey::new(TerminalId(11), TradeSide::Sell, GameEnvironment::Live).unwrap(),
            ),
            &live,
        )
        .unwrap();
    assert!(
        app.edit(
            report.id(),
            report.version(),
            ManualEdit::TakeOver(row),
            &live
        )
        .is_err()
    );
    assert!(
        app.edit(
            report.id(),
            report.version(),
            ManualEdit::Status(row, std::num::NonZeroU8::new(3)),
            &live
        )
        .is_err()
    );
    app.set_mapping(
        EnvironmentMapping::default().with_environment(GameEnvironment::Live, UexEnvironment::Ptu),
    );
    assert!(
        app.edit(
            report.id(),
            report.version(),
            ManualEdit::Price(row, None),
            &live
        )
        .is_err()
    );
    report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::Price(row, None),
            &ptu,
        )
        .unwrap();
    assert_eq!(report.rows().first().unwrap().price(), None);
    assert_eq!(
        report.rows().first().unwrap().stock().unwrap().to_string(),
        "0"
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_man_6__creating_from_reference_shows_known_commodity_inputs_empty_with_the_current_version()
 {
    let snapshot = snapshot(UexEnvironment::Live).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let report = app
        .create_from_reference(key().unwrap(), &snapshot)
        .unwrap();
    assert_eq!(report.game_version(), Some("4.10.2"));
    assert_eq!(report.rows().len(), 1);
    let row = report.rows().first().unwrap();
    assert_eq!(row.commodity(), CommodityId(42));
    assert_eq!(row.price(), None);
    assert_eq!(row.stock(), None);
    assert_eq!(row.status(), None);
    assert_eq!(row.origin(), None);
    assert_eq!(app.drafts().unwrap(), vec![report]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_man_6__reselecting_the_current_report_key_keeps_the_draft_available_without_a_false_error()
 {
    let snapshot = snapshot(UexEnvironment::Live).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let report = app.create(key().unwrap(), Some("4.10.2")).unwrap();
    assert_eq!(
        app.edit(
            report.id(),
            report.version(),
            ManualEdit::ChangeKey(key().unwrap()),
            &snapshot
        )
        .unwrap(),
        report
    );
}
