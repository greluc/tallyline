// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configurable channel-to-UEX environment mapping contracts.

use tallyline_game::api::model::{EnvironmentMapping, GameEnvironment};
use tallyline_shared_kernel::api::model::UexEnvironment;

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_cap_3a__channel_mapping_is_configurable_without_changing_other_channels() {
    let mapping = EnvironmentMapping::default();
    let expected = [
        (GameEnvironment::Live, UexEnvironment::Live),
        (GameEnvironment::Hotfix, UexEnvironment::Live),
        (GameEnvironment::Ptu, UexEnvironment::Ptu),
        (GameEnvironment::Eptu, UexEnvironment::Ptu),
        (GameEnvironment::TechPreview, UexEnvironment::Ptu),
    ];
    for (channel, environment) in expected {
        assert_eq!(mapping.environment(channel), environment);
        let replacement = match environment {
            UexEnvironment::Live => UexEnvironment::Ptu,
            UexEnvironment::Ptu => UexEnvironment::Live,
        };
        let changed = mapping.with_environment(channel, replacement);
        for (other, original) in expected {
            assert_eq!(
                changed.environment(other),
                if other == channel {
                    replacement
                } else {
                    original
                }
            );
            assert_eq!(mapping.environment(other), original);
        }
    }
    let changed = mapping.with_environment(GameEnvironment::Hotfix, UexEnvironment::Ptu);
    assert_eq!(
        changed.environment(GameEnvironment::Hotfix),
        UexEnvironment::Ptu
    );
    assert_eq!(
        changed.environment(GameEnvironment::Live),
        UexEnvironment::Live
    );
    assert_eq!(
        changed.environment(GameEnvironment::Ptu),
        UexEnvironment::Ptu
    );
    assert_eq!(
        mapping.environment(GameEnvironment::Hotfix),
        UexEnvironment::Live
    );
}
