use anyhow::Result;

use mini_assets::Locale::ZhHans;
use mini_assets::{scan_categories, Category, I18nStr, Manifest};

fn main() -> Result<()> {
    let manifest = &mut gen_manifest()?;
    let categories = scan_categories(ZhHans)?;

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
                categories_conf.push(Category {
                    id: category.id.clone(),
                    parents: vec![],
                    name: I18nStr::new(vec![ZhHans.value(category.id.clone())]),
                })
            }
        }

        categories_conf
    };

    manifest.categories = categories_conf;

    manifest.override_include_formats()?;
    manifest.fix_categories_parents()?;
    manifest.override_dt_now();
    manifest.save()?;

    Ok(())
}

fn gen_manifest() -> Result<Manifest> {
    if let Some(manifest) = Manifest::load_if()? {
        Ok(manifest)
    } else {
        Ok(Manifest::new())
    }
}
