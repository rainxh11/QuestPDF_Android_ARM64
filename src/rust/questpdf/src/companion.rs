//! QuestPDF Companion API v3: native page snapshots and localhost preview sessions.
use crate::{
    Document, LicenseType, Renderer, Settings, Size,
    native::{self, Api, Handle},
};
use anyhow::{Context, Result, bail, ensure};
use base64::Engine;
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageLocation {
    pub page_number: usize,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl PageLocation {
    pub(crate) fn from_matrix(page_number: usize, size: Size, m: native::Matrix) -> Self {
        let points = [
            (0., 0.),
            (size.width, 0.),
            (size.width, size.height),
            (0., size.height),
        ]
        .map(|(x, y)| {
            (
                x * m.scale_x + y * m.skew_x + m.translate_x,
                x * m.skew_y + y * m.scale_y + m.translate_y,
            )
        });
        Self {
            page_number,
            left: points.iter().map(|p| p.0).fold(f32::INFINITY, f32::min),
            top: points.iter().map(|p| p.1).fold(f32::INFINITY, f32::min),
            right: points.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max),
            bottom: points.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HierarchyElement {
    pub element_type: String,
    pub hint: Option<String>,
    pub searchable_content: Option<String>,
    pub is_single_child_container: bool,
    pub page_locations: Vec<PageLocation>,
    pub layout_error_measurements: Vec<Value>,
    pub properties: Vec<Value>,
    pub source_code_declaration_path: Option<Value>,
    pub children: Vec<HierarchyElement>,
}
impl HierarchyElement {
    pub(crate) fn group(kind: &str, children: Vec<Self>) -> Self {
        Self {
            element_type: kind.into(),
            hint: None,
            searchable_content: None,
            is_single_child_container: children.len() == 1,
            page_locations: vec![],
            layout_error_measurements: vec![],
            properties: vec![],
            source_code_declaration_path: None,
            children,
        }
    }
}
pub struct PageSnapshot {
    pub size: Size,
    pub(crate) template_index: usize,
    layers: Vec<Handle>,
    api: Arc<Api>,
}
impl PageSnapshot {
    pub(crate) fn new(
        size: Size,
        template_index: usize,
        layers: Vec<Handle>,
        api: &Arc<Api>,
    ) -> Result<Self> {
        Ok(Self {
            size,
            template_index,
            layers,
            api: api.clone(),
        })
    }
    /// The Companion protocol uses powers of two rather than a DPI value.
    pub fn render_image(&self, zoom_level: i32) -> Result<Vec<u8>> {
        let scale = 2f64.powi(zoom_level) as f32;
        let (width, height) = (
            (self.size.width * scale) as i32,
            (self.size.height * scale) as i32,
        );
        ensure!(
            width > 0 && height > 0 && (width as i64) * (height as i64) <= 500_000_000,
            "Companion raster dimensions out of range"
        );
        let a = &self.api;
        unsafe {
            let bitmap = Handle::new(
                (a.questpdf_skia_bitmap_create)(width, height),
                a.questpdf_skia_bitmap_delete,
                a,
            )?;
            let canvas = Handle::new(
                (a.questpdf_skia_canvas_create_from_bitmap)(bitmap.ptr),
                a.questpdf_skia_canvas_delete,
                a,
            )?;
            a.questpdf_skia_canvas_scale(canvas.ptr, scale, scale);
            let paint = Handle::new(
                (a.questpdf_skia_paint_create)(),
                a.questpdf_skia_paint_delete,
                a,
            )?;
            (a.questpdf_skia_paint_set_solid_color)(paint.ptr, 0xffffffff);
            a.questpdf_skia_canvas_draw_rectangle(
                canvas.ptr,
                &native::Rect {
                    left: 0.,
                    top: 0.,
                    right: self.size.width,
                    bottom: self.size.height,
                },
                paint.ptr,
            );
            for layer in &self.layers {
                a.questpdf_skia_canvas_draw_picture(canvas.ptr, layer.ptr);
            }
            drop(canvas);
            let encoded = Handle::new(
                (a.questpdf_skia_bitmap_encode_as_jpg)(bitmap.ptr, 90),
                a.questpdf_skia_data_unref,
                a,
            )?;
            let mut data = native::DataBytes::default();
            (a.questpdf_skia_data_get_bytes)(encoded.ptr, &mut data);
            ensure!(
                data.length >= 0 && !data.bytes.is_null(),
                "Invalid Companion JPEG data"
            );
            Ok(std::slice::from_raw_parts(data.bytes, data.length as usize).to_vec())
        }
    }
}
pub struct DocumentSnapshot {
    pub pages: Vec<PageSnapshot>,
    pub hierarchy: HierarchyElement,
}
impl DocumentSnapshot {
    pub fn structure(&self, hot_reloaded: bool) -> Value {
        json!({"isDocumentHotReloaded":hot_reloaded,"pages":self.pages.iter().map(|p|json!({"width":p.size.width,"height":p.size.height})).collect::<Vec<_>>(),"hierarchy":self.hierarchy})
    }
}
#[derive(Clone, Debug)]
pub struct CompanionOptions {
    pub port: u16,
    /// Optional endpoint prefix, useful for a local contract simulation.
    pub base_url: Option<String>,
    pub launch_application: bool,
}
impl Default for CompanionOptions {
    fn default() -> Self {
        Self {
            port: 12500,
            base_url: None,
            launch_application: true,
        }
    }
}
pub struct CompanionSession {
    stopped: Arc<AtomicBool>,
    client: Client,
    base_url: String,
    client_id: String,
    snapshot: Option<DocumentSnapshot>,
    notified: Instant,
}
impl CompanionSession {
    pub fn connect(options: &CompanionOptions) -> Result<Self> {
        crate::settings::validate_license()?;
        Settings::configure(|s| {
            s.enable_caching = false;
            s.enable_detailed_layout_errors = true;
        });
        static CLIENT_ID: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        static ACTIVE: std::sync::Mutex<Option<std::sync::Weak<AtomicBool>>> =
            std::sync::Mutex::new(None);
        let stopped = Arc::new(AtomicBool::new(false));
        if let Some(previous) = ACTIVE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace(Arc::downgrade(&stopped))
            .and_then(|weak| weak.upgrade())
        {
            previous.store(true, Ordering::Release);
        }
        let session = Self {
            stopped,
            client: Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(10))
                .build()?,
            base_url: options
                .base_url
                .clone()
                .unwrap_or_else(|| format!("http://127.0.0.1:{}", options.port))
                .trim_end_matches('/')
                .to_owned(),
            client_id: CLIENT_ID
                .get_or_init(|| uuid::Uuid::new_v4().to_string())
                .clone(),
            snapshot: None,
            notified: Instant::now(),
        };
        if !session.responding() {
            ensure!(
                options.launch_application,
                "The QuestPDF Companion is not responding on {}",
                session.base_url
            );
            let mut command = std::process::Command::new("questpdf-companion");
            command.arg(options.port.to_string());
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000);
            }
            command
                .spawn()
                .context("Start the QuestPDF Companion application, or launch it manually")?;
            let deadline = Instant::now() + Duration::from_secs(10);
            while !session.responding() {
                ensure!(
                    Instant::now() < deadline,
                    "The QuestPDF Companion did not accept connections on port {}",
                    options.port
                );
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        let version: Value = session
            .client
            .get(session.url("/version"))
            .send()?
            .error_for_status()?
            .json()?;
        ensure!(
            version["supportedVersions"]
                .as_array()
                .is_some_and(|v| v.iter().any(|x| x.as_i64() == Some(3))),
            "The QuestPDF Companion application does not support API version 3"
        );
        Ok(session)
    }
    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
    fn responding(&self) -> bool {
        self.client
            .get(self.url("/ping"))
            .send()
            .is_ok_and(|r| r.status().is_success())
    }
    pub fn update_document(
        &mut self,
        renderer: &mut Renderer,
        document: &Document,
        hot_reloaded: bool,
    ) -> Result<()> {
        ensure!(
            document.page_number_groups.is_none(),
            "The QuestPDF Companion does not support merged documents"
        );
        let snapshot = match renderer.generate_companion_snapshot(document) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                let _=self.client.post(self.url("/v3/genericException/show")).json(&json!({"exception":{"type":"questpdf::Error","message":format!("{error:#}"),"stackTrace":[],"innerException":null}})).send();
                return Err(error);
            }
        };
        self.client
            .post(self.url("/v3/documentPreview/update"))
            .json(&snapshot.structure(hot_reloaded))
            .send()?
            .error_for_status()?;
        self.snapshot = Some(snapshot);
        Ok(())
    }
    /// Serve one long-poll round. Call repeatedly from an application's own loop.
    pub fn poll(&mut self) -> Result<bool> {
        self.poll_inner(true)
    }
    fn poll_inner(&mut self, notify: bool) -> Result<bool> {
        if self.stopped.load(Ordering::Acquire) {
            return Ok(false);
        }
        if notify && self.notified.elapsed() >= Duration::from_millis(250) {
            if !self.notify()? {
                return Ok(false);
            }
            self.notified = Instant::now();
        }
        let request = self
            .client
            .get(self.url("/v3/documentPreview/getRenderingRequests"))
            .send();
        let response = match request {
            Ok(response) => response.error_for_status()?,
            Err(error) if error.is_connect() => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        let requests: Value = response.json()?;
        let Some(snapshot) = &self.snapshot else {
            return Ok(true);
        };
        let mut pages = vec![];
        for request in requests.as_array().into_iter().flatten() {
            let Some(index) = request["pageIndex"]
                .as_i64()
                .filter(|i| *i >= 0)
                .map(|i| i as usize)
            else {
                continue;
            };
            let Some(page) = snapshot.pages.get(index) else {
                continue;
            };
            let zoom = request["zoomLevel"].as_i64().unwrap_or(0) as i32;
            pages.push(json!({"pageIndex":index,"zoomLevel":zoom,"imageData":base64::engine::general_purpose::STANDARD.encode(page.render_image(zoom)?)}));
        }
        if !pages.is_empty() {
            self.client
                .post(self.url("/v3/documentPreview/provideRenderedImages"))
                .json(&json!({"pages":pages}))
                .send()?
                .error_for_status()?;
        }
        Ok(true)
    }
    fn notify(&self) -> Result<bool> {
        let license = match Settings::get().license.unwrap_or(LicenseType::Community) {
            LicenseType::Evaluation => "evaluation",
            LicenseType::Community => "community",
            LicenseType::Professional => "professional",
            LicenseType::Enterprise => "enterprise",
        };
        match self
            .client
            .post(self.url("/v3/notify"))
            .json(&json!({"clientId":self.client_id,"license":license}))
            .send()
        {
            Ok(_) => Ok(true),
            Err(error) if error.is_connect() => Ok(false),
            Err(error) if error.is_timeout() => Ok(true),
            Err(error) => Err(error.into()),
        }
    }
    pub fn run(&mut self, cancel: &AtomicBool) -> Result<()> {
        // Heartbeats run independently of the application's long-poll requests.
        let client = self.client.clone();
        let base = self.base_url.clone();
        let id = self.client_id.clone();
        let stopped = self.stopped.clone();
        let stop = stopped.clone();
        let heartbeat = std::thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(250));
                if stop.load(Ordering::Acquire) {
                    break;
                }
                let license = format!(
                    "{:?}",
                    Settings::get().license.unwrap_or(LicenseType::Community)
                )
                .to_lowercase();
                if let Err(error) = client
                    .post(format!("{base}/v3/notify"))
                    .json(&json!({"clientId":id,"license":license}))
                    .send()
                    && error.is_connect()
                {
                    stop.store(true, Ordering::Release);
                }
            }
        });
        let result = (|| {
            while !cancel.load(Ordering::Acquire) && !stopped.load(Ordering::Acquire) {
                match self.poll_inner(false) {
                    Ok(false) => break,
                    Ok(true) => (),
                    Err(error) => {
                        if error.downcast_ref::<reqwest::Error>().is_none() {
                            return Err(error);
                        }
                        std::thread::sleep(Duration::from_millis(250));
                    }
                }
            }
            Ok(())
        })();
        stopped.store(true, Ordering::Release);
        if heartbeat.join().is_err() {
            bail!("Companion heartbeat thread failed");
        }
        result
    }
}
impl Document {
    pub fn show_in_companion(&self, port: u16) -> Result<()> {
        ensure!(
            self.page_number_groups.is_none(),
            "The QuestPDF Companion does not support merged documents"
        );
        Renderer::new()?.show_in_companion(self, port)
    }
}
impl Renderer {
    pub fn show_in_companion(&mut self, document: &Document, port: u16) -> Result<()> {
        ensure!(
            document.page_number_groups.is_none(),
            "The QuestPDF Companion does not support merged documents"
        );
        let mut session = CompanionSession::connect(&CompanionOptions {
            port,
            ..Default::default()
        })?;
        session.update_document(self, document, false)?;
        session.run(&AtomicBool::new(false))
    }
}
