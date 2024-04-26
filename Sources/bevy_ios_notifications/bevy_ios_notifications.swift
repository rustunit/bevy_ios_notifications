import UIKit

@available(iOS 10.0, *)
public class BevyNotifications : NSObject, UNUserNotificationCenterDelegate
{
    static let shared = BevyNotifications()
    
    var callback:((UnsafePointer<CUnsignedChar>, CLong)->())? = nil
    var presentInForeground = false;
    
    internal static func Init(cb: @escaping (UnsafePointer<CUnsignedChar>, CLong)->()) {
        UNUserNotificationCenter.current().delegate = BevyNotifications.shared;
        BevyNotifications.shared.callback = cb;
    }
    
    internal func requestPending() {
        UNUserNotificationCenter.current().getPendingNotificationRequests() { requests in
            
            var response = BevyIos_Notifications_AsyncEvent.Pending();
            
            for r in requests {
                response.items.append(BevyIos_Notifications_AsyncEvent.Pending.PendingNotification.with{
                    $0.identifier = r.identifier
                });
            }
            
            self.sendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
                $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.pending(response)
            })
        }
    }
    
    @available(iOS 15.0, *)
    internal func Schedule(_ request:BevyIos_Notifications_Request.Schedule) -> String {
        let content = UNMutableNotificationContent()
        content.title = request.title
        content.body = request.body
        content.badge = if request.hasBadge {request.badge as NSNumber}else{nil}
        content.categoryIdentifier = request.categoryIdentifier
        content.subtitle = request.subtitle
        content.relevanceScore = request.relevanceScore
        content.threadIdentifier = request.threadIdentifier
        
        content.userInfo = request.userData.data
        
        let id = if request.hasIdentifier {
            request.identifier
        } else {
            UUID().uuidString
        }
        
        let trigger : UNNotificationTrigger? = if request.hasTrigger {
            UNTimeIntervalNotificationTrigger(timeInterval: request.trigger.seconds, repeats: request.trigger.repeat)
        } else { nil }
        
        // Create the request
        let request = UNNotificationRequest(
            identifier: id,
            content: content,
            trigger: trigger)

        // Schedule the request with the system.
        let notificationCenter = UNUserNotificationCenter.current()
        notificationCenter.add(request) { (error) in
            if error != nil {
                self.sendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
                    $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.scheduled(BevyIos_Notifications_AsyncEvent.Scheduled.with{
                        $0.identifier = id
                        $0.error = error!.localizedDescription
                        $0.success = false
                    })
                })
           }else{
               self.sendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
                   $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.scheduled(BevyIos_Notifications_AsyncEvent.Scheduled.with{
                       $0.identifier = id
                       $0.success = true
                   })
               })
           }
        }
        
        return id;
    }
    
    private func sendAsyncEvent(_ response:BevyIos_Notifications_AsyncEvent) {
        var data = try! response.serializedData();
        let data_count = data.count;
        
        data.withUnsafeMutableBytes({ (bytes: UnsafeMutablePointer<UInt8>) -> Void in
            self.callback?(bytes,data_count);
        })
    }
    
    static func SendAsyncEvent(_ response:BevyIos_Notifications_AsyncEvent) {
        BevyNotifications.shared.sendAsyncEvent(response)
    }

    internal func RemoveAllPending() {
        UNUserNotificationCenter.current().removeAllPendingNotificationRequests()
    }
    
    internal func RemoveAllDelivered() {
        UNUserNotificationCenter.current().removeAllDeliveredNotifications()
    }
    
    internal func AskPermission(_ request:BevyIos_Notifications_Request.Permissions) {
    
        let center = UNUserNotificationCenter.current()
        
        var permissions: UNAuthorizationOptions = [];
        
        if request.alert {
            permissions.insert(.alert)
        }
        if request.sound {
            permissions.insert(.sound)
        }
        if request.badge {
            permissions.insert(.badge)
        }
        
        center.requestAuthorization(options: permissions) { granted, error in
            
            if let error = error {
                self.sendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
                    $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.permission(BevyIos_Notifications_AsyncEvent.Permission.with{
                        //TODO: add error
                        $0.granted = false
                    })
                })
            }else{
                self.sendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
                    $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.permission(BevyIos_Notifications_AsyncEvent.Permission.with{
                        $0.granted = true
                    })
                })
            }
        }
    }
    
    public func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions)
            -> Void) {
                
                UIApplication.shared.registerForRemoteNotifications()
                self.sendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
                    $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.triggeredWhileRunning(BevyIos_Notifications_AsyncEvent.NotificationWhileRunning.with{
                        //TODO: provide at least userData back
                        $0.identifier = notification.request.identifier
                    })
                })
                
                if presentInForeground {
                    completionHandler([.alert, .badge, .sound]);
                }
    }
    
    public func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive notification: UNNotificationResponse, withCompletionHandler completionHandler: @escaping () -> Void) {
        
        self.sendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
            $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.notificationResponse(BevyIos_Notifications_AsyncEvent.NotificationResponse.with{
                //TODO: provide at least userData back and trigger so we know whether it was a push
                $0.identifier = notification.notification.request.identifier
                $0.actionIdentifier = notification.actionIdentifier
            })
        })
        
        completionHandler();
    }
    
    //TODO: openSettingsFor delegate
}

class AppDelegateSwizzling {
    
    static var hasOriginal_registerWithToken = false;
    static var hasOriginal_failToRegister = false;
    
    @objc func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let deviceTokenString = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
        
        BevyNotifications.SendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
            $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.remoteNotificationRegistration(BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.with{
                $0.results = BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.OneOf_Results.token(.with{
                    $0.token = deviceTokenString
                })
            })
        })
        
        if AppDelegateSwizzling.hasOriginal_registerWithToken {
            // call original before we swizzled
            self.application(application, didRegisterForRemoteNotificationsWithDeviceToken: deviceToken)
        }
    }
    
    @objc func application(_ application: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: NSError) {
        print("didFailToRegisterForRemoteNotificationsWithError\n")
        
        BevyNotifications.SendAsyncEvent(BevyIos_Notifications_AsyncEvent.with{
            $0.calls = BevyIos_Notifications_AsyncEvent.OneOf_Calls.remoteNotificationRegistration(BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.with{
                $0.results = BevyIos_Notifications_AsyncEvent.RemoteNotificationRegistration.OneOf_Results.failed(.with{
                    $0.code = Int32(error.code)
                    $0.localizedDescription = error.localizedDescription
                })
            })
        })
        
        if AppDelegateSwizzling.hasOriginal_failToRegister {
            // call original before we swizzled
            self.application(application, didFailToRegisterForRemoteNotificationsWithError: error)
        }
    }

    static func swizzleDidRegisterForRemoteNotifications() {
        let appDelegate = UIApplication.shared.delegate
        let appDelegateClass = object_getClass(appDelegate)

        let originalSelector = #selector(UIApplicationDelegate.application(_:didRegisterForRemoteNotificationsWithDeviceToken:))
        let swizzledSelector = #selector(AppDelegateSwizzling.self.application(_:didRegisterForRemoteNotificationsWithDeviceToken:))

        guard let swizzledMethod = class_getInstanceMethod(AppDelegateSwizzling.self, swizzledSelector) else {
            return
        }

        if let originalMethod = class_getInstanceMethod(appDelegateClass, originalSelector)  {
            // exchange implementation
            method_exchangeImplementations(originalMethod, swizzledMethod)
            AppDelegateSwizzling.hasOriginal_registerWithToken = true
        } else {
            // add implementation
            class_addMethod(appDelegateClass, swizzledSelector, method_getImplementation(swizzledMethod), method_getTypeEncoding(swizzledMethod))
        }
    }
    
    static func swizzleDidFailToRegisterForRemoteNotifications() {
        let appDelegate = UIApplication.shared.delegate
        let appDelegateClass = object_getClass(appDelegate)

        let originalSelector = #selector(UIApplicationDelegate.application(_:didFailToRegisterForRemoteNotificationsWithError:))
        let swizzledSelector = #selector(AppDelegateSwizzling.self.application(_:didFailToRegisterForRemoteNotificationsWithError:))

        guard let swizzledMethod = class_getInstanceMethod(AppDelegateSwizzling.self, swizzledSelector) else {
            return
        }

        if let originalMethod = class_getInstanceMethod(appDelegateClass, originalSelector)  {
            // exchange implementation
            method_exchangeImplementations(originalMethod, swizzledMethod)
            AppDelegateSwizzling.hasOriginal_failToRegister = true
        } else {
            // add implementation
            class_addMethod(appDelegateClass, swizzledSelector, method_getImplementation(swizzledMethod), method_getTypeEncoding(swizzledMethod))
        }
    }
}

@_cdecl("swift_notifications_init")
public func swiftNotificationsInit(cb: @escaping (UnsafePointer<CUnsignedChar>, CLong)->()) {
    BevyNotifications.Init(cb:cb)
}

@_cdecl("swift_notifications_register_for_push")
public func swiftNotificationsRegisterForPush() {
    AppDelegateSwizzling.swizzleDidRegisterForRemoteNotifications()
    AppDelegateSwizzling.swizzleDidFailToRegisterForRemoteNotifications()
    UIApplication.shared.registerForRemoteNotifications()
}

@_cdecl("swift_notifications_badge_set")
public func swiftNotificationsBadgeSet(number: Int) {
    UIApplication.shared.applicationIconBadgeNumber = number
}

@_cdecl("swift_notifications_registered_for_push")
public func swiftNotificationsRegisteredForPush() -> Bool {
    UIApplication.shared.isRegisteredForRemoteNotifications
}

@_cdecl("swift_notifications_show_in_foreground_set")
public func swiftNotificationsShowForegrounded(present: Bool) {
    BevyNotifications.shared.presentInForeground = present
}

@_cdecl("swift_notifications_show_in_foreground_get")
public func swiftNotificationsShowForegrounded() -> Bool {
    BevyNotifications.shared.presentInForeground
}

@_cdecl("swift_notifications_badge_get")
public func swiftNotificationsBadgeGet() -> Int {
    UIApplication.shared.applicationIconBadgeNumber
}

@_cdecl("ios_notifications_request")
@available(iOS 15.0, *)
public func request(
    buffer: UnsafeMutablePointer<UInt8>,
    dataSize: CInt,
    dataLen: UnsafeMutablePointer<CInt>
) {
    let data_in = Data(bytes: buffer, count: Int(dataLen.pointee))
    
    let request = try! BevyIos_Notifications_Request(serializedData: data_in)
    
    var response = BevyIos_Notifications_Response()
    
    switch request.calls {
    case .schedule(let schedule):
        let id = BevyNotifications.shared.Schedule(schedule)
        let call_response = BevyIos_Notifications_Response.Schedule.with{$0.identifier = id}
        response.calls = BevyIos_Notifications_Response.OneOf_Calls.schedule(call_response)
    case .pending(let pending):
        let id = BevyNotifications.shared.requestPending()
        response.calls = BevyIos_Notifications_Response.OneOf_Calls.pending(BevyIos_Notifications_Response.Pending())
    case .permissions(let permissions):
        BevyNotifications.shared.AskPermission(permissions)
    case .removeAllDelivered(_):
        BevyNotifications.shared.RemoveAllDelivered()
    case .removeAllPending(_):
        BevyNotifications.shared.RemoveAllPending()
    case .none:
        break
    }
    
    var data_out = try! response.serializedData();
    let data_count = data_out.count;
    data_out.copyBytes(to: buffer, count: data_count)
    dataLen.pointee = CInt(data_count)
}
