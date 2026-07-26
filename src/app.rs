use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType,
    NSButton, NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength, NSWindow, NSWindowStyleMask, NSWorkspace,
    NSWorkspaceWillPowerOffNotification,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize,
    ns_string,
};

use crate::automation::{self, Setting};

#[derive(Debug)]
struct AppDelegateIvars {
    status_item: OnceCell<Retained<NSStatusItem>>,
    settings_window: OnceCell<Retained<NSWindow>>,
    bluetooth_button: OnceCell<Retained<NSButton>>,
    wifi_button: OnceCell<Retained<NSButton>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    #[derive(Debug)]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    impl AppDelegate {
        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: &AnyObject) {
            self.sync_button_states();

            if let Some(window) = self.ivars().settings_window.get() {
                window.makeKeyAndOrderFront(None);
                NSApplication::sharedApplication(self.mtm()).activate();
            }
        }

        #[unsafe(method(toggleBluetooth:))]
        fn toggle_bluetooth(&self, sender: &NSButton) {
            automation::set_enabled(
                Setting::BluetoothOnShutdown,
                sender.state() == NSControlStateValueOn,
            );
        }

        #[unsafe(method(toggleWifi:))]
        fn toggle_wifi(&self, sender: &NSButton) {
            automation::set_enabled(
                Setting::WifiOnShutdown,
                sender.state() == NSControlStateValueOn,
            );
        }

        #[unsafe(method(handleWillPowerOff:))]
        fn handle_will_power_off(&self, _notification: &NSNotification) {
            automation::handle_shutdown();
        }
    }

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _notification: &NSNotification) {
            self.create_status_item();
            self.create_settings_window();
            self.observe_shutdown();
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars {
            status_item: OnceCell::new(),
            settings_window: OnceCell::new(),
            bluetooth_button: OnceCell::new(),
            wifi_button: OnceCell::new(),
        });

        unsafe { msg_send![super(this), init] }
    }

    fn create_status_item(&self) {
        let mtm = self.mtm();
        let status_item =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);

        if let Some(button) = status_item.button(mtm) {
            button.setTitle(ns_string!("⚙︎"));
        }

        let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(""));
        let settings_item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                ns_string!("设置…"),
                Some(sel!(openSettings:)),
                ns_string!(""),
            )
        };
        unsafe {
            settings_item.setTarget(Some(self.as_ref()));
        }

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let quit_item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                ns_string!("退出"),
                Some(sel!(terminate:)),
                ns_string!("q"),
            )
        };
        unsafe {
            quit_item.setTarget(Some(NSApplication::sharedApplication(mtm).as_ref()));
        }

        status_item.setMenu(Some(&menu));
        let _ = self.ivars().status_item.set(status_item);
    }

    fn create_settings_window(&self) {
        let mtm = self.mtm();
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(420.0, 180.0));
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                frame,
                NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
                NSBackingStoreType::Buffered,
                false,
            )
        };

        unsafe {
            window.setReleasedWhenClosed(false);
        }
        window.setTitle(ns_string!("设置"));
        window.center();

        let bluetooth_button = unsafe {
            NSButton::checkboxWithTitle_target_action(
                ns_string!("关机自动关闭蓝牙"),
                Some(self.as_ref()),
                Some(sel!(toggleBluetooth:)),
                mtm,
            )
        };
        bluetooth_button.setFrame(NSRect::new(
            NSPoint::new(24.0, 104.0),
            NSSize::new(360.0, 28.0),
        ));

        let wifi_button = unsafe {
            NSButton::checkboxWithTitle_target_action(
                ns_string!("关机自动关闭 Wi-Fi"),
                Some(self.as_ref()),
                Some(sel!(toggleWifi:)),
                mtm,
            )
        };
        wifi_button.setFrame(NSRect::new(
            NSPoint::new(24.0, 64.0),
            NSSize::new(360.0, 28.0),
        ));

        if let Some(content_view) = window.contentView() {
            content_view.addSubview(&bluetooth_button);
            content_view.addSubview(&wifi_button);
        }

        let _ = self.ivars().bluetooth_button.set(bluetooth_button);
        let _ = self.ivars().wifi_button.set(wifi_button);
        let _ = self.ivars().settings_window.set(window);
        self.sync_button_states();
    }

    fn sync_button_states(&self) {
        if let Some(button) = self.ivars().bluetooth_button.get() {
            button.setState(if automation::is_enabled(Setting::BluetoothOnShutdown) {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }

        if let Some(button) = self.ivars().wifi_button.get() {
            button.setState(if automation::is_enabled(Setting::WifiOnShutdown) {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    }

    fn observe_shutdown(&self) {
        let workspace = NSWorkspace::sharedWorkspace();
        let notification_center = workspace.notificationCenter();

        unsafe {
            notification_center.addObserver_selector_name_object(
                self.as_ref(),
                sel!(handleWillPowerOff:),
                Some(NSWorkspaceWillPowerOffNotification),
                None,
            );
        }
    }
}

pub fn run() {
    let mtm = MainThreadMarker::new().expect("应用必须从主线程启动");
    let app = NSApplication::sharedApplication(mtm);
    let delegate = AppDelegate::new(mtm);

    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    app.run();
}
