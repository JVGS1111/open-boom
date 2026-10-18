//! macOS Microphone Privacy Permission handling (AVFoundation TCC).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicrophoneAuthStatus {
    NotDetermined = 0,
    Restricted = 1,
    Denied = 2,
    Authorized = 3,
}

impl MicrophoneAuthStatus {
    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized)
    }

    pub fn can_prompt(&self) -> bool {
        matches!(self, Self::NotDetermined)
    }
}

#[cfg(target_os = "macos")]
mod native {
    use super::MicrophoneAuthStatus;
    use std::sync::mpsc::channel;

    extern "C" {
        fn open_boom_get_microphone_authorization_status() -> libc::c_int;
        fn open_boom_request_microphone_authorization(
            callback: extern "C" fn(libc::c_int, *mut libc::c_void),
            context: *mut libc::c_void,
        );
        fn open_boom_open_microphone_privacy_settings();
    }

    pub fn get_status() -> MicrophoneAuthStatus {
        let code = unsafe { open_boom_get_microphone_authorization_status() };
        match code {
            0 => MicrophoneAuthStatus::NotDetermined,
            1 => MicrophoneAuthStatus::Restricted,
            2 => MicrophoneAuthStatus::Denied,
            3 => MicrophoneAuthStatus::Authorized,
            _ => MicrophoneAuthStatus::NotDetermined,
        }
    }

    extern "C" fn auth_callback(granted: libc::c_int, context: *mut libc::c_void) {
        if !context.is_null() {
            let tx = unsafe { Box::from_raw(context as *mut std::sync::mpsc::Sender<bool>) };
            let _ = tx.send(granted == 1);
        }
    }

    pub fn request_access_sync() -> bool {
        let (tx, rx) = channel();
        let box_tx = Box::into_raw(Box::new(tx));
        unsafe {
            open_boom_request_microphone_authorization(auth_callback, box_tx as *mut libc::c_void);
        }
        rx.recv().unwrap_or(false)
    }

    pub fn request_access_async<F: FnOnce(bool) + Send + 'static>(callback: F) {
        std::thread::spawn(move || {
            let res = request_access_sync();
            callback(res);
        });
    }

    pub fn open_privacy_settings() {
        unsafe {
            open_boom_open_microphone_privacy_settings();
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod native {
    use super::MicrophoneAuthStatus;

    pub fn get_status() -> MicrophoneAuthStatus {
        MicrophoneAuthStatus::Authorized
    }

    pub fn request_access_sync() -> bool {
        true
    }

    pub fn request_access_async<F: FnOnce(bool) + Send + 'static>(callback: F) {
        callback(true);
    }

    pub fn open_privacy_settings() {}
}

pub use native::*;
