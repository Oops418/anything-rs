#import "quicklook_shim.h"
#import <Cocoa/Cocoa.h>
#import <QuickLookUI/QuickLookUI.h>

#pragma mark - QLPreviewItem
@interface QLSItem : NSObject <QLPreviewItem>
@property (nonatomic, strong) NSURL *url;
@end
@implementation QLSItem
- (NSURL *)previewItemURL { return self.url; }
@end

#pragma mark - DataSource & Delegate
@interface QLSDataSource : NSObject <
        QLPreviewPanelDataSource,
        QLPreviewPanelDelegate>
@property (nonatomic, strong) QLSItem *item;
@end

@implementation QLSDataSource
- (NSInteger)numberOfPreviewItemsInPreviewPanel:(QLPreviewPanel *)panel { return 1; }
- (id<QLPreviewItem>)previewPanel:(QLPreviewPanel *)panel
              previewItemAtIndex:(NSInteger)idx
{
    return self.item;
}
@end

static QLSDataSource *ds;

void ql_preview(const char *c_path)
{
    @autoreleasepool {
        if (!ds) ds = [QLSDataSource new];

        NSString *path = [NSString stringWithUTF8String:c_path];
        ds.item = [QLSItem new];
        ds.item.url = [NSURL fileURLWithPath:path];

        QLPreviewPanel *panel = [QLPreviewPanel sharedPreviewPanel];
        [panel setDataSource:ds];
        [panel setDelegate:ds];

        if (panel.isVisible) {
            [panel reloadData];
        } else {
            [panel makeKeyAndOrderFront:nil];
        }
    }
}

void ql_close(void)
{
    QLPreviewPanel *panel = [QLPreviewPanel sharedPreviewPanelExists] ?
                            [QLPreviewPanel sharedPreviewPanel] : nil;
    if (panel && panel.isVisible) {
        [panel orderOut:nil];
    }
}