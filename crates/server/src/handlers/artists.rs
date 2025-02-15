use anyhow::Error;
use rockbox_library::repo;

use crate::http::{Context, Request, Response};

pub async fn get_artists(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let artists = repo::artist::all(ctx.pool.clone()).await?;
    res.json(&artists);
    Ok(())
}

pub async fn get_artist(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let artist = repo::artist::find(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&artist);
    Ok(())
}

pub async fn get_artist_albums(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let albums = repo::album::find_by_artist(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&albums);
    Ok(())
}

pub async fn get_artist_tracks(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let tracks = repo::artist_tracks::find_by_artist(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&tracks);
    Ok(())
}

pub async fn play_artist_tracks(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

