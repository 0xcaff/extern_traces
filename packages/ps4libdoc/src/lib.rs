use crate::read::ReadExt;
use glob::glob;
use rust_embed::{Embed, RustEmbed};
use serde::Deserialize;
use std::env;
use std::env::VarError;
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;

mod read;

#[derive(Embed)]
#[folder = "defs"]
struct EmbeddedDocs;

pub struct LoadedDocumentation {
    modules: Vec<ModuleDocumentation>,
}

impl LoadedDocumentation {
    pub fn open(path: &str) -> Result<LoadedDocumentation, anyhow::Error> {
        Ok(LoadedDocumentation {
            modules: SharedObjectDocumentation::open_directory(path)?
                .flat_map(|it| it.modules)
                .collect(),
        })
    }

    pub fn bundled() -> Result<LoadedDocumentation, anyhow::Error> {
        Ok(LoadedDocumentation {
            modules: SharedObjectDocumentation::open_embed::<EmbeddedDocs>()?
                .flat_map(|it| it.modules)
                .collect(),
        })
    }

    pub fn try_from_env() -> Result<Option<LoadedDocumentation>, anyhow::Error> {
        let Some(documentation_directory) = path_env("PS4_LIB_DOC_DIR") else {
            return Ok(None);
        };

        let docs = LoadedDocumentation::open(documentation_directory.to_str().unwrap())?;

        Ok(Some(docs))
    }

    pub fn from_env() -> Result<LoadedDocumentation, anyhow::Error> {
        if let Some(docs) = LoadedDocumentation::try_from_env()? {
            return Ok(docs);
        };

        Ok(LoadedDocumentation::bundled()?)
    }

    pub fn lookup(
        &self,
        module_name: &str,
        library_name: &str,
        name_identifier: &str,
    ) -> Option<&SymbolDocumentation> {
        self.modules.iter().find_map(|module_doc| {
            if module_doc.name != module_name {
                return None;
            }

            let library_doc = module_doc
                .libraries
                .iter()
                .find(|library_doc| library_doc.name == library_name)?;

            Some(
                library_doc
                    .symbols
                    .iter()
                    .find(|it| it.encoded_id == name_identifier)?,
            )
        })
    }
}

#[derive(Deserialize)]
pub struct SharedObjectDocumentation {
    #[serde(rename = "shared_object_name")]
    pub export: Option<String>,

    #[serde(rename = "shared_object_names")]
    pub imports: Vec<String>,

    pub modules: Vec<ModuleDocumentation>,
}

#[derive(Deserialize, Debug)]
pub struct ModuleDocumentation {
    pub name: String,
    pub version_major: u8,
    pub version_minor: u8,
    pub libraries: Vec<LibraryDocumentation>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDocumentation {
    pub name: String,
    pub version: u8,
    pub is_export: bool,
    pub symbols: Vec<SymbolDocumentation>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SymbolDocumentation {
    pub encoded_id: String,
    pub name: Option<String>,
}

impl SharedObjectDocumentation {
    fn open_directory(
        path: &str,
    ) -> Result<impl Iterator<Item = SharedObjectDocumentation>, anyhow::Error> {
        Ok(glob(&format!("{}/**/*.json", path))?
            .map(|file_path| -> Result<_, anyhow::Error> {
                let file_path = file_path.unwrap();

                let file = File::open(file_path)?.read_all()?;

                let doc: SharedObjectDocumentation = serde_json::from_slice(&file[3..])?;

                Ok(doc)
            })
            .filter_map(|it| match it {
                Ok(it) => Some(it),
                Err(err) => {
                    println!("error: {}", err);

                    None
                }
            }))
    }

    fn open_embed<T: RustEmbed>(
    ) -> Result<impl Iterator<Item = SharedObjectDocumentation>, anyhow::Error> {
        Ok(T::iter()
            .map(|it| -> Result<_, anyhow::Error> {
                let file = T::get(&it).unwrap();

                let doc: SharedObjectDocumentation = serde_json::from_slice(&file.data[3..])?;

                Ok(doc)
            })
            .filter_map(|it| match it {
                Ok(it) => Some(it),
                Err(err) => {
                    println!("error: {}", err);

                    None
                }
            }))
    }
}

fn path_env(key: impl AsRef<OsStr>) -> Option<PathBuf> {
    match env::var(key) {
        Err(VarError::NotUnicode(value)) => Some(PathBuf::from(value)),
        Ok(value) => Some(PathBuf::from(value)),
        Err(VarError::NotPresent) => None,
    }
}
