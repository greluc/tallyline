// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Atomic manual edit contracts at the public application boundary.

mod support;

use support::{Fixed, Repository};
use tallyline_reference_data::api::model::UexEnvironment;
use tallyline_reporting::api::{ManualEdit, ReportingApplication};
use tallyline_shared_kernel::api::model::CommodityId;

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_man_3__ordered_edits_preserve_exact_user_values_and_explicit_takeover_together() {
    let snapshot = support::snapshot(UexEnvironment::Live).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let report = app.create(support::key().unwrap(), None).unwrap();
    let report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::AddCommodity(CommodityId(42)),
            &snapshot,
        )
        .unwrap();
    let row = report.rows()[0].id();
    let report = app
        .edit_batch(
            report.id(),
            report.version(),
            &[
                ManualEdit::TakeOver(row),
                ManualEdit::Price(row, Some("12345678901234567890.125".parse().unwrap())),
            ],
            &snapshot,
        )
        .unwrap();
    assert_eq!(
        report.rows()[0].price().unwrap().to_string(),
        "12345678901234567890.125"
    );
    assert_eq!(report.rows()[0].stock().unwrap().to_string(), "0");
    assert_eq!(report.rows()[0].status().unwrap().get(), 3);
    assert_eq!(app.drafts().unwrap(), vec![report]);
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_2_r_man_6__batch_refusals_preserve_the_complete_draft() {
    use tallyline_game::api::model::{EnvironmentMapping, GameEnvironment};
    use tallyline_reporting::api::RepositoryError;
    use tallyline_reporting::api::model::{ReportKey, RowId};
    use tallyline_shared_kernel::api::model::{TerminalId, TradeSide};
    let live = support::snapshot(UexEnvironment::Live).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let report = app.create(support::key().unwrap(), None).unwrap();
    let report = app
        .edit(
            report.id(),
            0,
            ManualEdit::AddCommodity(CommodityId(42)),
            &live,
        )
        .unwrap();
    let row = report.rows()[0].id();
    assert_eq!(
        app.edit_batch(report.id(), report.version(), &[], &live),
        Err(RepositoryError::Invalid)
    );
    assert_eq!(
        app.edit_batch(report.id(), 0, &[ManualEdit::TakeOver(row)], &live),
        Err(RepositoryError::Conflict)
    );
    for invalid in [
        ManualEdit::AddCommodity(CommodityId(999)),
        ManualEdit::Price(RowId(99), Some("100".parse().unwrap())),
        ManualEdit::Price(row, Some("0".parse().unwrap())),
        ManualEdit::Status(row, std::num::NonZeroU8::new(7)),
        ManualEdit::ChangeKey(
            ReportKey::new(TerminalId(999), TradeSide::Buy, GameEnvironment::Live).unwrap(),
        ),
    ] {
        assert_eq!(
            app.edit_batch(
                report.id(),
                report.version(),
                &[
                    ManualEdit::Stock(row, Some("12.5".parse().unwrap())),
                    invalid,
                ],
                &live
            ),
            Err(RepositoryError::Invalid)
        );
        assert_eq!(app.drafts().unwrap(), vec![report.clone()]);
    }
    let empty = support::cached_snapshot(UexEnvironment::Live, vec![]);
    assert_eq!(
        app.edit_batch(
            report.id(),
            report.version(),
            &[
                ManualEdit::Stock(row, Some("12.5".parse().unwrap())),
                ManualEdit::TakeOver(row),
            ],
            &empty
        ),
        Err(RepositoryError::Invalid)
    );
    app.set_mapping(
        EnvironmentMapping::default().with_environment(GameEnvironment::Live, UexEnvironment::Ptu),
    );
    assert_eq!(
        app.edit_batch(
            report.id(),
            report.version(),
            &[ManualEdit::TakeOver(row)],
            &live
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
fn r_man_2_r_val_7_q_manual__successful_batch_revokes_only_decisions_for_changed_values() {
    let snapshot = support::snapshot(UexEnvironment::Live).unwrap();
    let mut app = ReportingApplication::new(Repository::default(), Fixed);
    let report = app.create(support::key().unwrap(), None).unwrap();
    let report = app
        .edit(
            report.id(),
            0,
            ManualEdit::AddCommodity(CommodityId(42)),
            &snapshot,
        )
        .unwrap();
    let row = report.rows()[0].id();
    let report = app
        .edit(
            report.id(),
            report.version(),
            ManualEdit::Price(row, Some("123.45".parse().unwrap())),
            &snapshot,
        )
        .unwrap();
    let report = app
        .confirm_price(report.id(), report.version(), row, &snapshot)
        .unwrap();
    let approval = report.rows()[0].price_confirmation();
    assert!(approval.is_some());
    let report = app
        .edit_batch(
            report.id(),
            report.version(),
            &[
                ManualEdit::Price(row, Some("123.45".parse().unwrap())),
                ManualEdit::Stock(row, Some("2.5".parse().unwrap())),
            ],
            &snapshot,
        )
        .unwrap();
    assert_eq!(report.rows()[0].price_confirmation(), approval);
    let report = app
        .edit_batch(
            report.id(),
            report.version(),
            &[
                ManualEdit::Price(row, Some("900".parse().unwrap())),
                ManualEdit::Stock(row, Some("3.5".parse().unwrap())),
            ],
            &snapshot,
        )
        .unwrap();
    assert!(report.rows()[0].price_confirmation().is_none());
    assert_eq!(app.drafts().unwrap(), vec![report]);
}
