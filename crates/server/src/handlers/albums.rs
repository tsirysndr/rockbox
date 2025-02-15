use anyhow::Error;
use rockbox_library::repo;

use crate::http::{Context, Request, Response};

pub async fn get_albums(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let albums = repo::album::all(ctx.pool.clone()).await?;
    res.json(&albums);
    Ok(())
}

pub async fn get_album(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let album = repo::album::find(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&album);
    Ok(())
}

pub async fn get_album_tracks(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let tracks = repo::album_tracks::find_by_album(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&tracks);
    Ok(())
}

pub async fn play_album(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}
