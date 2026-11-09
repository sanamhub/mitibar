//! Tray icon and menu (WR-01), implementation A: Slint `SystemTrayIcon` (ADR-0010).
//!
//! This is implementation A on every OS. Slint's `SystemTrayIcon` covers Windows (notification
//! area), macOS (menu bar) and Linux (`StatusNotifierItem`). If it cannot meet
//! WR-01 on some OS, implementation B (`tray-icon` + `muda`, ADR-0010) replaces this module; the
//! `Tray` trait keeps the rest of the app unchanged.

use nepdate::Lang;

use crate::TrayComponent;
use crate::platform::bar::MenuLabels;

/// Events the tray delivers to the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Open,
    CopyBs,
    CopyAd,
    /// The Show bar check item (Windows only, WR-01 AC5): toggles the bar.
    ShowBar,
    Settings,
    Quit,
}

/// The tray surface the app updates: once per tick, and when a setting changes (WR-06 AC5).
pub trait Tray {
    fn set_title(&self, text: &str);
    fn set_icon_day(&self, day: &str);
    fn set_tooltip(&self, tooltip: &str);
    /// Picks the icon ink for a dark (`true`) or light taskbar.
    fn set_dark_taskbar(&self, dark: bool);
    /// Picks the digits of the icon: Devanagari for Nepali, ASCII otherwise.
    fn set_language(&self, lang: Lang);
    /// Shows the Show bar check item, on a platform that has a bar (WR-01 AC5).
    fn set_bar_supported(&self, supported: bool);
    /// Checks or unchecks the Show bar item to match the `bar` setting.
    fn set_bar_shown(&self, shown: bool);
    /// The translated labels, for the bar's right-click menu (WR-08 AC6).
    fn menu_labels(&self) -> MenuLabels;
    /// Adds the icon again after the notification area was recreated (Explorer restarted). The
    /// state it showed carries over; the caller refreshes the day and tooltip after.
    fn re_add(&self);
}

/// The six menu and click callbacks share one handler, so it lives behind `Rc<RefCell>`.
type Events = std::rc::Rc<std::cell::RefCell<dyn FnMut(TrayEvent)>>;

/// Implementation A: a Slint `SystemTrayIcon`. The two day-number image arrays live in
/// `ui/tray.slint` (pre-rendered by `cargo xtask icons`); `set_icon_day` only sets the index,
/// and the `is-ne` flag picks the digit set.
///
/// The component sits in a `RefCell` so `re_add` can replace it after Explorer restarts. Slint's
/// Windows tray receives its callbacks on a message-only window, and Windows does not send
/// broadcasts such as `TaskbarCreated` to message-only windows, so the icon would stay gone.
pub struct SlintTray {
    component: std::cell::RefCell<TrayComponent>,
    events: Events,
}

impl SlintTray {
    /// Creates the tray and wires the menu and click events to `on_event`. The language is fixed
    /// at construction; the settings UI updates `is_ne` when it changes.
    pub fn new(lang: Lang, on_event: impl FnMut(TrayEvent) + 'static) -> anyhow::Result<Self> {
        let events: Events = std::rc::Rc::new(std::cell::RefCell::new(on_event));
        let component = Self::component(&events)?;
        component.set_is_ne(lang == Lang::Ne);
        Ok(SlintTray {
            component: std::cell::RefCell::new(component),
            events,
        })
    }

    /// A new tray component with the callbacks wired to `events`. Each borrow of the handler is
    /// dropped before the next event.
    fn component(events: &Events) -> anyhow::Result<TrayComponent> {
        let component = TrayComponent::new()?;
        let wire = |event: TrayEvent| {
            let events = events.clone();
            move || (events.borrow_mut())(event)
        };
        component.on_open(wire(TrayEvent::Open));
        component.on_copy_bs(wire(TrayEvent::CopyBs));
        component.on_copy_ad(wire(TrayEvent::CopyAd));
        component.on_show_bar(wire(TrayEvent::ShowBar));
        component.on_settings(wire(TrayEvent::Settings));
        component.on_quit(wire(TrayEvent::Quit));
        Ok(component)
    }
}

impl Tray for SlintTray {
    fn set_title(&self, text: &str) {
        self.component.borrow().set_title_text(text.into());
    }

    fn set_icon_day(&self, day: &str) {
        let Ok(day) = day.parse::<i32>() else {
            return;
        };
        if (1..=32).contains(&day) {
            self.component.borrow().set_day_index(day - 1);
        }
    }

    fn set_tooltip(&self, tooltip: &str) {
        self.component.borrow().set_tooltip_text(tooltip.into());
    }

    fn set_dark_taskbar(&self, dark: bool) {
        self.component.borrow().set_dark_taskbar(dark);
    }

    fn set_language(&self, lang: Lang) {
        self.component.borrow().set_is_ne(lang == Lang::Ne);
    }

    fn set_bar_supported(&self, supported: bool) {
        self.component.borrow().set_bar_supported(supported);
    }

    fn set_bar_shown(&self, shown: bool) {
        self.component.borrow().set_bar_shown(shown);
    }

    fn re_add(&self) {
        let fresh = match Self::component(&self.events) {
            Ok(fresh) => fresh,
            Err(err) => {
                log::warn!("tray: could not add the icon again: {err}");
                return;
            }
        };
        let old = self.component.replace(fresh);
        let new = self.component.borrow();
        new.set_is_ne(old.get_is_ne());
        new.set_dark_taskbar(old.get_dark_taskbar());
        new.set_day_index(old.get_day_index());
        new.set_title_text(old.get_title_text());
        new.set_tooltip_text(old.get_tooltip_text());
        new.set_bar_supported(old.get_bar_supported());
        new.set_bar_shown(old.get_bar_shown());
        // Dropping the old component sends NIM_DELETE for an icon the new Explorer never had,
        // which fails harmlessly.
        drop(old);
        log::info!("tray: icon added again after Explorer restarted");
    }

    /// The popup may not exist yet; the tray component always does.
    fn menu_labels(&self) -> MenuLabels {
        MenuLabels {
            open: self.component.borrow().get_menu_open().to_string(),
            copy_bs: self.component.borrow().get_menu_copy_bs().to_string(),
            copy_ad: self.component.borrow().get_menu_copy_ad().to_string(),
            customize: self.component.borrow().get_menu_customize_bar().to_string(),
            lock: self.component.borrow().get_menu_lock_position().to_string(),
            hide: self.component.borrow().get_menu_hide_bar().to_string(),
            settings: self.component.borrow().get_menu_settings().to_string(),
            quit: self.component.borrow().get_menu_quit().to_string(),
        }
    }
}
