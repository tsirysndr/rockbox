use crate::state::AppState;
use crate::ui::pages::playlist_details::PlaylistDetails;
use crate::ui::{delete_playlist::DeletePlaylistDialog, edit_playlist::EditPlaylistDialog};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image, Label, MenuButton};
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlist.ui")]
    pub struct Playlist {
        #[template_child]
        pub playlist_icon: TemplateChild<Image>,
        #[template_child]
        pub playlist_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,
        #[template_child]
        pub playlist_menu: TemplateChild<MenuButton>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub state: glib::WeakRef<AppState>,
        pub playlist_details: RefCell<Option<PlaylistDetails>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Playlist {
        const NAME: &'static str = "Playlist";
        type ParentType = gtk::Box;
        type Type = super::Playlist;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.playlist.play-now",
                None,
                move |playlist, _action, _target| {
                    playlist.play_now(false);
                },
            );

            klass.install_action(
                "app.playlist.play-next",
                None,
                move |playlist, _action, _target| {
                    playlist.play_next();
                },
            );

            klass.install_action(
                "app.playlist.play-last",
                None,
                move |playlist, _action, _target| {
                    playlist.play_next();
                },
            );

            klass.install_action(
                "app.playlist.add-shuffled",
                None,
                move |playlist, _action, _target| {
                    playlist.add_shuffled();
                },
            );

            klass.install_action(
                "app.playlist.play-last-shuffled",
                None,
                move |playlist, _action, _target| {
                    playlist.play_last_shuffled();
                },
            );

            klass.install_action(
                "app.playlist.play-shuffled",
                None,
                move |playlist, _action, _target| {
                    playlist.play_now(true);
                },
            );

            klass.install_action(
                "app.playlist.edit",
                None,
                move |playlist, _action, _target| {
                    let edit_playlist_dialog = EditPlaylistDialog::default();
                    edit_playlist_dialog.present(Some(playlist));
                },
            );

            klass.install_action(
                "app.playlist.delete",
                None,
                move |playlist, _action, _target| {
                    let delete_playlist_dialog = DeletePlaylistDialog::default();
                    delete_playlist_dialog.present(Some(playlist));
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Playlist {
        fn constructed(&self) {
            self.parent_constructed();
            let self_weak = self.downgrade();
            let click = gtk::GestureClick::new();
            click.connect_released(move |_, _, _, _| {
                if let Some(self_) = self_weak.upgrade() {
                    let main_stack = self_.main_stack.borrow();
                    let main_stack_ref = main_stack.as_ref().unwrap();
                    main_stack_ref.set_visible_child_name("playlist-details-page");
                    let state = self_.state.upgrade().unwrap();
                    state.push_navigation("Playlist", "playlist-details-page");

                    let go_back_button = self_.go_back_button.borrow();
                    if let Some(go_back_button) = go_back_button.as_ref() {
                        go_back_button.set_visible(true);
                    }
                }
            });

            self.row.add_controller(click);
        }
    }

    impl WidgetImpl for Playlist {}
    impl BoxImpl for Playlist {}
}

glib::wrapper! {
  pub struct Playlist(ObjectSubclass<imp::Playlist>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Playlist {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn play_now(&self, _shuffle: bool) {}

    pub fn play_next(&self) {}

    pub fn play_last(&self) {}

    pub fn add_shuffled(&self) {}

    pub fn play_last_shuffled(&self) {}
}
