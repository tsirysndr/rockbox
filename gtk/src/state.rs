use crate::navigation::NavigationHistory;
use glib::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AppState {
        pub navigation_history: RefCell<NavigationHistory>,
        pub current_path: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppState {
        const NAME: &'static str = "AppState";
        type Type = super::AppState;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AppState {}
}

glib::wrapper! {
    pub struct AppState(ObjectSubclass<imp::AppState>);
}

impl AppState {
    pub fn new() -> Self {
        let obj = glib::Object::new::<Self>();

        obj.imp()
            .navigation_history
            .replace(NavigationHistory::new());
        obj.imp().current_path.replace(None);

        obj
    }

    pub fn new_navigation_from(&self, page_name: &str, page_id: &str) {
        let self_ = imp::AppState::from_obj(self);
        *self_.navigation_history.borrow_mut() =
            NavigationHistory::new_from(page_name.to_string(), page_id.to_string());
    }

    pub fn push_navigation(&self, page_name: &str, page_id: &str) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow_mut();
        navigation.push(page_name.to_string(), page_id.to_string());
    }

    pub fn pop_navigation(&self) -> (String, String) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow_mut();
        navigation.pop()
    }

    pub fn current_page(&self) -> (String, String) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow();
        navigation.current()
    }

    pub fn reset_navigation(&self) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow_mut();
        navigation.reset();
    }

    pub fn navigation_stack_len(&self) -> usize {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow();
        navigation.len()
    }

    pub fn get_navigation_history(&self) -> NavigationHistory {
        let self_ = imp::AppState::from_obj(self);
        self_.navigation_history.borrow().clone()
    }

    pub fn current_path(&self) -> Option<String> {
        let self_ = imp::AppState::from_obj(self);
        self_.current_path.borrow().clone()
    }

    pub fn set_current_path(&self, path: &str) {
        let self_ = imp::AppState::from_obj(self);
        *self_.current_path.borrow_mut() = Some(path.to_string());
    }
}
