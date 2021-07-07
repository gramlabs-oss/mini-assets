use lazy_static::lazy_static;
use magick_rust::{magick_wand_genesis, MagickWand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::{env, fs};

pub mod error;
pub mod result;
pub use error::Error;

use result::Result;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VAR_MINI_ASSETS_PREFIX: &str = "MINI_ASSETS_PREFIX";
pub const VAR_MINI_ASSETS_OUTPUT: &str = "MINI_ASSETS_OUTPUT";
pub const VAR_MINI_ASSETS_WIDTH: &str = "MINI_ASSETS_WIDTH";

const MANIFEST_FILE: &str = "Manifest.yaml";

const OUTPUT_MANIFEST_COMMENTS: &str = r#"# 本文件由 mini-assets-gen 生成到输出目录中，请不要修改此文件。
# 若要修改，请改动源 Manifest.yaml 文件以后再重写生成。
"#;

static START: Once = Once::new();

lazy_static! {
    static ref PREFIX: String =
        env::var(VAR_MINI_ASSETS_PREFIX).expect("missing variable `MINI_ASSETS_PREFIX`");
    static ref PREFIX_PATH: &'static Path = Path::new(&*PREFIX);
    static ref OUTPUT: String =
        env::var(VAR_MINI_ASSETS_OUTPUT).expect("missing variable `MINI_ASSETS_OUTPUT`");
    static ref OUTPUT_PATH: &'static Path = Path::new(&*OUTPUT);
    static ref WIDTH_STR: String =
        env::var(VAR_MINI_ASSETS_WIDTH).expect("missing variable `MINI_ASSETS_WIDTH`");
    static ref WIDTH: usize = {
        (*WIDTH_STR)
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("the width value `{}` is invalid", *WIDTH_STR))
    };
}

/// 一个本地化的值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleValue {
    locale: Locale,
    value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    ZhHans,
    ZhHant,
    En,
}

impl Locale {
    pub fn value<S: Into<String>>(self, value: S) -> LocaleValue {
        LocaleValue {
            locale: self,
            value: value.into(),
        }
    }
}

/// 国际化字符串，支持添加 `zh-hans` / `zh-hant` / `en` 三种语言文本。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct I18nStr {
    #[serde(rename = "zh-hans")]
    zh_hans: Option<String>,
    #[serde(rename = "zh-hant")]
    zh_hant: Option<String>,
    en: Option<String>,
}

impl I18nStr {
    /// 通过 [`LocaleValue`](enum.LocaleValue.html) 列表创建一个国际化字符串。
    pub fn new(locale_values: Vec<LocaleValue>) -> Self {
        let mut i18n_str = Self {
            zh_hans: None,
            zh_hant: None,
            en: None,
        };

        for locale_value in locale_values.into_iter() {
            i18n_str.localize(locale_value);
        }

        i18n_str
    }

    /// 包含一个本地化的值。
    pub fn localize(&mut self, locale_value: LocaleValue) -> &Self {
        match locale_value.locale {
            Locale::ZhHans => self.zh_hans = Some(locale_value.value),
            Locale::ZhHant => self.zh_hant = Some(locale_value.value),
            Locale::En => self.en = Some(locale_value.value),
        }

        self
    }
}

/// 单个类别和其它类别的关系。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    /// 类别 ID。
    pub id: String,
    /// 父级类别的 ID 列表。
    pub parents: Vec<String>,
    /// 国际化名称。
    pub name: I18nStr,
}

impl Category {
    /// 配置中的类别目录是否还存在。
    pub fn dir_exists(&self) -> bool {
        let mut path_buf = PathBuf::from(*PREFIX_PATH);
        path_buf.push(Path::new(&self.id));

        path_buf.exists()
    }

    /// 扫描此类别下的所有图片。
    pub fn scan_images(&self) -> Result<Vec<Image>> {
        let mut path_buf = PathBuf::from(*PREFIX_PATH);
        path_buf.push(Path::new(&self.id));

        let mut images = vec![];

        for entry in fs::read_dir(path_buf)? {
            let entry = entry?;
            let sub_path = entry.path();

            if !sub_path.is_file() {
                continue;
            }

            if let Some(image) = Image::new(self, sub_path.clone()) {
                images.push(image);
            }
        }

        Ok(images)
    }
}

impl Category {
    /// 扫描所有子文件的后缀名。
    pub fn scan_extensions(&self) -> Result<Vec<String>> {
        let mut path_buf = PathBuf::from(*PREFIX_PATH);
        path_buf.push(Path::new(&self.id));

        let mut extensions = vec![];

        for entry in fs::read_dir(path_buf)? {
            let entry = entry?;
            let sub_path = entry.path();

            if !sub_path.is_file() {
                continue;
            }

            if let Some(extention) = sub_path.extension() {
                let extension = extention.to_string_lossy().to_string();

                extensions.push(extension);
            }
        }

        Ok(extensions)
    }
}

/// 资源根目录的清单配置。
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// 版本字符串。
    version: String,
    /// 生成时间。
    pub datetime: String,
    /// 宽度。
    width: usize,
    /// 包含的格式列表。
    pub include_formats: Vec<String>,
    /// 每一个类别的配置。
    pub categories: Vec<Category>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            datetime: Self::datetime_now(),
            width: *WIDTH,
            ..Default::default()
        }
    }

    /// 从当前目录加载 `manifest.json` 并创建对象，不存在将返回 `OK(None)`。返回 `Error` 表示文件内容或存在其它 IO 错误。
    pub fn load() -> Result<Option<Self>> {
        if let Some(strdata) = read_manifest()? {
            Ok(Some(serde_yaml::from_str::<Manifest>(&strdata)?))
        } else {
            Ok(None)
        }
    }

    /// 将当前的对象数据序列化为 YAML 再保存至当前目录的 `Manifest.yaml` 文件。如果此文件存在，将会覆盖原有内容。
    pub fn save(&mut self) -> Result<()> {
        self.width = *WIDTH;

        let mut prfix_buf = PathBuf::from(*PREFIX_PATH);
        prfix_buf.push(Path::new(MANIFEST_FILE));
        let file_in_prefix_path = prfix_buf.as_path();

        let yaml = &serde_yaml::to_string(self)?;
        fs::write(file_in_prefix_path, yaml)?;

        let mut output_buf = PathBuf::from(*OUTPUT_PATH);
        fs::create_dir_all(&output_buf)?;

        output_buf.push(Path::new(MANIFEST_FILE));
        let file_in_output_path = output_buf.as_path();

        let output_yaml = format!("{}{}", &*OUTPUT_MANIFEST_COMMENTS, yaml);

        fs::write(file_in_output_path, output_yaml)?;

        Ok(())
    }

    /// 重写日期时间当此刻。
    pub fn override_dt_now(&mut self) -> &mut Self {
        self.datetime = Self::datetime_now();

        self
    }

    /// 重写为当前版本。
    pub fn override_version(&mut self) -> &mut Self {
        self.version = (*VERSION).to_string();

        self
    }

    fn datetime_now() -> String {
        use chrono::SecondsFormat;

        chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
    }

    /// 根据当前的对象数据，重写扫描并重写已包含的文件格式列表。
    pub fn override_include_formats(&mut self) -> Result<&mut Self> {
        let mut extensions = vec![];

        for category in &self.categories {
            for extension in category.scan_extensions()? {
                if !extensions.contains(&extension) {
                    extensions.push(extension);
                }
            }
        }

        self.include_formats = extensions;

        Ok(self)
    }

    /// 类别 ID 是否在对象中存在。
    pub fn category_exists(&self, id: &str) -> bool {
        self.categories
            .iter()
            .any(|category_conf| category_conf.id == id)
    }

    /// 重写类别列表，可修正一些数据错误。包括父级类别中存在错误的父级引用。
    /// ## 如果存在以下情况，父级将从父级列表中剔除：
    /// - 父级类别等于子类别自己（存在双向引用）。
    /// - 父级类别在类别列表中已不存在。
    pub fn override_categories(&mut self) -> Result<&mut Self> {
        // TODO: 支持修复场景：父级类别的父级引用了子类别（存在循环引用）。

        let mut categories = vec![];

        for category in &self.categories {
            let mut parents = vec![];
            for parent_id in &category.parents {
                if let Some(_parent) = self.get_category(parent_id) {
                    // 存在父级。
                    if &category.id != parent_id {
                        // 父级不是自己。
                        parents.push(parent_id.clone());
                    }
                }
            }

            // TODO: 待优化：直接从数组中删除数据，避免数据克隆开销。
            categories.push(Category {
                parents,
                ..category.clone()
            })
        }

        self.categories = categories;

        Ok(self)
    }

    /// 根据 ID 获取类别。
    fn get_category(&self, id: &str) -> Option<&Category> {
        self.categories.iter().find(|category| category.id == id)
    }
}

/// 从当前目录读取 `manifest.json` 文件内容，不存在将返回 `OK(None)`。返回 `Error` 表示出现了其它 IO 错误。
fn read_manifest() -> Result<Option<String>> {
    let mut path_buf = PathBuf::from(*PREFIX_PATH);
    path_buf.push(Path::new(MANIFEST_FILE));
    let path = path_buf.as_path();

    match fs::read_to_string(path) {
        Ok(strdata) => Ok(Some(strdata)),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(error::Error::IO(e))
            }
        }
    }
}

/// 从指定路径扫描并生成类别列表。
pub fn scan_categories(default_locale: Locale, skips: Vec<&str>) -> Result<Vec<Category>> {
    if (*PREFIX_PATH).is_dir() {
        let mut categories = vec![];

        for entry in fs::read_dir(*PREFIX_PATH)? {
            let entry = entry?;
            let sub_path = entry.path();

            if !sub_path.is_dir() {
                continue;
            }

            let dir_name = if let Some(file_name) = sub_path.file_name() {
                let dir_name = file_name.to_string_lossy().to_string();
                if skips.contains(&dir_name.as_str()) {
                    continue;
                }

                dir_name
            } else {
                // TODO: 输出警告日志：此目录名称不正常已略过。
                continue;
            };

            categories.push(Category {
                id: dir_name.clone(),
                name: I18nStr::new(vec![default_locale.value(dir_name)]),
                parents: vec![],
            })
        }

        Ok(categories)
    } else {
        Err(Error::NonFolder((*PREFIX).clone()))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Dimension {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// 单张图片。
#[derive(Debug, PartialEq, Eq)]
pub struct Image<'a> {
    category: &'a Category,
    file: PathBuf,
    extension: String,
    digest: String,
}

impl<'a> Image<'a> {
    /// 创建一张图片，如果它并非一张图片文件或计算 hash 时发送错误，将返回 `None`。
    pub fn new(category: &'a Category, file: PathBuf) -> Option<Self> {
        if let Some(extension) = file.extension() {
            let extension = extension.to_string_lossy().to_string();

            if let Ok(digest) = Self::make_digest(&file) {
                return Some(Self {
                    category,
                    file,
                    extension,
                    digest,
                });
            }
        }

        None
    }

    /// 生成文件摘要。
    fn make_digest(file: &Path) -> Result<String> {
        let mut buf = Vec::new();
        {
            let f = File::open(file)?;

            let mut reader = BufReader::new(f);
            let r = reader.read_to_end(&mut buf)?;

            if r == 0 {
                return Err(Error::EmptyImage(file.to_string_lossy().to_string()));
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(&buf);

        let result = hasher.finalize();

        let hash = hex::encode(result);

        Ok(hash[0..15].to_string())
    }

    /// 将文件保存到输出目录，包括压制等处理过程。
    pub fn output(&self) -> Result<PathBuf> {
        START.call_once(|| {
            magick_wand_genesis();
        });

        let mut output_buf = PathBuf::from(*OUTPUT_PATH);
        output_buf.push(Path::new(&self.category.id));
        fs::create_dir_all(&output_buf)?;
        output_buf.push(Path::new(&format!("{}.{}", self.digest, self.extension)));

        let wand = MagickWand::new();
        wand.read_image(&self.file.to_string_lossy())
            .map_err(|err| Error::Message(err.to_owned()))?;
        let ratio = (*WIDTH as f64) / (wand.get_image_width() as f64);
        let hegiht = (wand.get_image_height() as f64 * ratio) as usize;

        wand.adaptive_resize_image(*WIDTH, hegiht)?;
        wand.write_image_blob(&self.extension)?;
        wand.write_image(&output_buf.to_string_lossy())?;

        Ok(output_buf)
    }
}
