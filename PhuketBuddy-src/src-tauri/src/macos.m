#import <AppKit/AppKit.h>
#import <CoreGraphics/CoreGraphics.h>

void pb_work_area(int x, int y, int *out) {
    void (^readScreens)(void) = ^{
        NSArray<NSScreen *> *screens = [NSScreen screens];
        NSScreen *primary = screens.firstObject;
        if (!primary) { out[0] = 0; out[1] = 0; out[2] = 1920; out[3] = 1080; return; }
        CGFloat top = NSMaxY(primary.frame);
        NSScreen *chosen = primary;
        double best = DBL_MAX;
        for (NSScreen *screen in screens) {
            CGFloat scale = screen.backingScaleFactor;
            NSRect frame = screen.frame;
            double left = frame.origin.x * scale;
            double upper = (top - NSMaxY(frame)) * scale;
            double dx = MAX(MAX(left - x, 0), x - (left + frame.size.width * scale));
            double dy = MAX(MAX(upper - y, 0), y - (upper + frame.size.height * scale));
            if (dx * dx + dy * dy < best) { best = dx * dx + dy * dy; chosen = screen; }
        }
        NSRect visible = chosen.visibleFrame;
        CGFloat scale = chosen.backingScaleFactor;
        out[0] = (int)round(visible.origin.x * scale);
        out[1] = (int)round((top - NSMaxY(visible)) * scale);
        out[2] = (int)round(visible.size.width * scale);
        out[3] = (int)round(visible.size.height * scale);
    };
    if ([NSThread isMainThread]) readScreens();
    else dispatch_sync(dispatch_get_main_queue(), readScreens);
}
bool pb_left_button_down(void) { return CGEventSourceButtonState(kCGEventSourceStateCombinedSessionState, kCGMouseButtonLeft); }
bool pb_is_active(void) {
    __block bool active;
    void (^readActive)(void) = ^{ active = NSApp.active; };
    if ([NSThread isMainThread]) readActive();
    else dispatch_sync(dispatch_get_main_queue(), readActive);
    return active;
}

void pb_window_work_area(void *window, int *out) {
    void (^readScreen)(void) = ^{
        NSWindow *native = (__bridge NSWindow *)window;
        NSScreen *screen = native.screen ?: [NSScreen screens].firstObject;
        if (!screen) { out[0] = 0; out[1] = 0; out[2] = 1920; out[3] = 1080; return; }
        CGFloat top = NSMaxY([NSScreen screens].firstObject.frame);
        NSRect visible = screen.visibleFrame;
        CGFloat scale = native.backingScaleFactor;
        out[0] = (int)round(visible.origin.x * scale);
        out[1] = (int)round((top - NSMaxY(visible)) * scale);
        out[2] = (int)round(visible.size.width * scale);
        out[3] = (int)round(visible.size.height * scale);
    };
    if ([NSThread isMainThread]) readScreen();
    else dispatch_sync(dispatch_get_main_queue(), readScreen);
}
