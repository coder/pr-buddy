use std::collections::HashMap;

pub(crate) const PR_URL_USER_INFO_KEY: &str = "pr_buddy_pr_url";

pub(crate) fn build_click_payload(url: &str) -> HashMap<String, String> {
    let mut payload = HashMap::with_capacity(1);
    payload.insert(PR_URL_USER_INFO_KEY.to_string(), url.to_string());
    payload
}

pub(crate) fn extract_clicked_url(payload: &HashMap<String, String>) -> Option<String> {
    let url = payload.get(PR_URL_USER_INFO_KEY)?;
    sanitize_clicked_url(url)
}

fn sanitize_clicked_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }

    if !trimmed.starts_with("https://") && !trimmed.starts_with("http://") {
        return None;
    }

    Some(trimmed.to_string())
}

#[cfg(target_os = "macos")]
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Mutex, OnceLock,
    },
};

#[cfg(target_os = "macos")]
use block2::DynBlock;
#[cfg(target_os = "macos")]
use objc2::{
    define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::{NSObject, NSObjectProtocol, ProtocolObject},
    ClassType, MainThreadMarker,
};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSDictionary, NSString};
#[cfg(target_os = "macos")]
use objc2_user_notifications::{
    UNMutableNotificationContent, UNNotification, UNNotificationDefaultActionIdentifier,
    UNNotificationDismissActionIdentifier, UNNotificationPresentationOptions,
    UNNotificationRequest, UNNotificationResponse, UNUserNotificationCenter,
    UNUserNotificationCenterDelegate,
};
#[cfg(target_os = "macos")]
use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri_plugin_opener::OpenerExt;

#[cfg(target_os = "macos")]
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
#[cfg(target_os = "macos")]
static DELEGATE_PTR: OnceLock<usize> = OnceLock::new();
#[cfg(target_os = "macos")]
static HANDLED_NOTIFICATION_IDS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
#[cfg(target_os = "macos")]
static NOTIFICATION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "PrBuddyNotificationClickDelegate"]
    #[thread_kind = MainThreadOnly]
    struct NotificationClickDelegate;

    impl NotificationClickDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for NotificationClickDelegate {}

    unsafe impl UNUserNotificationCenterDelegate for NotificationClickDelegate {
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn userNotificationCenter_willPresentNotification_withCompletionHandler(
            &self,
            _center: &UNUserNotificationCenter,
            _notification: &UNNotification,
            completion_handler: &DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            completion_handler.call((
                UNNotificationPresentationOptions::Banner
                    | UNNotificationPresentationOptions::List
                    | UNNotificationPresentationOptions::Sound,
            ));
        }

        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        fn userNotificationCenter_didReceiveNotificationResponse_withCompletionHandler(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            completion_handler: &DynBlock<dyn Fn()>,
        ) {
            handle_notification_response(response);
            completion_handler.call(());
        }
    }
);

#[cfg(target_os = "macos")]
pub(crate) fn register_delegate(app: &AppHandle) -> Result<(), String> {
    if let Some(existing_app) = APP_HANDLE.get() {
        debug_assert_eq!(
            existing_app.config().identifier,
            app.config().identifier,
            "macOS notification delegate should only be registered for a single app identifier"
        );
    } else {
        let _ = APP_HANDLE.set(app.clone());
    }

    if MainThreadMarker::new().is_some() {
        return register_delegate_on_main_thread();
    }

    let (sender, receiver) = mpsc::channel();
    app.run_on_main_thread(move || {
        let result = register_delegate_on_main_thread();
        let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;

    receiver
        .recv()
        .map_err(|_| "failed to receive macOS notification delegate registration result".to_string())?
}

#[cfg(target_os = "macos")]
pub(crate) fn send_notification(app: &AppHandle, title: &str, body: &str, url: &str) {
    let Some(url) = sanitize_clicked_url(url) else {
        eprintln!(
            "[macos_notifications] Skipping notification with invalid PR URL payload: {:?}",
            url
        );
        return;
    };

    let title = title.to_string();
    let body = body.to_string();
    let app = app.clone();

    if let Err(error) = app.run_on_main_thread(move || {
        send_notification_on_main_thread(&title, &body, &url);
    }) {
        eprintln!(
            "[macos_notifications] Failed to schedule notification on the main thread: {}",
            error
        );
    }
}

#[cfg(target_os = "macos")]
fn register_delegate_on_main_thread() -> Result<(), String> {
    debug_assert!(
        MainThreadMarker::new().is_some(),
        "macOS notification delegate registration must run on the main thread"
    );

    if DELEGATE_PTR.get().is_some() {
        return Ok(());
    }

    let center = UNUserNotificationCenter::currentNotificationCenter();
    let delegate: Retained<NotificationClickDelegate> = unsafe { msg_send![NotificationClickDelegate::class(), new] };
    center.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    let raw_delegate = Retained::into_raw(delegate) as usize;
    match DELEGATE_PTR.set(raw_delegate) {
        Ok(()) => Ok(()),
        Err(_) => {
            let _ = unsafe {
                Retained::from_raw(raw_delegate as *mut NotificationClickDelegate)
            };
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
fn send_notification_on_main_thread(title: &str, body: &str, url: &str) {
    debug_assert!(
        MainThreadMarker::new().is_some(),
        "macOS notification delivery must run on the main thread"
    );
    debug_assert!(!title.trim().is_empty(), "notification titles should not be empty");
    debug_assert!(!body.trim().is_empty(), "notification bodies should not be empty");

    let payload = build_click_payload(url);
    let user_info = payload_to_user_info(&payload);
    let content = UNMutableNotificationContent::new();
    content.setTitle(&NSString::from_str(title));
    content.setBody(&NSString::from_str(body));
    unsafe {
        content.setUserInfo(user_info.as_ref());
    }

    let identifier = NSString::from_str(&next_notification_identifier());
    let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
        &identifier,
        content.as_ref(),
        None,
    );

    UNUserNotificationCenter::currentNotificationCenter()
        .addNotificationRequest_withCompletionHandler(&request, None);
}

#[cfg(target_os = "macos")]
fn payload_to_user_info(payload: &HashMap<String, String>) -> Retained<NSDictionary<NSString, NSString>> {
    let url = payload
        .get(PR_URL_USER_INFO_KEY)
        .expect("click payload should always contain the PR URL key");
    let key = NSString::from_str(PR_URL_USER_INFO_KEY);
    let value = NSString::from_str(url);
    NSDictionary::from_slices(&[&*key], &[&*value])
}

#[cfg(target_os = "macos")]
fn extract_clicked_url_from_user_info(user_info: &NSDictionary) -> Option<String> {
    let key = NSString::from_str(PR_URL_USER_INFO_KEY);
    let typed_user_info: &NSDictionary<NSString, NSString> = unsafe {
        // SAFETY: PR Buddy only writes string keys and values into the notification
        // payload, and this helper only reads back the single key that it writes.
        user_info.cast_unchecked()
    };
    let value = typed_user_info.objectForKey(&*key)?;
    let payload = build_click_payload(&value.to_string());
    extract_clicked_url(&payload)
}

#[cfg(target_os = "macos")]
fn handle_notification_response(response: &UNNotificationResponse) {
    let action_identifier = response.actionIdentifier();
    if &*action_identifier != UNNotificationDefaultActionIdentifier {
        if &*action_identifier != UNNotificationDismissActionIdentifier {
            eprintln!(
                "[macos_notifications] Ignoring unsupported macOS notification action: {}",
                action_identifier
            );
        }
        return;
    }

    let request = response.notification().request();
    let request_identifier = request.identifier().to_string();
    if !mark_notification_handled(&request_identifier) {
        eprintln!(
            "[macos_notifications] Ignoring duplicate click for notification {}",
            request_identifier
        );
        return;
    }

    let Some(url) = extract_clicked_url_from_user_info(&request.content().userInfo()) else {
        eprintln!(
            "[macos_notifications] Notification click payload did not contain a usable PR URL"
        );
        return;
    };

    let Some(app) = APP_HANDLE.get() else {
        eprintln!(
            "[macos_notifications] Notification delegate received a click before app state was registered"
        );
        return;
    };

    if let Err(error) = app.opener().open_url(&url, None::<&str>) {
        eprintln!(
            "[macos_notifications] Failed to open PR URL from notification click: {}",
            error
        );
    }
}

#[cfg(target_os = "macos")]
fn next_notification_identifier() -> String {
    format!(
        "pr-buddy-notification-{}",
        NOTIFICATION_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(target_os = "macos")]
fn mark_notification_handled(identifier: &str) -> bool {
    let handled = HANDLED_NOTIFICATION_IDS.get_or_init(|| Mutex::new(HashSet::new()));
    let mut handled = handled
        .lock()
        .expect("handled notification id set should not be poisoned");
    if handled.len() >= 256 {
        handled.clear();
    }
    handled.insert(identifier.to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_click_payload, extract_clicked_url};

    #[test]
    fn payload_round_trips_pr_url() {
        let payload = build_click_payload("https://github.com/coder/pr-buddy/pull/42");

        assert_eq!(
            extract_clicked_url(&payload),
            Some("https://github.com/coder/pr-buddy/pull/42".to_string())
        );
    }

    #[test]
    fn extract_clicked_url_returns_none_when_payload_key_is_missing() {
        let payload = std::collections::HashMap::from([(
            "other_key".to_string(),
            "https://github.com/coder/pr-buddy/pull/42".to_string(),
        )]);

        assert_eq!(extract_clicked_url(&payload), None);
    }

    #[test]
    fn extract_clicked_url_returns_none_for_empty_or_invalid_payload() {
        let empty_payload = build_click_payload("   ");
        let invalid_payload = build_click_payload("github.com/coder/pr-buddy/pull/42");

        assert_eq!(extract_clicked_url(&empty_payload), None);
        assert_eq!(extract_clicked_url(&invalid_payload), None);
    }
}
