// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The native Tallyline composition root.

#![forbid(unsafe_code)]

mod executor;
mod settings_wiring;

use tallyline_ui::api::{Catalogue, DesktopShell, configure_fonts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (files, light_logo, dark_logo) =
        executor::spawn("tallyline-resources", || -> std::io::Result<_> {
            Ok((
                font_files()?,
                logo_pixels(include_bytes!(
                    "../../../docs/images/fankit/made-by-the-community.png"
                ))?,
                logo_pixels(include_bytes!(
                    "../../../docs/images/fankit/MadeByTheCommunity_White.png"
                ))?,
            ))
        })?
        .join()
        .map_err(|_| std::io::Error::other("resource worker terminated unexpectedly"))??;
    let (lifecycle_sender, lifecycle_receiver) = std::sync::mpsc::sync_channel(1);
    let result = eframe::run_native(
        "Tallyline",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1000.0, 900.0])
                .with_min_inner_size([800.0, 640.0]),
            ..Default::default()
        },
        Box::new(move |creation| {
            configure_fonts(&creation.egui_ctx, files);
            let catalogue = Catalogue::english()?;

            let (commands, events, worker) = settings_wiring::start(creation.egui_ctx.clone())?;
            lifecycle_sender.send(worker)?;
            Ok(Box::new(DesktopShell::new(
                &creation.egui_ctx,
                catalogue,
                light_logo,
                dark_logo,
                commands,
                events,
            )))
        }),
    );
    for worker in lifecycle_receiver.try_iter() {
        worker
            .join()
            .map_err(|_| std::io::Error::other("settings worker terminated unexpectedly"))?;
    }
    result?;
    Ok(())
}

fn font_files() -> std::io::Result<tallyline_ui::api::model::FontFiles> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| std::io::Error::other("executable directory is missing"))?;
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let root = match arguments.as_slice() {
        [] => directory.join("assets/fonts"),
        [option, path] if option == "--assets" => std::path::PathBuf::from(path).join("fonts"),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "usage: tallyline [--assets <resource directory>]",
            ));
        }
    };
    let read = |name: &str| std::fs::read(root.join(name));
    Ok(tallyline_ui::api::model::FontFiles {
        plex_regular: read("ibm-plex/IBMPlexSans-Regular.ttf")?,
        plex_semibold: read("ibm-plex/IBMPlexSans-SemiBold.ttf")?,
        plex_bold: read("ibm-plex/IBMPlexSans-Bold.ttf")?,
        mono_regular: read("jetbrains-mono/JetBrainsMonoNL-Regular.ttf")?,
        mono_bold: read("jetbrains-mono/JetBrainsMonoNL-Bold.ttf")?,
    })
}

fn logo_pixels(bytes: &[u8]) -> std::io::Result<egui::ColorImage> {
    let raster = tallyline_adapter_files::api::decode_logo(bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [raster.width as usize, raster.height as usize],
        &raster.rgba,
    ))
}
