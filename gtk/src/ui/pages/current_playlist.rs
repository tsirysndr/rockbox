use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::{
    CurrentTrackResponse, PlaylistResponse, StreamPlaylistRequest,
};
use crate::state::AppState;
use crate::types::track::Track;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::{EllipsizeMode, WrapMode};
use gtk::{CompositeTemplate, Image, Label, ListBox, ScrolledWindow};
use std::cell::{Cell, RefCell};
use std::env;
use std::thread;
use tokio::sync::mpsc;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/current_playlist.ui")]
    pub struct CurrentPlaylist {
        #[template_child]
        pub album_cover: TemplateChild<Image>,
        #[template_child]
        pub track_title: TemplateChild<Label>,
        #[template_child]
        pub track_artist: TemplateChild<Label>,
        #[template_child]
        pub track_index: TemplateChild<Label>,
        #[template_child]
        pub now_playing: TemplateChild<gtk::Box>,
        #[template_child]
        pub next_tracks: TemplateChild<ListBox>,
        #[template_child]
        pub scrolled_window: TemplateChild<ScrolledWindow>,

        pub state: glib::WeakRef<AppState>,
        pub ready: Cell<bool>,
        pub tracks: RefCell<Vec<CurrentTrackResponse>>,
        pub current_index: Cell<usize>,
        pub size: Cell<usize>,
        pub play_all_button: RefCell<Option<gtk::Button>>,
        pub shuffle_all_button: RefCell<Option<gtk::Button>>,
        pub new_playlist_menu_button: RefCell<Option<gtk::MenuButton>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CurrentPlaylist {
        const NAME: &'static str = "CurrentPlaylist";
        type ParentType = gtk::Box;
        type Type = super::CurrentPlaylist;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CurrentPlaylist {
        fn constructed(&self) {
            self.parent_constructed();
            self.size.set(10);

            let self_weak = self.downgrade();
            self.scrolled_window.connect_edge_reached(move |_, pos| {
                if pos == gtk::PositionType::Bottom {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };

                    let index = self_.current_index.get() as usize + 1;
                    let size = self_.size.get();
                    let tracks = self_.tracks.borrow().clone();

                    if index + size >= tracks.len() {
                        return;
                    }

                    let next_range_end = (index + size + 3).min(tracks.len());
                    let next_tracks = tracks[index + size..next_range_end].to_vec();

                    if next_tracks.is_empty() {
                        return;
                    }

                    self_.size.set(size + next_tracks.len());
                    let mut i = index + size;
                    for track in next_tracks {
                        let song = create_song_widget(
                            Track {
                                title: track.title.clone(),
                                artist: track.artist.clone(),
                                album_art: track.album_art.clone(),
                                ..Default::default()
                            },
                            i as i32,
                        );
                        song.imp().remove_button.set_visible(true);

                        self_.next_tracks.append(&song);
                        i += 1;
                    }
                }
            });

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let (tx, mut rx) = mpsc::channel(32);

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::spawn_future_local(async move {
                    let obj = self_.obj();
                    obj.stream_playlist(tx);
                });

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::spawn_future_local(async move {
                    let obj = self_.obj();
                    while let Some(playlist) = rx.recv().await {
                        obj.load_current_track();
                        let state = obj.imp().state.upgrade().unwrap();

                        let current_index = match state.resume_index() > -1 {
                            true => state.resume_index() as usize,
                            false => playlist.index as usize,
                        };
                        obj.imp().current_index.set(current_index);
                        obj.imp().tracks.replace(playlist.tracks.clone());

                        if let Some(track) = state.current_track() {
                            if track.elapsed > 1000 && obj.imp().ready.get() {
                                continue;
                            }
                        }

                        obj.imp().ready.set(true);

                        let index = current_index + 1;
                        let mut tracks = playlist.tracks.clone();
                        let next_tracks =
                            tracks.drain(index..).collect::<Vec<CurrentTrackResponse>>();

                        while let Some(row) = obj.imp().now_playing.first_child() {
                            obj.imp().now_playing.remove(&row);
                        }

                        let label = Label::new(Some("Now playing"));
                        label.set_halign(gtk::Align::Start);
                        label.set_margin_start(10);
                        label.add_css_class("bold");

                        obj.imp().now_playing.append(&label);

                        if let Some(track) = state.current_track() {
                            let song = create_song_widget(track, current_index as i32);
                            song.imp().remove_button.set_visible(false);
                            obj.imp().now_playing.append(&song);
                        }

                        while let Some(row) = obj.imp().next_tracks.first_child() {
                            obj.imp().next_tracks.remove(&row);
                        }

                        let limit = match next_tracks.len() > 10 {
                            true => 10,
                            false => next_tracks.len(),
                        };
                        let mut i = index;
                        for track in next_tracks.into_iter().take(limit) {
                            let song = create_song_widget(
                                Track {
                                    title: track.title.clone(),
                                    artist: track.artist.clone(),
                                    album_art: track.album_art.clone(),
                                    ..Default::default()
                                },
                                i as i32,
                            );

                            song.imp().remove_button.set_visible(true);
                            song.imp().album_art_container.set_visible(true);
                            obj.imp().next_tracks.append(&song);
                            i += 1;
                        }
                    }
                });
                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for CurrentPlaylist {}
    impl BoxImpl for CurrentPlaylist {}
}

glib::wrapper! {
  pub struct CurrentPlaylist(ObjectSubclass<imp::CurrentPlaylist>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl CurrentPlaylist {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn hide_top_buttons(&self, hide: bool) {
        let play_all_button = self.imp().play_all_button.borrow();
        let play_all_button_ref = play_all_button.as_ref();
        let shuffle_all_button = self.imp().shuffle_all_button.borrow();
        let shuffle_all_button_ref = shuffle_all_button.as_ref();
        play_all_button_ref.unwrap().set_visible(!hide);
        shuffle_all_button_ref.unwrap().set_visible(!hide);
    }

    pub fn hide_playlist_buttons(&self, hide: bool) {
        let new_playlist_menu_button = self.imp().new_playlist_menu_button.borrow();
        let new_playlist_menu_button_ref = new_playlist_menu_button.as_ref();
        new_playlist_menu_button_ref.unwrap().set_visible(!hide);
    }

    pub fn load_current_track(&self) {
        let state = self.imp().state.upgrade().unwrap();
        match state.current_track() {
            Some(track) => {
                let current_index = match state.resume_index() > -1 {
                    true => state.resume_index() as usize,
                    false => self.imp().current_index.get(),
                };
                self.imp().track_title.set_text(&track.title);
                self.imp().track_title.set_wrap_mode(WrapMode::WordChar);
                self.imp().track_title.set_max_width_chars(20);
                self.imp().track_title.set_wrap(true);
                self.imp().track_artist.set_text(&track.artist);
                self.imp().track_artist.set_wrap_mode(WrapMode::WordChar);
                self.imp().track_artist.set_max_width_chars(20);
                self.imp().track_artist.set_wrap(true);
                self.imp().track_index.set_text(&format!(
                    "{} of {}",
                    current_index + 1,
                    self.imp().tracks.borrow().len()
                ));

                match track.album_art {
                    Some(filename) => {
                        let home = env::var("HOME").unwrap();
                        let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                        self.imp().album_cover.set_from_file(Some(&path));
                    }
                    None => {
                        self.imp().album_cover.set_resource(Some(
                            "/io/github/tsirysndr/Rockbox/icons/jpg/albumart.jpg",
                        ));
                    }
                }
            }
            None => {
                self.imp()
                    .album_cover
                    .set_resource(Some("/io/github/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
                self.imp().track_title.set_text("No song playing");
                self.imp().track_artist.set_text("");
            }
        }
    }

    pub fn load_current_playlist(&self) {
        let index = self.imp().current_index.get() + 1;
        let mut tracks = self.imp().tracks.borrow().clone();
        let next_tracks = tracks.drain(index..).collect::<Vec<CurrentTrackResponse>>();

        while let Some(row) = self.imp().now_playing.first_child() {
            self.imp().now_playing.remove(&row);
        }

        let label = Label::new(Some("Now playing"));
        label.set_halign(gtk::Align::Start);
        label.set_margin_start(10);
        label.add_css_class("bold");

        self.imp().now_playing.append(&label);
        let state = self.imp().state.upgrade().unwrap();

        if let Some(track) = state.current_track() {
            let song = create_song_widget(track, index as i32 - 1);
            self.imp().now_playing.append(&song);
        }

        while let Some(row) = self.imp().next_tracks.first_child() {
            self.imp().next_tracks.remove(&row);
        }

        let limit = match next_tracks.len() > 10 {
            true => 10,
            false => next_tracks.len(),
        };

        let mut i = index;
        for track in next_tracks.into_iter().take(limit) {
            let song = create_song_widget(
                Track {
                    title: track.title.clone(),
                    artist: track.artist.clone(),
                    album_art: track.album_art.clone(),
                    ..Default::default()
                },
                i as i32,
            );
            song.imp().remove_button.set_visible(true);
            self.imp().next_tracks.append(&song);
            i += 1;
        }
    }

    pub fn stream_playlist(&self, tx: mpsc::Sender<PlaylistResponse>) {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                let mut stream = client
                    .stream_playlist(StreamPlaylistRequest {})
                    .await?
                    .into_inner();

                while let Some(playlist) = stream.message().await? {
                    tx.send(playlist).await?;
                }

                Ok::<(), Error>(())
            });
        });
    }
}

fn create_song_widget(track: Track, index: i32) -> Song {
    let song = Song::new();
    song.imp().track_number.set_visible(false);
    song.imp().track_title.set_text(&track.title);
    song.imp().track_title.set_ellipsize(EllipsizeMode::End);
    song.imp().track_title.set_max_width_chars(80);
    song.imp().artist.set_text(&track.artist);
    song.imp().artist.set_ellipsize(EllipsizeMode::End);
    song.imp().artist.set_max_width_chars(80);
    song.imp().track_duration.set_visible(false);
    song.imp().heart_button.set_visible(false);
    song.imp().more_button.set_visible(false);
    song.imp().index.set(index);
    song.imp().is_playlist.set(true);
    song.imp().track.replace(Some(track.clone().into()));

    match track.album_art {
        Some(filename) => {
            let home = env::var("HOME").unwrap();
            let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
            song.imp().album_art.set_from_file(Some(&path));
        }
        None => {
            song.imp()
                .album_art
                .set_resource(Some("/io/github/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
        }
    }

    song.imp().album_art_container.set_visible(true);

    song
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
