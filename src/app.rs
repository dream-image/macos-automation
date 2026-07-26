use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSControlStateValueOff,
    NSControlStateValueOn, NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength, NSWorkspace, NSWorkspaceDidWakeNotification,
    NSWorkspaceWillSleepNotification,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSSize, ns_string,
};

use crate::automation::{self, Setting};

#[derive(Debug)]
struct AppDelegateIvars {
    status_item: OnceCell<Retained<NSStatusItem>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    #[derive(Debug)]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    impl AppDelegate {
        #[unsafe(method(toggleBluetooth:))]
        fn toggle_bluetooth(&self, sender: &NSMenuItem) {
            let enabled = sender.state() != NSControlStateValueOn;
            if enabled {
                automation::request_permission(Setting::BluetoothOnSleep);
            }
            sender.setState(menu_state(enabled));
            automation::set_enabled(Setting::BluetoothOnSleep, enabled);
        }

        #[unsafe(method(toggleWifi:))]
        fn toggle_wifi(&self, sender: &NSMenuItem) {
            let enabled = sender.state() != NSControlStateValueOn;
            sender.setState(menu_state(enabled));
            automation::set_enabled(Setting::WifiOnSleep, enabled);
        }

        #[unsafe(method(handleWillSleep:))]
        fn handle_will_sleep(&self, _notification: &NSNotification) {
            automation::handle_sleep();
        }

        #[unsafe(method(handleDidWake:))]
        fn handle_did_wake(&self, _notification: &NSNotification) {
            automation::handle_wake();
        }

        #[unsafe(method(quitApplication:))]
        fn quit_application(&self, _sender: &NSMenuItem) {
            NSApplication::sharedApplication(self.mtm()).terminate(None);
        }
    }

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _notification: &NSNotification) {
            self.create_status_item();
            self.observe_power_events();
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars {
            status_item: OnceCell::new(),
        });

        unsafe { msg_send![super(this), init] }
    }

    fn create_status_item(&self) {
        let mtm = self.mtm();
        let status_item =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);

        if let Some(button) = status_item.button(mtm)
            && let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
                ns_string!("gearshape.fill"),
                Some(ns_string!("自动化")),
            )
        {
            image.setSize(NSSize::new(18.0, 18.0));
            image.setTemplate(true);
            button.setImage(Some(&image));
        }

        let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(""));
        let bluetooth_item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                ns_string!("睡眠自动关闭蓝牙"),
                Some(sel!(toggleBluetooth:)),
                ns_string!(""),
            )
        };
        let bluetooth_enabled = automation::is_enabled(Setting::BluetoothOnSleep);
        if bluetooth_enabled {
            automation::request_permission(Setting::BluetoothOnSleep);
        }
        bluetooth_item.setState(menu_state(bluetooth_enabled));
        unsafe {
            bluetooth_item.setTarget(Some(self.as_ref()));
        }

        let wifi_item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                ns_string!("睡眠自动关闭 Wi-Fi"),
                Some(sel!(toggleWifi:)),
                ns_string!(""),
            )
        };
        wifi_item.setState(menu_state(automation::is_enabled(Setting::WifiOnSleep)));
        unsafe {
            wifi_item.setTarget(Some(self.as_ref()));
        }

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let quit_item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                ns_string!("退出"),
                Some(sel!(quitApplication:)),
                ns_string!("q"),
            )
        };
        unsafe {
            quit_item.setTarget(Some(self.as_ref()));
        }

        status_item.setMenu(Some(&menu));
        let _ = self.ivars().status_item.set(status_item);
    }

    fn observe_power_events(&self) {
        let workspace = NSWorkspace::sharedWorkspace();
        let notification_center = workspace.notificationCenter();

        unsafe {
            notification_center.addObserver_selector_name_object(
                self.as_ref(),
                sel!(handleWillSleep:),
                Some(NSWorkspaceWillSleepNotification),
                None,
            );
            notification_center.addObserver_selector_name_object(
                self.as_ref(),
                sel!(handleDidWake:),
                Some(NSWorkspaceDidWakeNotification),
                None,
            );
        }
    }
}

fn menu_state(enabled: bool) -> isize {
    if enabled {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
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
