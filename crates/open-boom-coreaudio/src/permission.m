#import <Foundation/Foundation.h>
#import <AVFoundation/AVFoundation.h>
#import <AppKit/AppKit.h>

// Returns:
// 0 = NotDetermined
// 1 = Restricted
// 2 = Denied
// 3 = Authorized
int open_boom_get_microphone_authorization_status(void) {
    if (@available(macOS 10.14, *)) {
        return (int)[AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio];
    }
    return 3;
}

typedef void (*OpenBoomAuthCallback)(int granted, void *context);

void open_boom_request_microphone_authorization(OpenBoomAuthCallback callback, void *context) {
    if (@available(macOS 10.14, *)) {
        AVAuthorizationStatus status = [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio];
        if (status == AVAuthorizationStatusAuthorized) {
            if (callback) callback(1, context);
            return;
        } else if (status == AVAuthorizationStatusDenied || status == AVAuthorizationStatusRestricted) {
            if (callback) callback(0, context);
            return;
        }

        [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL granted) {
            if (callback) {
                callback(granted ? 1 : 0, context);
            }
        }];
    } else {
        if (callback) callback(1, context);
    }
}

void open_boom_open_microphone_privacy_settings(void) {
    NSString *urlString = @"x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone";
    NSURL *url = [NSURL URLWithString:urlString];
    [[NSWorkspace sharedWorkspace] openURL:url];
}
