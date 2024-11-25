use std::{env, thread};

use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image, Label, Scale};

use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::{
    CurrentTrackRequest, CurrentTrackResponse, PlayRequest, StreamCurrentTrackRequest,
};
use crate::time::format_milliseconds;
use crate::types::track::Track;
use std::cell::RefCell;
use tokio::sync::mpsc;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "gtk/media_controls.ui")]
    pub struct MediaControls {
        #[template_child]
        pub shuffle_button: TemplateChild<Button>,
        #[template_child]
        pub previous_button: TemplateChild<Button>,
        #[template_child]
        pub play_pause_button: TemplateChild<Button>,
        #[template_child]
        pub next_button: TemplateChild<Button>,
        #[template_child]
        pub repeat_button: TemplateChild<Button>,
        #[template_child]
        pub album_art: TemplateChild<Image>,
        #[template_child]
        pub current_song_details: TemplateChild<gtk::Box>,
        #[template_child]
        pub title: TemplateChild<Label>,
        #[template_child]
        pub artist_album: TemplateChild<Label>,
        #[template_child]
        pub elapsed: TemplateChild<Label>,
        #[template_child]
        pub duration: TemplateChild<Label>,
        #[template_child]
        pub media_control_bar_progress: TemplateChild<gtk::Box>,
        #[template_child]
        pub progress_bar: TemplateChild<Scale>,

        pub current_track: RefCell<Option<Track>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MediaControls {
        const NAME: &'static str = "MediaControls";
        type ParentType = gtk::Box;
        type Type = super::MediaControls;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MediaControls {
        fn constructed(&self) {
            self.parent_constructed();

            let self_weak = self.downgrade();
            self.progress_bar
                .connect_change_value(move |_, _scroll_type, value| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return glib::Propagation::Stop,
                    };
                    let current_track = self_.current_track.borrow();
                    if let Some(track) = &*current_track {
                        let elapsed = (track.duration as i64 * value as i64) / 100;
                        glib::MainContext::default().spawn_local(async move {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let _ = rt.block_on(async {
                                let url = build_url();
                                let mut client = PlaybackServiceClient::connect(url).await?;
                                client.play(PlayRequest { elapsed, offset: 0 }).await?;
                                Ok::<(), Error>(())
                            });
                        });
                    }
                    glib::Propagation::Stop
                });

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                let (tx, mut rx) = mpsc::channel(32);

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.load_current_track(tx);
                });

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    while let Some(track) = rx.recv().await {
                        let title = self_.title.get();
                        let artist_album = self_.artist_album.get();
                        let elapsed = self_.elapsed.get();
                        let duration = self_.duration.get();
                        let album_art = self_.album_art.get();
                        let media_control_bar_progress = self_.media_control_bar_progress.get();
                        let progress_bar = self_.progress_bar.get();

                        if track.length == 0 {
                            continue;
                        }

                        let progression = (track.elapsed as f64 / track.length as f64) * 100.0;
                        progress_bar.set_value(progression);
                        media_control_bar_progress.set_visible(true);

                        title.set_text(&track.title);
                        artist_album.set_text(&format!("{} - {}", track.artist, track.album));
                        elapsed.set_text(&format_milliseconds(track.elapsed));
                        duration.set_text(&format_milliseconds(track.length));

                        if let Some(filename) = track.album_art {
                            let home = std::env::var("HOME").unwrap();
                            let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                            album_art.set_from_file(Some(&path));
                        }

                        self_.set_current_track(Track {
                            title: track.title,
                            artist: track.artist,
                            album: track.album,
                            album_artist: track.album_artist,
                            duration: track.length,
                            elapsed: track.elapsed,
                            ..Default::default()
                        });
                    }
                });

                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for MediaControls {}
    impl BoxImpl for MediaControls {}

    impl MediaControls {
        pub fn set_current_track(&self, track: Track) {
            let mut current_track = self.current_track.borrow_mut();
            *current_track = Some(track);
        }
    }
}

glib::wrapper! {
  pub struct MediaControls(ObjectSubclass<imp::MediaControls>)
    @extends gtk::Widget, gtk::Box;
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}

#[gtk::template_callbacks]
impl MediaControls {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_current_track(&self, tx: mpsc::Sender<CurrentTrackResponse>) {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                let mut stream = client
                    .stream_current_track(StreamCurrentTrackRequest {})
                    .await?
                    .into_inner();

                while let Some(track) = stream.message().await? {
                    tx.send(track).await?;
                }

                Ok::<(), Error>(())
            });
        });

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                let response = client.current_track(CurrentTrackRequest {}).await?;
                Ok::<CurrentTrackResponse, Error>(response.into_inner())
            })
        });
        if let Ok(track) = handle.join().unwrap() {
            let title = self.imp().title.get();
            let artist_album = self.imp().artist_album.get();
            let elapsed = self.imp().elapsed.get();
            let duration = self.imp().duration.get();
            let album_art = self.imp().album_art.get();
            let media_control_bar_progress = self.imp().media_control_bar_progress.get();
            let progress_bar = self.imp().progress_bar.get();

            let progression = (track.elapsed as f64 / track.length as f64) * 100.0;
            progress_bar.set_value(progression);
            media_control_bar_progress.set_visible(true);

            title.set_text(&track.title);
            artist_album.set_text(&format!("{} - {}", track.artist, track.album));
            elapsed.set_text(&format_milliseconds(track.elapsed));
            duration.set_text(&format_milliseconds(track.length));

            if let Some(filename) = track.album_art {
                let home = std::env::var("HOME").unwrap();
                let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                album_art.set_from_file(Some(&path));
            }

            self.imp().set_current_track(Track {
                title: track.title,
                artist: track.artist,
                album: track.album,
                album_artist: track.album_artist,
                duration: track.length,
                elapsed: track.elapsed,
                ..Default::default()
            });
        }
    }
}
