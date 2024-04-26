import UIKit

private typealias ApplicationWithDeviceToken = @convention(c) (Any, Selector, UIApplication, Data) -> Void
private typealias ApplicationFailedToRegisterForRemoteNotification = @convention(c) (Any, Selector, UIApplication, NSError) -> Void

private struct AssociatedObjectKeys {
    static var originalClass = "bevy_ios_notifications_OriginalClass"
    static var originalImplementations = "bevy_ios_notifications_OriginalImplementations"
}

private var gOriginalAppDelegate: UIApplicationDelegate?
private var gAppDelegateSubClass: AnyClass?

public class AppDelegateSwizzler: NSProxy {

    public static func setup() {
        // Let the property be initialized and run its block.
        _ = runOnce
    }

    /// Using Swift's lazy evaluation of a static property we get the same
    /// thread-safety and called-once guarantees as dispatch_once provided.
    private static let runOnce: () = {
        weak var appDelegate = UIApplication.shared.delegate
        proxyAppDelegate(appDelegate)
    }()

    private static func proxyAppDelegate(_ appDelegate: UIApplicationDelegate?) {
        guard let appDelegate = appDelegate else {
            NSLog("Cannot proxy AppDelegate. Instance is nil.")
            return
        }

        gAppDelegateSubClass = createSubClass(from: appDelegate)
        self.reassignAppDelegate()
    }

    private static func reassignAppDelegate() {
        weak var delegate = UIApplication.shared.delegate
        UIApplication.shared.delegate = nil
        UIApplication.shared.delegate = delegate
        gOriginalAppDelegate = delegate
        // TODO observe UIApplication
    }

    private static func createSubClass(from originalDelegate: UIApplicationDelegate) -> AnyClass? {
        let originalClass = type(of: originalDelegate)
        let newClassName = "\(originalClass)_\(UUID().uuidString)"

        guard NSClassFromString(newClassName) == nil else {
            NSLog("Cannot create subclass. Subclass already exists.")
            return nil
        }

        guard let subClass = objc_allocateClassPair(originalClass, newClassName, 0) else {
            NSLog("Cannot create subclass. Subclass already exists.")
            return nil
        }

        self.createMethodImplementations(in: subClass, withOriginalDelegate: originalDelegate)
        self.overrideDescription(in: subClass)

        // Store the original class
        objc_setAssociatedObject(originalDelegate, &AssociatedObjectKeys.originalClass, originalClass, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)

        guard class_getInstanceSize(originalClass) == class_getInstanceSize(subClass) else {
            NSLog("Cannot create subclass. Original class' and subclass' sizes do not match.")
            return nil
        }

        objc_registerClassPair(subClass)
        if object_setClass(originalDelegate, subClass) == nil {
            NSLog("Error creating proxy.")
        }

        return subClass
    }

    private static func createMethodImplementations(
            in subClass: AnyClass,
            withOriginalDelegate originalDelegate: UIApplicationDelegate
    ) {
        let originalClass = type(of: originalDelegate)
        var originalImplementationsStore: [String: NSValue] = [:]
        
        // For didRegisterForRemoteNotificationsWithDeviceToken:
        let applicationWithDeviceToken = #selector(self.application(_:didRegisterForRemoteNotificationsWithDeviceToken:))
        self.proxyInstanceMethod(
                toClass: subClass,
                withSelector: applicationWithDeviceToken,
                fromClass: AppDelegateSwizzler.self,
                fromSelector: applicationWithDeviceToken,
                withOriginalClass: originalClass,
                storeOriginalImplementationInto: &originalImplementationsStore)
        
        // For didRegisterForRemoteNotificationsWithDeviceToken:
        let applicationFailedToRegisterForRemoteNotifications = #selector(self.application(_:didFailToRegisterForRemoteNotificationsWithError:))
        self.proxyInstanceMethod(
                toClass: subClass,
                withSelector: applicationFailedToRegisterForRemoteNotifications,
                fromClass: AppDelegateSwizzler.self,
                fromSelector: applicationFailedToRegisterForRemoteNotifications,
                withOriginalClass: originalClass,
                storeOriginalImplementationInto: &originalImplementationsStore)

        // Store original implementations
        objc_setAssociatedObject(originalDelegate, &AssociatedObjectKeys.originalImplementations, originalImplementationsStore, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
    }

    private static func overrideDescription(in subClass: AnyClass) {
        // Override the description so the custom class name will not show up.
        self.addInstanceMethod(
                toClass: subClass,
                toSelector: #selector(description),
                fromClass: AppDelegateSwizzler.self,
                fromSelector: #selector(originalDescription))
    }

    private static func proxyInstanceMethod(
            toClass destinationClass: AnyClass,
            withSelector destinationSelector: Selector,
            fromClass sourceClass: AnyClass,
            fromSelector sourceSelector: Selector,
            withOriginalClass originalClass: AnyClass,
            storeOriginalImplementationInto originalImplementationsStore: inout [String: NSValue]
    ) {
        self.addInstanceMethod(
                toClass: destinationClass,
                toSelector: destinationSelector,
                fromClass: sourceClass,
                fromSelector: sourceSelector)

        let sourceImplementation = methodImplementation(for: destinationSelector, from: originalClass)
        let sourceImplementationPointer = NSValue(pointer: UnsafePointer(sourceImplementation))

        let destinationSelectorStr = NSStringFromSelector(destinationSelector)
        originalImplementationsStore[destinationSelectorStr] = sourceImplementationPointer
    }

    private static func addInstanceMethod(
            toClass destinationClass: AnyClass,
            toSelector destinationSelector: Selector,
            fromClass sourceClass: AnyClass,
            fromSelector sourceSelector: Selector
    ) {
        let method = class_getInstanceMethod(sourceClass, sourceSelector)!
        let methodImplementation = method_getImplementation(method)
        let methodTypeEncoding = method_getTypeEncoding(method)

        if !class_addMethod(destinationClass, destinationSelector, methodImplementation, methodTypeEncoding) {
            NSLog("Cannot copy method to destination selector '\(destinationSelector)' as it already exists.")
        }
    }

    private static func methodImplementation(for selector: Selector, from fromClass: AnyClass) -> IMP? {
        guard let method = class_getInstanceMethod(fromClass, selector) else {
            return nil
        }

        return method_getImplementation(method)
    }

    private static func originalMethodImplementation(for selector: Selector, object: Any) -> NSValue? {
        let originalImplementationsStore = objc_getAssociatedObject(object, &AssociatedObjectKeys.originalImplementations) as? [String: NSValue]
        return originalImplementationsStore?[NSStringFromSelector(selector)]
    }

    @objc
    private func originalDescription() -> String {
        let originalClass: AnyClass = objc_getAssociatedObject(self, &AssociatedObjectKeys.originalClass) as! AnyClass

        let originalClassName = NSStringFromClass(originalClass)
        let pointerHex = String(format: "%p", unsafeBitCast(self, to: Int.self))

        return "<\(originalClassName): \(pointerHex)>"
    }
    
    @objc 
    private func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let deviceTokenString = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
        
        BevyNotifications.SendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
            $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.remoteNotificationRegistration(BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.with{
                $0.results = BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.OneOf_Results.token(.with{
                    $0.token = deviceTokenString
                })
            })
        })
        
        NSLog("Framework: application didRegisterForRemoteNotificationsWithDeviceToken")

        let methodSelector = #selector(self.application(_:didRegisterForRemoteNotificationsWithDeviceToken:))
        guard let pointer = AppDelegateSwizzler.originalMethodImplementation(for: methodSelector, object: self),
              let pointerValue = pointer.pointerValue else {

            NSLog("No original implementation for 'application didRegisterForRemoteNotificationsWithDeviceToken'. Skipping...")
            return
        }

        let originalImplementation = unsafeBitCast(pointerValue, to: ApplicationWithDeviceToken.self)
        originalImplementation(self, methodSelector, application, deviceToken)
    }
    
    @objc 
    private func application(_ application: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: NSError) {
        BevyNotifications.SendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
            $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.remoteNotificationRegistration(BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.with{
                $0.results = BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.OneOf_Results.failed(.with{
                    $0.code = Int32(error.code)
                    $0.localizedDescription = error.localizedDescription
                })
            })
        })
        
        NSLog("Framework: application didFailToRegisterForRemoteNotificationsWithError")

        let methodSelector = #selector(self.application(_:didFailToRegisterForRemoteNotificationsWithError:))
        guard let pointer = AppDelegateSwizzler.originalMethodImplementation(for: methodSelector, object: self),
              let pointerValue = pointer.pointerValue else {

            NSLog("No original implementation for 'application didFailToRegisterForRemoteNotificationsWithError'. Skipping...")
            return
        }

        let originalImplementation = unsafeBitCast(pointerValue, to: ApplicationFailedToRegisterForRemoteNotification.self)
        originalImplementation(self, methodSelector, application, error)
    }
}
