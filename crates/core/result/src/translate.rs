use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fs;
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error as ThisError;

pub type TranslateFunc =
  Box<dyn Fn(&str, &str, &HashMap<String, Value>) -> Result<String, Box<dyn Error>>>;

#[derive(Debug, Deserialize, Serialize)]
struct TranslationElement {
  pub id: String,
  pub tr: String,
}

const I18N_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../i18n");

#[derive(Debug, Clone, ThisError)]
pub enum TranslationError {
  #[error("IO error: {0}")]
  Io(String),
  #[error("JSON parse error: {0}")]
  Json(String),
  #[error("translation store is not initialized")]
  NotInitialized,
  #[error("translation is missing params")]
  MissingParams,
  #[error("translation key is not found: {0}")]
  KeyNotFound(String),
  #[error("template render error: {0}")]
  RenderError(String),
}

impl From<tera::Error> for TranslationError {
  fn from(value: tera::Error) -> Self {
    TranslationError::RenderError(value.to_string())
  }
}

impl From<std::io::Error> for TranslationError {
  fn from(value: std::io::Error) -> Self {
    TranslationError::Io(value.to_string())
  }
}

impl From<serde_json::Error> for TranslationError {
  fn from(value: serde_json::Error) -> Self {
    TranslationError::Json(value.to_string())
  }
}

fn load_translations() -> Result<HashMap<String, HashMap<String, String>>, TranslationError> {
  let mut result = HashMap::new();

  for entry in fs::read_dir(I18N_DIR)? {
    let entry = entry?; // This ? also works now
    let entry_path = entry.path();

    if entry_path.extension().and_then(|ext| ext.to_str()) == Some("json") {
      let lang = entry_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| TranslationError::Io("Invalid filename".to_string()))?
        .to_string();

      let content = fs::read_to_string(&entry_path)?;

      let translations: Vec<TranslationElement> = serde_json::from_str(&content)?;
      println!("the file content {:?}", translations);

      let mut lang_map = HashMap::new();
      for element in translations {
        lang_map.insert(element.id.clone(), element.tr.clone());
      }

      result.insert(lang, lang_map);
    }
  }

  Ok(result)
}

#[derive(Debug)]
struct TemplatePool {
  available: VecDeque<tera::Tera>,
  template_str: String,
  has_vars: bool,
  max_size: usize,
}

impl TemplatePool {
  fn new(template: &str, max_size: usize) -> Self {
    Self {
      available: VecDeque::with_capacity(max_size),
      template_str: template.to_string(),
      has_vars: template.contains("{{") && template.contains("}}"),
      max_size,
    }
  }

  fn get(&mut self) -> Result<tera::Tera, tera::Error> {
    self.available.pop_front().map_or_else(
      || {
        let mut t = tera::Tera::default();
        t.add_raw_template("pooled_template", &self.template_str)?;
        Ok(t)
      },
      Ok,
    )
  }

  fn return_instance(&mut self, instance: tera::Tera) {
    if self.available.len() < self.max_size {
      self.available.push_back(instance);
    }
  }
}

static TRANSLATION_STORE: OnceLock<HashMap<String, HashMap<String, Arc<Mutex<TemplatePool>>>>> =
  OnceLock::new();
static DEFAULT_LANGUAGE: OnceLock<String> = OnceLock::new();
static AVAILABLE_LANGUAGES: OnceLock<Vec<String>> = OnceLock::new();

pub fn translations_init(
  max_pool_size: usize,
  default_language: String,
  available_languages: Vec<String>,
) -> Result<(), TranslationError> {
  let parsed = load_translations()?;
  let mut store = HashMap::new();

  for (lang, lang_trans) in parsed {
    let mut lang_map = HashMap::new();
    for (id, tr) in lang_trans {
      let pool = Arc::new(Mutex::new(TemplatePool::new(&tr, max_pool_size)));
      lang_map.insert(id, pool);
    }
    store.insert(lang, lang_map);
  }

  AVAILABLE_LANGUAGES.set(available_languages).map_err(|_| TranslationError::NotInitialized)?;
  DEFAULT_LANGUAGE.set(default_language.clone()).map_err(|_| TranslationError::NotInitialized)?;
  TRANSLATION_STORE.set(store).map_err(|_| TranslationError::NotInitialized)
}

pub fn tr<P: Serialize>(
  lang: &str,
  id: &str,
  params: Option<P>,
) -> Result<String, TranslationError> {
  let store = TRANSLATION_STORE.get().ok_or(TranslationError::NotInitialized)?;

  let mut lang = lang;
  if !AVAILABLE_LANGUAGES.get().unwrap_or(&vec![]).contains(&lang.to_string()) {
    lang = DEFAULT_LANGUAGE.get().unwrap();
  }

  let pool = store
    .get(lang)
    .and_then(|lang_pools| lang_pools.get(id))
    .ok_or_else(|| TranslationError::KeyNotFound(id.to_string()))?;

  let mut pool_guard = pool.lock().unwrap();
  if pool_guard.has_vars && params.is_none() {
    return Err(TranslationError::MissingParams);
  }

  let tera = pool_guard.get()?;

  let result = match params {
    Some(p) => {
      let context = tera::Context::from_serialize(&p)?;
      tera.render("pooled_template", &context)
    }
    None => {
      // For non-parameterized templates, just return the raw template string
      Ok(pool_guard.template_str.clone())
    }
  }?;

  pool_guard.return_instance(tera);
  Ok(result)
}
