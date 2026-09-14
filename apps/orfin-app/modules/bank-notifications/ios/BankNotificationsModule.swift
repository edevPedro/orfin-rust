import ExpoModulesCore

public class BankNotificationsModule: Module {
  public func definition() -> ModuleDefinition {
    Name("BankNotifications")

    Events("onBankNotification")

    AsyncFunction("isEnabled") { () -> Bool in
      false
    }

    AsyncFunction("openSettings") {
      // NotificationListener is Android-only.
    }

    AsyncFunction("configure") { (_: String, _: String) in
      // no-op on iOS — use Pluggy Open Finance instead.
    }
  }
}
