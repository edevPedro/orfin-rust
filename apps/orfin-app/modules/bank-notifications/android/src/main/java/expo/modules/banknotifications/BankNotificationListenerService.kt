package expo.modules.banknotifications

import android.app.Notification
import android.content.Context
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Log
import org.json.JSONObject
import java.io.OutputStreamWriter
import java.net.HttpURLConnection
import java.net.URL
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone
import java.util.concurrent.Executors
import kotlin.math.floor

class BankNotificationListenerService : NotificationListenerService() {
  private val executor = Executors.newSingleThreadExecutor()

  override fun onNotificationPosted(sbn: StatusBarNotification?) {
    if (sbn == null || sbn.isOngoing) return
    val packageName = sbn.packageName ?: return
    if (!looksLikeBank(packageName)) return

    val extras = sbn.notification?.extras ?: return
    val title = extras.getCharSequence(Notification.EXTRA_TITLE)?.toString().orEmpty()
    val text = sequenceOf(
      extras.getCharSequence(Notification.EXTRA_BIG_TEXT),
      extras.getCharSequence(Notification.EXTRA_TEXT),
      extras.getCharSequence(Notification.EXTRA_SUB_TEXT)
    ).mapNotNull { it?.toString()?.takeIf(String::isNotBlank) }
      .firstOrNull()
      .orEmpty()

    if (title.isBlank() && text.isBlank()) return
    val combined = "$title $text"
    if (!combined.contains("R$", ignoreCase = true) &&
      !combined.contains("compra", ignoreCase = true) &&
      !combined.contains("pix", ignoreCase = true) &&
      !combined.contains("pagamento", ignoreCase = true)
    ) {
      return
    }

    val postedAt = sbn.postTime
    moduleRef?.emitNotification(packageName, title, text, postedAt)

    executor.execute {
      postToBackend(packageName, title, text, postedAt)
    }
  }

  private fun postToBackend(packageName: String, title: String, text: String, postedAt: Long) {
    val prefs = getSharedPreferences(BankNotificationsModule.PREFS, Context.MODE_PRIVATE)
    val apiUrl = prefs.getString(BankNotificationsModule.KEY_API_URL, null) ?: return
    val userId = prefs.getString(BankNotificationsModule.KEY_USER_ID, null) ?: return

    val amount = parseAmount("$title $text") ?: return
    val merchant = parseMerchant(title, text)
    val externalId = fingerprint(packageName, "$title|$text", postedAt)

    val payload = JSONObject()
      .put("user_id", userId)
      .put("external_id", externalId)
      .put("amount", amount)
      .put("currency", "BRL")
      .put("description", text.ifBlank { title })
      .put("merchant", merchant)
      .put("paid_at", iso8601(postedAt))
      .put(
        "raw_payload",
        JSONObject()
          .put("package", packageName)
          .put("title", title)
          .put("text", text)
      )

    try {
      val url = URL("$apiUrl/payments/from-notification")
      val conn = (url.openConnection() as HttpURLConnection).apply {
        requestMethod = "POST"
        connectTimeout = 8_000
        readTimeout = 8_000
        doOutput = true
        setRequestProperty("Content-Type", "application/json")
        setRequestProperty("Accept", "application/json")
      }
      OutputStreamWriter(conn.outputStream).use { it.write(payload.toString()) }
      Log.i(TAG, "posted notification payment status=${conn.responseCode}")
      conn.disconnect()
    } catch (error: Exception) {
      Log.w(TAG, "failed to post notification payment", error)
    }
  }

  companion object {
    private const val TAG = "OrfinBankNls"

    @JvmField
    var moduleRef: BankNotificationsModule? = null

    private fun looksLikeBank(packageName: String): Boolean {
      val known = setOf(
        "com.nu.production",
        "com.nu.nubank",
        "com.itau",
        "br.com.bradesco",
        "com.santander.app",
        "br.com.bb.android",
        "com.picpay",
        "com.mercadopago.wallet",
        "br.com.intermedium",
        "com.c6bank.app",
        "com.btg.pactual.banking"
      )
      return known.contains(packageName) ||
        packageName.contains("bank") ||
        packageName.contains("banco")
    }

    private fun parseAmount(combined: String): String? {
      val regex = Regex(
        """R\$\s*(\d{1,3}(?:\.\d{3})*,\d{2}|\d+,\d{2}|\d+(?:\.\d{2})?)""",
        RegexOption.IGNORE_CASE
      )
      val match = regex.find(combined) ?: return null
      var raw = match.groupValues[1]
      if (raw.contains(',')) {
        raw = raw.replace(".", "").replace(',', '.')
      }
      val value = raw.toDoubleOrNull() ?: return null
      if (value <= 0) return null
      return String.format(Locale.US, "%.2f", value)
    }

    private fun parseMerchant(title: String, text: String): String? {
      val combined = "$title $text"
      val regex = Regex("""(?:em|no|na)\s+([A-Za-zÀ-ÿ0-9 .&'\-]{3,40})""", RegexOption.IGNORE_CASE)
      val match = regex.find(combined)
      val candidate = match?.groupValues?.getOrNull(1)?.trim()?.replace(Regex("""\s+"""), " ")
      return candidate?.take(60) ?: title.trim().ifBlank { null }
    }

    private fun fingerprint(packageName: String, text: String, postedAt: Long): String {
      val minute = floor(postedAt / 60_000.0).toLong()
      return "$packageName|${text.trim()}|$minute"
    }

    private fun iso8601(epochMs: Long): String {
      val sdf = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss'Z'", Locale.US)
      sdf.timeZone = TimeZone.getTimeZone("UTC")
      return sdf.format(Date(epochMs))
    }
  }
}
