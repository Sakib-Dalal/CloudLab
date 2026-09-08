#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{webview::NewWindowResponse, Manager};
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            let web = app.path().resource_dir()?.join("web");
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let address = if let Ok(remote) = std::env::var("CLOUDLAB_DESKTOP_URL") {
                    cloudlab::agent::validate_coordinator(&remote).map(|url| url.to_string())
                } else {
                    cloudlab::server::start_desktop(data, web).await
                };
                match address {
                    Ok(address) => {
                        let windows = handle.clone();
                        let result = tauri::WebviewWindowBuilder::new(
                            &handle,
                            "main",
                            tauri::WebviewUrl::External(address.parse().unwrap()),
                        )
                        .title("CloudLab")
                        .inner_size(1440., 960.)
                        .min_inner_size(800., 600.)
                        .on_page_load(|_, payload| {
                            eprintln!("CloudLab desktop page: {:?}", payload.event());
                        })
                        .on_navigation(|url| {
                            url.scheme() == "https"
                                || (url.scheme() == "http"
                                    && url.host_str().is_some_and(|h| {
                                        h == "127.0.0.1"
                                            || h == "localhost"
                                            || h.ends_with(".localhost")
                                    }))
                        })
                        .on_new_window(move |url, features| {
                            let local = url
                                .host_str()
                                .is_some_and(|host| host.ends_with(".localhost"));
                            if url.as_str() != "about:blank"
                                && url.scheme() != "https"
                                && !(url.scheme() == "http" && local)
                            {
                                return NewWindowResponse::Deny;
                            }
                            let builder = tauri::WebviewWindowBuilder::new(
                                &windows,
                                format!("workspace-{}", cloudlab::model::id()),
                                tauri::WebviewUrl::External("about:blank".parse().unwrap()),
                            )
                            .window_features(features)
                            .title("CloudLab workspace")
                            .inner_size(1280., 850.);
                            match builder.build() {
                                Ok(window) => NewWindowResponse::Create { window },
                                Err(_) => NewWindowResponse::Deny,
                            }
                        })
                        .build();
                        if let Err(error) = result {
                            eprintln!("Could not open CloudLab: {error}");
                            handle.exit(1);
                        }
                    }
                    Err(error) => {
                        eprintln!("Could not start CloudLab: {error}");
                        handle.exit(1);
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Could not start CloudLab desktop");
}
