use anyhow::{Context, Result};
use clap::{App, Arg};

use mini_assets::Locale::ZhHans;
use mini_assets::{
    scan_categories, Category, I18nStr, Image, Manifest, VAR_MINI_ASSETS_OUTPUT,
    VAR_MINI_ASSETS_PREFIX, VAR_MINI_ASSETS_WIDTH, VERSION,
};
use std::path::PathBuf;

const DEFAULT_WIDTH: &str = "200";
const DEFAULT_OUTPUT_DIR: &str = "_images";

fn main() -> Result<()> {
    let matches = build_cli().get_matches();

    let prefix = matches.value_of("prefix").unwrap_or(".");
    let width = matches.value_of("width").unwrap_or(DEFAULT_WIDTH);

    let mut outout_path_buf = PathBuf::from(prefix);
    outout_path_buf.push(DEFAULT_OUTPUT_DIR);
    let output = &outout_path_buf.to_string_lossy().to_string();

    std::env::set_var(VAR_MINI_ASSETS_PREFIX, prefix);
    std::env::set_var(VAR_MINI_ASSETS_WIDTH, width);
    std::env::set_var(VAR_MINI_ASSETS_OUTPUT, output);

    let manifest = &mut gen_manifest().context("Invalid manifest file content")?;

    let categories = scan_categories(ZhHans, vec![DEFAULT_OUTPUT_DIR])?;

    let categories_conf = if manifest.categories.is_empty() {
        // 如果类别配置是空的，直接使用扫描结果。
        categories
            .iter()
            .map(|category| Category {
                id: category.id.clone(),
                parents: vec![],
                name: I18nStr::new(vec![ZhHans.value(category.id.clone())]),
            })
            .collect()
    } else {
        // 如果类别配置不是空的，将剔除已不存在的类别目录，并添加未配置的新类别目录。
        let mut categories_conf: Vec<Category> = manifest
            .categories
            .iter()
            .filter(|category_conf| category_conf.dir_exists())
            .cloned()
            .collect();

        for category in categories {
            if !manifest.category_exists(&category.id) {
                categories_conf.push(category);
            }
        }

        categories_conf
    };

    manifest.categories = categories_conf;

    // 输出每一张图片。
    for category in &manifest.categories {
        for image in category.scan_images()? {
            if let Some(output) = save_image(&image) {
                println!("Save to {}", output.to_string_lossy());
            }
        }
    }

    // 重构并保存清单文件。
    manifest
        .override_include_formats()?
        .override_categories()?
        .override_dt_now()
        .override_version()
        .save()?;

    Ok(())
}

fn gen_manifest() -> Result<Manifest> {
    if let Some(manifest) = Manifest::load()? {
        Ok(manifest)
    } else {
        Ok(Manifest::new())
    }
}

fn save_image(image: &Image) -> Option<PathBuf> {
    match image.output() {
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
        .author("https://mini.telestd.me")
        .about("Generate image verification resources suitable for Policr Mini project")
        .arg(
            Arg::with_name("prefix")
                .value_name("prefix")
                .long("prefix")
                .default_value(".")
                .help("Input path"),
        )
        .arg(
            Arg::with_name("width")
                .value_name("width")
                .long("width")
                .default_value("250")
                .help("The width of images"),
        )
}
