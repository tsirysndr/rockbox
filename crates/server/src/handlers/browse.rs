use std::{env, fs};

use crate::{
    cache::update_cache,
    http::{Context, Request, Response},
    AUDIO_EXTENSIONS,
};
use anyhow::Error;
use rockbox_sys::types::tree::Entry;

pub async fn get_tree_entries(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let home = env::var("HOME").unwrap();
    let music_library = env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home));

    let path = match req.query_params.get("q") {
        Some(path) => path.as_str().unwrap_or(&music_library),
        None => &music_library,
    };
    let show_hidden = match req.query_params.get("show_hidden") {
        Some(show_hidden) => show_hidden.as_str().unwrap_or("false") == "true",
        None => false,
    };

    if !fs::metadata(path)?.is_dir() {
        res.set_status(500);
        res.text("Path is not a directory");
        return Ok(());
    }

    let mut fs_cache = ctx.fs_cache.lock().await;
    if let Some(entries) = fs_cache.get(&path.to_string()) {
        update_cache(ctx, path, show_hidden);
        res.json(entries);
        return Ok(());
    }

    let mut entries = vec![];

    for file in fs::read_dir(path)? {
        let file = file?;

        if file.metadata()?.is_file()
            && !AUDIO_EXTENSIONS.iter().any(|ext| {
                file.path()
                    .to_string_lossy()
                    .ends_with(&format!(".{}", ext))
            })
        {
            continue;
        }

        if file.file_name().to_string_lossy().starts_with(".") && !show_hidden {
            continue;
        }

        entries.push(Entry {
            name: file.path().to_string_lossy().to_string(),
            time_write: file
                .metadata()?
                .modified()?
                .duration_since(std::time::SystemTime::UNIX_EPOCH)?
                .as_secs() as u32,
            attr: match file.metadata()?.is_dir() {
                true => 0x10,
                false => 0,
            },
            ..Default::default()
        });
    }

    fs_cache.insert(path.to_string(), entries.clone());

    res.json(&entries);
    Ok(())
}

pub fn play_directory(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}
