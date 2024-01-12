#[cfg(feature = "image")]
use image::GenericImageView;
use lazy_static::lazy_static;
#[cfg(feature = "magickwand")]
use magick_rust::{magick_wand_genesis, MagickWand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
#[cfg(feature = "magickwand")]
use std::sync::Once;
use std::{env, fs};

use crate::utils::center_dimensions;

pub mod error;
pub mod result;
pub mod utils;
pub use error::Error;

use result::Result;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VAR_MINI_ASSETS_PREFIX: &str = "MINI_ASSETS_PREFIX";
pub const VAR_MINI_ASSETS_OUTPUT: &str = "MINI_ASSETS_OUTPUT";

const MANIFEST_FILE: &str = "Manifest.yaml";

const OUTPUT_MANIFEST_COMMENTS: &str = r#"# 本文件由 mini-assets-gen 生成到输出目录中，请不要修改此文件。
# 若要修改，请改动源 Manifest.yaml 文件以后再重写生成。
"#;

#[cfg(feature = "magickwand")]
static START: Once = Once::new();

lazy_static! {
    static ref PREFIX: String =
        env::var(VAR_MINI_ASSETS_PREFIX).expect("missing variable `MINI_ASSETS_PREFIX`");
    static ref PREFIX_PATH: &'static Path = Path::new(&*PREFIX);
    static ref OUTPUT: String =
        env::var(VAR_MINI_ASSETS_OUTPUT).expect("missing variable `MINI_ASSETS_OUTPUT`");
    static ref OUTPUT_PATH: &'static Path = Path::new(&*OUTPUT);
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
pub struct Album {
    /// 类别 ID。
    pub id: String,
    /// 父级类别的 ID 列表。
    pub parents: Vec<String>,
    /// 国际化名称。
    pub name: I18nStr,
}

impl Album {
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

            images.push(Image::new(self, sub_path.clone())?);
        }

        Ok(images)
    }
}

impl Album {
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
                let extension = extention.to_str().ok_or(Error::InvalidUnicode)?.to_owned();

                extensions.push(extension);
            }
        }

        Ok(extensions)
    }
}

/// 资源根目录的清单配置。
#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// 版本字符串。
    pub version: String,
    /// 生成时间。
    pub datetime: String,
    /// 宽度。
    pub width: usize,
    // 高度
    pub height: usize,
    /// 包含的格式列表。
    pub include_formats: Vec<String>,
    /// 每一个类别的配置。
    pub albums: Vec<Album>,
    // 宽度比例。
    #[serde(skip)]
    width_scale: Option<f64>,
}

impl Manifest {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            datetime: Self::datetime_now(),
            width,
            height,
            ..Default::default()
        }
    }

    /// 从当前目录加载 `manifest.json` 并创建对象，不存在将返回 `OK(None)`。返回 `Error` 表示文件内容或存在其它 IO 错误。
    pub fn load() -> Result<Option<Self>> {
        if let Some(text) = read_manifest()? {
            Ok(Some(serde_yaml::from_str::<Manifest>(&text)?))
        } else {
            Ok(None)
        }
    }

    /// 将当前的对象数据序列化为 YAML 再保存至当前目录的 `Manifest.yaml` 文件。如果此文件存在，将会覆盖原有内容。
    pub fn save(&mut self) -> Result<()> {
        let mut prfix_buf = PathBuf::from(*PREFIX_PATH);
        prfix_buf.push(Path::new(MANIFEST_FILE));
        let file_in_prefix_path = prfix_buf.as_path();

        let yaml = &serde_yaml::to_string(self)?;
        fs::write(file_in_prefix_path, yaml)?;

        let mut output_buf = PathBuf::from(*OUTPUT_PATH);
        fs::create_dir_all(&output_buf)?;

        output_buf.push(Path::new(MANIFEST_FILE));
        let file_in_output_path = output_buf.as_path();

        let output_yaml = format!("{}{}", OUTPUT_MANIFEST_COMMENTS, yaml);

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

        for album in &self.albums {
            for extension in album.scan_extensions()? {
                if !extensions.contains(&extension) {
                    extensions.push(extension);
                }
            }
        }

        self.include_formats = extensions;

        Ok(self)
    }

    /// 类别 ID 是否在对象中存在。
    pub fn album_exists(&self, id: &str) -> bool {
        self.albums.iter().any(|album_conf| album_conf.id == id)
    }

    /// 重写类别列表，可修正一些数据错误。包括父级类别中存在错误的父级引用。
    /// ## 如果存在以下情况，父级将从父级列表中剔除：
    /// - 父级类别等于子类别自己（存在双向引用）。
    /// - 父级类别在类别列表中已不存在。
    pub fn override_albums(&mut self) -> Result<&mut Self> {
        // TODO: 支持修复场景：父级类别的父级引用了子类别（存在循环引用）。

        let mut albums = vec![];

        for album in &self.albums {
            let mut parents = vec![];
            for parent_id in &album.parents {
                if let Some(_parent) = self.get_album(parent_id) {
                    // 存在父级。
                    if &album.id != parent_id {
                        // 父级不是自己。
                        parents.push(parent_id.clone());
                    }
                }
            }

            // TODO: 待优化：直接从数组中删除数据，避免数据克隆开销。
            albums.push(Album {
                parents,
                ..album.clone()
            })
        }

        self.albums = albums;

        Ok(self)
    }

    /// 根据 ID 获取类别。
    fn get_album(&self, id: &str) -> Option<&Album> {
        self.albums.iter().find(|album| album.id == id)
    }

    pub fn width_scale(&mut self) -> f64 {
        if let Some(width_scale) = self.width_scale {
            width_scale
        } else {
            let width_scale = self.width as f64 / self.height as f64;
            self.width_scale.replace(width_scale);

            width_scale
        }
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
pub fn scan_albums(default_locale: Locale, skips: Vec<&str>) -> Result<Vec<Album>> {
    if (*PREFIX_PATH).is_dir() {
        let mut albums = vec![];

        for entry in fs::read_dir(*PREFIX_PATH)? {
            let entry = entry?;
            let sub_path = entry.path();

            if !sub_path.is_dir() {
                continue;
            }

            let dir_name = if let Some(file_name) = sub_path.file_name() {
                let dir_name = file_name.to_str().ok_or(Error::InvalidUnicode)?;
                if skips.contains(&dir_name) {
                    continue;
                }

                dir_name
            } else {
                // TODO: 输出警告日志：此目录名称不正常已略过。
                continue;
            };

            albums.push(Album {
                id: dir_name.to_owned(),
                name: I18nStr::new(vec![default_locale.value(dir_name)]),
                parents: vec![],
            })
        }

        Ok(albums)
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
    album: &'a Album,
    file: PathBuf,
    extension: String,
    digest: String,
}

impl<'a> Image<'a> {
    pub fn new(album: &'a Album, file: PathBuf) -> Result<Self> {
        let extension = file
            .extension()
            .ok_or( Error::MissingExtension(file.to_str_ext()?.to_owned()))?
            .to_str()
            .ok_or(Error::InvalidUnicode)?
            .to_owned();

        let digest = Self::make_digest(&file)?;

        Ok(Self {
            album,
            file,
            extension,
            digest,
        })
    }

    /// 生成文件摘要。
    fn make_digest(file: &Path) -> Result<String> {
        let mut buf = Vec::new();
        {
            let f = File::open(file)?;

            let mut reader = BufReader::new(f);
            let r = reader.read_to_end(&mut buf)?;

            if r == 0 {
                return Err(Error::EmptyImage(
                    file.to_str().ok_or(Error::InvalidUnicode)?.to_owned(),
                ));
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(&buf);

        let result = hasher.finalize();

        let hash = hex::encode(result);

        Ok(hash[0..15].to_string())
    }

    /// 将文件保存到输出目录，包括压制等处理过程。
    #[cfg(feature = "image")]
    pub fn output(&self, manifest: &mut Manifest) -> Result<PathBuf> {
        let mut output_buf = PathBuf::from(*OUTPUT_PATH);
        output_buf.push(Path::new(&self.album.id));
        fs::create_dir_all(&output_buf)?;
        output_buf.push(Path::new(&format!("{}.{}", self.digest, self.extension)));

        let mut img = image::open(&self.file)?;
        let dimensions = img.dimensions();

        let crop_area = center_dimensions(dimensions, manifest.width_scale());

        // println!("{}: {:?}", self.file.to_str_ext()?, crop_area);

        // 按照特定尺寸比例，居中切割图片。
        // 按照指定尺寸缩放图片。
        img = img
            .crop(crop_area.x, crop_area.y, crop_area.width, crop_area.height)
            .resize_exact(
                manifest.width as u32,
                manifest.height as u32,
                image::imageops::Lanczos3,
            );

        img.save(&output_buf)?;

        Ok(output_buf)
    }

    #[cfg(feature = "magickwand")]
    pub fn output(&self, manifest: &mut Manifest) -> Result<PathBuf> {
        START.call_once(|| {
            magick_wand_genesis();
        });

        let mut output_buf = PathBuf::from(*OUTPUT_PATH);
        output_buf.push(Path::new(&self.album.id));
        fs::create_dir_all(&output_buf)?;
        output_buf.push(Path::new(&format!("{}.{}", self.digest, self.extension)));

        let wand = MagickWand::new();
        wand.read_image(self.file.to_str_ext()?)?;

        let width = wand.get_image_width();
        let height = wand.get_image_height();

        // 按照特定尺寸比例，居中切割图片。
        let crop_area = center_dimensions((width as u32, height as u32), manifest.width_scale());
        wand.crop_image(
            crop_area.width as usize,
            crop_area.height as usize,
            crop_area.x as isize,
            crop_area.y as isize,
        )?;

        // 按照指定尺寸缩放图片。
        wand.resize_image(
            manifest.width,
            manifest.height,
            magick_rust::bindings::FilterType_LanczosFilter,
        );
        wand.write_image_blob(self.extension.as_str())?;
        wand.write_image(output_buf.to_str_ext()?)?;

        Ok(output_buf)
    }
}

pub trait PathExt {
    fn to_str_ext(&self) -> Result<&str>;
}

impl PathExt for std::path::PathBuf {
    fn to_str_ext(&self) -> Result<&str> {
        self.to_str().ok_or(Error::InvalidUnicode)
    }
}

pub trait InputExt {
    fn parse_int_input(&self) -> Result<usize>;
}

impl InputExt for &str {
    fn parse_int_input(&self) -> Result<usize> {
        self.parse::<usize>()
            .map_err(|_| Error::InvalidIntInput(self.to_string()))
    }
}
