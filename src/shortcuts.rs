use kurage_macro_rules::relm4::gtk::{self, gio::prelude::*, glib};

#[derive(Default)]
pub struct ShortcutManager {
    pub actions: gtk::gio::SimpleActionGroup,
    pub shortcut_ctl: gtk::ShortcutController,
}

impl ShortcutManager {
    pub fn make<F: Fn(&gtk::gio::SimpleAction, Option<&glib::Variant>) + 'static>(
        &self,
        shortcut: &'static str,
        name: &'static str,
        f: F,
    ) {
        let action = gtk::gio::SimpleAction::new(name, None);
        action.connect_activate(f);
        self.actions.add_action(&action);
        let kb_shortcut = gtk::Shortcut::builder()
            .trigger(&gtk::ShortcutTrigger::parse_string(shortcut).unwrap())
            .action(&gtk::ShortcutAction::parse_string(&format!("action(app.{name})")).unwrap())
            .build();
        self.shortcut_ctl.add_shortcut(kb_shortcut);
    }
}

#[macro_export]
macro_rules! gen_shortcut {
    ($AppMsg:ident $shortcutman:ident $sender:ident) => {
        macro_rules! shortcut {
            ($shortcut:literal => $name:ident) => { ::paste::paste! {
                let new_sender = $sender.clone();
                $shortcutman.make($shortcut, stringify!([<$name:lower>]), move |_, _| new_sender.input($AppMsg::[<$name:camel>]));
            }};
            ($shortcut:literal => $name:ident => $alias:ident) => { ::paste::paste! {
                let new_sender = $sender.clone();
                $shortcutman.make($shortcut, stringify!([<$name:lower>]), move |_, _| new_sender.input($AppMsg::$alias));
            }};
            ($shortcut:literal => $name:ident => $body:block) => { ::paste::paste! {{
                #[allow(unused_variables)]
                let [<sender>] = $sender.clone();
                $shortcutman.make($shortcut, stringify!([<$name:lower>]), move |_, _| $body);
            }}};
        }
    }
}
