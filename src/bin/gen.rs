use anyhow::{Context, Result};
use clap::{App, Arg};

use mini_assets::{
    scan_albums, Album, I18nStr, Image, InputExt, Locale::ZhHans, Manifest, PathExt,
    VAR_MINI_ASSETS_OUTPUT, VAR_MINI_ASSETS_PREFIX, VERSION,
};
use std::path::PathBuf;

const DEFAULT_PREFIX: &str = ".";
const DEFAULT_WIDTH: &str = "240";
const DEFAULT_HEIGHT: &str = "160";
const DEFAULT_OUTPUT_DIR: &str = "_albums";

fn main() -> Result<()> {
    let matches = build_cli().get_matches();

    // 从命令行读取参数。
    let prefix = matches.value_of("prefix").unwrap_or(DEFAULT_PREFIX);
    let width = matches
        .value_of("width")
        .unwrap_or(DEFAULT_WIDTH)
        .parse_int_input()?;
    let height = matches
        .value_of("height")
        .unwrap_or(DEFAULT_HEIGHT)
        .parse_int_input()?;

    // 初始化输出目录路径。
    let mut outout_path = PathBuf::from(prefix);
    outout_path.push(DEFAULT_OUTPUT_DIR);
    let output = outout_path.to_str_ext()?;

    // 使用环境变量全局保存。
    std::env::set_var(VAR_MINI_ASSETS_PREFIX, prefix);
    std::env::set_var(VAR_MINI_ASSETS_OUTPUT, output);

    let manifest = &mut gen_manifest(width, height).context("invalid manifest file content")?;

    let albums = scan_albums(ZhHans, vec![DEFAULT_OUTPUT_DIR])?;

    let albums_conf = if manifest.albums.is_empty() {
        // 如果类别配置是空的，直接使用扫描结果。
        albums
            .iter()
            .map(|album| Album {
                id: album.id.clone(),
                parents: vec![],
                name: I18nStr::new(vec![ZhHans.value(album.id.clone())]),
            })
            .collect()
    } else {
        // 如果类别配置不是空的，将剔除已不存在的类别目录，并添加未配置的新类别目录。
        let mut albums_conf: Vec<Album> = manifest
            .albums
            .iter()
            .filter(|album_conf| album_conf.dir_exists())
            .cloned()
            .collect();

        for album in albums {
            if !manifest.album_exists(&album.id) {
                albums_conf.push(album);
            }
        }

        albums_conf
    };

    manifest.albums = albums_conf.clone();

    // println!("manifest: {:?}", manifest);

    // 输出每一张图片。
    for album in albums_conf {
        for image in album.scan_images()? {
            if let Some(output) = save_image(&image, manifest) {
                println!("Save to {}", output.to_string_lossy());
            }
        }
    }

    // 重构并保存清单文件。
    manifest
        .override_include_formats()?
        .override_albums()?
        .override_dt_now()
        .override_version()
        .save()?;

    Ok(())
}

fn gen_manifest(width: usize, height: usize) -> Result<Manifest> {
    if let Some(manifest) = Manifest::load()? {
        Ok(manifest)
    } else {
        Ok(Manifest::new(width, height))
    }
}

fn save_image(image: &Image, manifest: &mut Manifest) -> Option<PathBuf> {
    match image.output(manifest) {
        Err(_e) => {
            // 此图片输出出错，输出错误日志。

            None
        }
        Ok(path) => Some(path),
    }
}

fn build_cli() -> App<'static, 'static> {
    App::new("mini-assets-gen")
        .version(VERSION)
        .author("https://mini.gramlabs.org")
        .about("Generate image verification resources suitable for Policr Mini project")
        .arg(
            Arg::with_name("prefix")
                .value_name("prefix")
                .long("prefix")
                .default_value(DEFAULT_PREFIX)
                .help("Input path"),
        )
        .arg(
            Arg::with_name("width")
                .value_name("width")
                .long("width")
                .default_value(DEFAULT_WIDTH)
                .help("The width of images"),
        )
        .arg(
            Arg::with_name("height")
                .value_name("height")
                .long("height")
                .default_value(DEFAULT_HEIGHT)
                .help("The height of images"),
        )
}
