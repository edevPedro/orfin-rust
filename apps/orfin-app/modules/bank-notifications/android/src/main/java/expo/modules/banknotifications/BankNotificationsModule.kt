package expo.modules.banknotifications

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.provider.Settings
import android.text.TextUtils
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

class BankNotificationsModule : Module() {
  override fun definition() = ModuleDefinition {
    Name("BankNotifications")

    Events("onBankNotification")

    AsyncFunction("isEnabled") {
      notificationListenerEnabled(appContext.reactContext!!)
    }

    AsyncFunction("openSettings") {
      val context = appContext.reactContext!!
      val intent = Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS).apply {
        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
      }
      context.startActivity(intent)
    }

    AsyncFunction("configure") { apiUrl: String, userId: String ->
      appContext.reactContext!!
        .getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        .edit()
        .putString(KEY_API_URL, apiUrl.trimEnd('/'))
        .putString(KEY_USER_ID, userId)
        .apply()
      BankNotificationListenerService.moduleRef = this@BankNotificationsModule
    }

    OnCreate {
      BankNotificationListenerService.moduleRef = this@BankNotificationsModule
    }

    OnDestroy {
      if (BankNotificationListenerService.moduleRef === this@BankNotificationsModule) {
        BankNotificationListenerService.moduleRef = null
      }
    }
  }

  fun emitNotification(packageName: String, title: String, text: String, postedAt: Long) {
    sendEvent(
      "onBankNotification",
      mapOf(
        "packageName" to packageName,
        "title" to title,
        "text" to text,
        "postedAt" to postedAt.toDouble()
      )
    )
  }

  companion object {
    const val PREFS = "orfin_bank_notifications"
    const val KEY_API_URL = "api_url"
    const val KEY_USER_ID = "user_id"
  }
}

private fun notificationListenerEnabled(context: Context): Boolean {
  val flat = Settings.Secure.getString(
    context.contentResolver,
    "enabled_notification_listeners"
  ) ?: return false
  val expected = ComponentName(context, BankNotificationListenerService::class.java)
  val splitter = TextUtils.SimpleStringSplitter(':')
  splitter.setString(flat)
  while (splitter.hasNext()) {
    val component = ComponentName.unflattenFromString(splitter.next())
    if (component != null && component == expected) return true
  }
  return false
}
