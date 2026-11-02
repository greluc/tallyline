// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dialog state changed only by direct user actions.

/// The single owner of user-requested dialogs.
#[derive(Default)]
pub struct DialogPresenter {
    about: bool,
    build: crate::api::model::BuildPresentation,
    gpl: bool,
    licences: bool,
    catalogue: Option<crate::api::model::LicenceCatalogue>,
    pub(crate) manual_confirmation: Option<crate::api::model::ManualReviewCommand>,
}

impl DialogPresenter {
    pub(crate) fn set_build_presentation(&mut self, build: crate::api::model::BuildPresentation) {
        self.build = build;
    }

    /// Returns the validated offline component catalogue.
    #[must_use]
    pub const fn licences(&self) -> Option<&crate::api::model::LicenceCatalogue> {
        self.catalogue.as_ref()
    }

    pub(crate) fn set_licences(&mut self, catalogue: Option<crate::api::model::LicenceCatalogue>) {
        self.catalogue = catalogue;
    }
    /// Opens About after a direct user action.
    pub fn open_about(&mut self) {
        self.about = true;
    }

    /// Draws user-requested legal dialogs without background-triggered window commands.
    pub fn show(
        &mut self,
        context: &egui::Context,
        catalogue: &crate::api::Catalogue,
        unit: &crate::api::FanKitUnit,
    ) {
        egui::Window::new(catalogue.text("about.title"))
            .open(&mut self.about)
            .resizable(true)
            .vscroll(true)
            .default_width(760.0)
            .show(context, |ui| {
                for action in
                    crate::legal_views::show_about_view_with_build(ui, catalogue, unit, &self.build)
                {
                    match action {
                        crate::api::model::AboutAction::ShowGpl => self.gpl = true,
                        crate::api::model::AboutAction::ShowLicences => self.licences = true,
                    }
                }
            });
        egui::Window::new(catalogue.text("about.gpl"))
            .open(&mut self.gpl)
            .vscroll(true)
            .show(context, |ui| {
                ui.label(include_str!("../../../LICENSE"));
            });
        egui::Window::new(catalogue.text("about.licences"))
            .open(&mut self.licences)
            .vscroll(true)
            .show(context, |ui| {
                if let Some(licences) = &self.catalogue {
                    for component in licences.components() {
                        ui.heading(format!("{} {}", component.name, component.version));
                        ui.label(&component.licence);
                        match &component.copyright {
                            crate::api::model::CopyrightInformation::Provided { notices } => {
                                for line in notices {
                                    ui.label(line);
                                }
                            }
                            crate::api::model::CopyrightInformation::NotStatedByUpstream => {
                                ui.label(catalogue.text("about.no_copyright_stated"));
                            }
                        }
                        ui.hyperlink_to(catalogue.text("about.source"), &component.source);
                        for notice in &component.notices {
                            ui.collapsing(&notice.path, |ui| {
                                ui.label(&notice.text);
                            });
                        }
                        ui.separator();
                    }
                } else {
                    ui.label(catalogue.text("about.missing_licences"));
                }
            });
    }
    /// Returns whether the user opened About.
    #[must_use]
    pub const fn about_open(&self) -> bool {
        self.about
    }
}
