//! Fluent PDF operations correspond to Fluent/DocumentOperation.cs.
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
#[derive(Clone, Default)]
pub struct LayerConfiguration {
    pub file_path: PathBuf,
    pub target_pages: Option<String>,
    pub source_pages: Option<String>,
    pub repeat_source_pages: Option<String>,
}
#[derive(Clone, Copy)]
pub enum DocumentAttachmentRelationship {
    Data,
    Source,
    Alternative,
    Supplement,
    Unspecified,
}
#[derive(Clone)]
pub struct DocumentAttachment {
    pub file_path: PathBuf,
    pub key: Option<String>,
    pub attachment_name: Option<String>,
    pub creation_date: Option<chrono::DateTime<chrono::Utc>>,
    pub modification_date: Option<chrono::DateTime<chrono::Utc>>,
    pub mime_type: Option<String>,
    pub description: Option<String>,
    pub replace: bool,
    pub relationship: Option<DocumentAttachmentRelationship>,
}
impl Default for DocumentAttachment {
    fn default() -> Self {
        Self {
            file_path: PathBuf::new(),
            key: None,
            attachment_name: None,
            creation_date: None,
            modification_date: None,
            mime_type: None,
            description: None,
            replace: true,
            relationship: None,
        }
    }
}
#[derive(Clone)]
pub struct Encryption {
    pub bits: u16,
    pub user_password: String,
    pub owner_password: String,
    pub allow_annotation: bool,
    pub allow_assembly: bool,
    pub allow_content_extraction: bool,
    pub allow_filling_forms: bool,
    pub allow_modification: bool,
    pub allow_printing: bool,
    pub encrypt_metadata: bool,
}
impl Default for Encryption {
    fn default() -> Self {
        Self {
            bits: 256,
            user_password: String::new(),
            owner_password: String::new(),
            allow_annotation: true,
            allow_assembly: true,
            allow_content_extraction: true,
            allow_filling_forms: true,
            allow_modification: true,
            allow_printing: true,
            encrypt_metadata: true,
        }
    }
}
pub struct DocumentOperation {
    configuration: Value,
    metadata_extensions: Vec<String>,
}
impl DocumentOperation {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            configuration: json!({"inputFile":resolve(path)?,"allowWeakCrypto":"","newlineBeforeEndstream":"","keepFilesOpen":"n"}),
            metadata_extensions: vec![],
        })
    }
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.configuration["password"] = password.into().into();
        self
    }
    pub fn extend_metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata_extensions.push(metadata.into());
        self
    }
    fn append(&mut self, key: &str, value: Value) {
        if self.configuration[key].is_null() {
            self.configuration[key] = json!([])
        }
        self.configuration[key].as_array_mut().unwrap().push(value)
    }
    pub fn take_pages(mut self, selector: impl Into<String>) -> Self {
        self.append("pages", json!({"file":".","range":selector.into()}));
        self
    }
    pub fn merge_file(mut self, path: impl AsRef<Path>, selector: Option<&str>) -> Result<Self> {
        if self.configuration["pages"].is_null() {
            self = self.take_pages("1-z")
        }
        self.append(
            "pages",
            json!({"file":resolve(path)?,"range":selector.unwrap_or("1-z")}),
        );
        Ok(self)
    }
    fn layer(mut self, key: &str, config: LayerConfiguration) -> Result<Self> {
        let mut value = json!({"file":resolve(config.file_path)?});
        for (k, v) in [
            ("to", config.target_pages),
            ("from", config.source_pages),
            ("repeat", config.repeat_source_pages),
        ] {
            if let Some(v) = v {
                value[k] = v.into()
            }
        }
        self.append(key, value);
        Ok(self)
    }
    pub fn overlay_file(self, config: LayerConfiguration) -> Result<Self> {
        self.layer("overlay", config)
    }
    pub fn underlay_file(self, config: LayerConfiguration) -> Result<Self> {
        self.layer("underlay", config)
    }
    pub fn decrypt(mut self) -> Self {
        self.configuration["decrypt"] = "".into();
        self
    }
    pub fn remove_restrictions(mut self) -> Self {
        self.configuration["decrypt"] = "".into();
        self.configuration["removeRestrictions"] = "".into();
        self
    }
    pub fn linearize(mut self) -> Self {
        self.configuration["linearize"] = "".into();
        self
    }
    pub fn encrypt(mut self, encryption: Encryption) -> Result<Self> {
        let e = encryption;
        if !self.configuration["encrypt"].is_null() {
            bail!("Encryption process can be set only once")
        }
        let flag = |b| if b { "y" } else { "n" };
        let mut options =
            json!({"annotate":flag(e.allow_annotation),"extract":flag(e.allow_content_extraction)});
        match e.bits {
            40 => {
                if !e.allow_modification {
                    options["modify"] = "none".into()
                }
                if !e.allow_printing {
                    options["print"] = "none".into()
                }
            }
            128 | 256 => {
                options["assemble"] = flag(e.allow_assembly).into();
                options["form"] = flag(e.allow_filling_forms).into();
                options["modifyOther"] = flag(e.allow_modification).into();
                options["print"] = if e.allow_printing { "full" } else { "none" }.into();
                if e.bits == 128 {
                    options["useAes"] = "y".into()
                }
                if !e.encrypt_metadata {
                    options["cleartextMetadata"] = "".into()
                }
            }
            _ => bail!("Encryption must use 40, 128 or 256 bits"),
        }
        let mut config = json!({"userPassword":e.user_password,"ownerPassword":e.owner_password});
        config[format!("{}bit", e.bits)] = options;
        self.configuration["encrypt"] = config;
        Ok(self)
    }
    pub fn add_attachment(mut self, attachment: DocumentAttachment) -> Result<Self> {
        use chrono::Timelike;
        let file = resolve(&attachment.file_path)?;
        let name = attachment
            .file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let metadata = std::fs::metadata(&file)?;
        let date = |date: Option<chrono::DateTime<chrono::Utc>>,
                    fallback: std::io::Result<std::time::SystemTime>| {
            let date = date
                .unwrap_or_else(|| fallback.unwrap_or(std::time::SystemTime::UNIX_EPOCH).into());
            format!("D:{}{:02}Z", date.format("%Y%m%d%H%M"), date.second())
        };
        let mime = attachment.mime_type.unwrap_or_else(|| {
            let extension = attachment
                .file_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            include_str!("../resources/MimeTypes.csv")
                .lines()
                .filter_map(|l| l.split_once(','))
                .find(|(ext, _)| *ext == extension)
                .map(|(_, mime)| mime)
                .unwrap_or("text/plain")
                .into()
        });
        let mut value = json!({"key":attachment.key.unwrap_or(name.clone()),"file":file,"filename":attachment.attachment_name.unwrap_or(name),"creationdate":date(attachment.creation_date,metadata.created()),"moddate":date(attachment.modification_date,metadata.modified()),"mimetype":mime});
        if let Some(v) = attachment.description {
            value["description"] = v.into()
        }
        if attachment.replace {
            value["replace"] = "".into()
        }
        if let Some(r) = attachment.relationship {
            value["relationship"] = match r {
                DocumentAttachmentRelationship::Data => "/Data",
                DocumentAttachmentRelationship::Source => "/Source",
                DocumentAttachmentRelationship::Alternative => "/Alternative",
                DocumentAttachmentRelationship::Supplement => "/Supplement",
                DocumentAttachmentRelationship::Unspecified => "/Unspecified",
            }
            .into()
        }
        self.append("addAttachment", value);
        Ok(self)
    }
    pub fn save(mut self, path: impl AsRef<Path>) -> Result<()> {
        let target = std::path::absolute(path)?;
        if self.configuration["inputFile"]
            .as_str()
            .is_some_and(|s| Path::new(s) == target)
        {
            bail!("Save PDF operations to a different path from the input")
        }
        self.configuration["outputFile"] = target.to_string_lossy().to_string().into();
        let transform = |bytes: &[u8]| crate::xmp::extend(bytes, &self.metadata_extensions);
        crate::qpdf::execute_job(
            &self.configuration,
            if self.metadata_extensions.is_empty() {
                None
            } else {
                Some(&transform)
            },
        )
    }
}
fn resolve(path: impl AsRef<Path>) -> Result<String> {
    let path = crate::runtime::resolve_resource_file_path(path)?.canonicalize()?;
    let value = path.to_string_lossy();
    Ok(value.strip_prefix(r"\\?\").unwrap_or(&value).to_string())
}
