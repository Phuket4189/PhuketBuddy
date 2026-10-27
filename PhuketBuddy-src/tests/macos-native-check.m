#import <AppKit/AppKit.h>
#include <assert.h>
void pb_work_area(int, int, int *);
void pb_window_work_area(void *, int *);
bool pb_left_button_down(void);
bool pb_is_active(void);
int main(void) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        int r[4];
        pb_work_area(0, 0, r);
        assert(r[2] > 0 && r[3] > 0);
        printf("Primary usable area: %d,%d %dx%d\n", r[0], r[1], r[2], r[3]);
        NSWindow *window = [[NSWindow alloc] initWithContentRect:NSMakeRect(100, 100, 72, 72) styleMask:NSWindowStyleMaskBorderless backing:NSBackingStoreBuffered defer:NO];
        pb_window_work_area((__bridge void *)window, r);
        assert(r[2] > 0 && r[3] > 0);
        pb_work_area(-100000, -100000, r);
        assert(r[2] > 0 && r[3] > 0);
        printf("Off-screen fallback: %d,%d %dx%d; mouse=%d active=%d\n", r[0], r[1], r[2], r[3], pb_left_button_down(), pb_is_active());
    }
}
