#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{webview::NewWindowResponse, Manager};

fn workspace_url(url: &tauri::Url) -> bool {
    let valid_host = url
        .host_str()
        .and_then(|host| host.split('.').next())
        .and_then(|label| label.strip_prefix("w-"))
        .is_some_and(cloudlab::model::valid_id);
    valid_host
        && url.username().is_empty()
        && url.password().is_none()
        && (url.scheme() == "https"
            || (url.scheme() == "http"
                && url.host_str().is_some_and(|h| h.ends_with(".localhost"))))
}
async fn open_workspace(handle: tauri::AppHandle, url: tauri::Url) -> Result<(), String> {
    if !workspace_url(&url) {
        return Err("The coordinator returned an unsupported workspace address.".into());
    }
    let origin = url.origin();
    let builder = tauri::WebviewWindowBuilder::new(
        &handle,
        format!("workspace-{}", cloudlab::model::id()),
        tauri::WebviewUrl::External(url),
    )
    .title("CloudLab workspace")
    .inner_size(1280., 850.)
    .incognito(true)
    .on_navigation(move |next| next.as_str() == "about:blank" || next.origin() == origin)
    .on_new_window(|_, _| NewWindowResponse::Deny)
    .on_document_title_changed(|window, title| {
        let _ = window.set_title(&title);
    });
    builder
        .build()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
fn main() {
    tauri::Builder::default().setup(|app| {
        let data = app.path().app_data_dir()?;
        let resource_web = app.path().resource_dir()?.join("web");
        let web = if cfg!(debug_assertions) { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dist") } else { resource_web };
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let address = if let Ok(remote) = std::env::var("CLOUDLAB_DESKTOP_URL") {
                cloudlab::agent::validate_coordinator(&remote).map(|url| url.to_string())
            } else { cloudlab::server::start_desktop(data,web).await };
            match address {
                Ok(address) => {
                    let windows=handle.clone();
                    let result=tauri::WebviewWindowBuilder::new(&handle,"main",tauri::WebviewUrl::External(address.parse().unwrap()))
                        .title("CloudLab").inner_size(1440.,960.).min_inner_size(800.,600.)
                        .initialization_script("window.__CLOUDLAB_DESKTOP__ = true;")
                        .on_navigation(move |url| {
                            if url.path() == "/_cloudlab/desktop-open" {
                                let same_origin = windows.get_webview_window("main").and_then(|w| w.url().ok()).is_some_and(|current| current.origin()==url.origin());
                                if same_origin {
                                    if let Some(target) = url.query_pairs().find(|(key,_)| key=="url").and_then(|(_,value)| value.parse::<tauri::Url>().ok()).filter(workspace_url) {
                                        let handle=windows.clone();
                                        tauri::async_runtime::spawn(async move {
                                            if let Err(error) = open_workspace(handle.clone(), target).await {
                                                if let Some(main) = handle.get_webview_window("main") {
                                                    let message = serde_json::to_string(&format!("Could not open workspace: {error}")).unwrap();
                                                    let _ = main.eval(format!("window.dispatchEvent(new CustomEvent('cloudlab-desktop-error', {{ detail: {message} }}));"));
                                                }
                                            }
                                        });
                                    }
                                }
                                return false;
                            }
                            url.scheme()=="https" || (url.scheme()=="http" && url.host_str().is_some_and(|h| h=="127.0.0.1" || h=="localhost"))
                        })
                        .on_new_window(|_,_| NewWindowResponse::Deny)
                        .build();
                    if let Err(error)=result { eprintln!("Could not open CloudLab: {error}"); handle.exit(1); }
                }
                Err(error)=>{eprintln!("Could not start CloudLab: {error}");handle.exit(1);}
            }
        });
        Ok(())
    }).run(tauri::generate_context!()).expect("Could not start CloudLab desktop");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desktop_only_opens_valid_workspace_origins() {
        let id = cloudlab::model::id();
        for url in [
            format!("http://w-{id}.localhost:8089/?ticket=test"),
            format!("https://w-{id}.apps.example.com/"),
        ] {
            assert!(workspace_url(&url.parse().unwrap()));
        }
        for url in [
            "http://127.0.0.1:8088/",
            "file:///etc/passwd",
            "https://example.com/",
            "http://w-test.localhost:8089/",
            "https://user:password@w-test.example.com/",
        ] {
            assert!(!workspace_url(&url.parse().unwrap()));
        }
    }
}
