// SPDX-License-Identifier: AGPL-3.0-only
package org.kuracare.app

import android.Manifest
import android.app.Activity
import android.app.AlarmManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.content.res.Configuration
import android.os.Build
import android.provider.Settings
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.view.WindowCompat
import org.json.JSONObject

/**
 * Invitations ("Ti va di…?").
 *
 * The plan is computed in Rust (kuracare-core/src/reminders.rs) and holds only
 * instants, keys and generic texts: nothing personal. It is kept in the
 * app-private preferences (the app has no backups) so that invitations arrive
 * while Kuracare is locked or closed, and again after a reboot.
 *
 * Called from Rust over JNI (crates/kuracare-app/src/notify.rs): keep the
 * method names and signatures in sync, and in kuracare-rules.pro.
 */
object Bridge {
    const val EXTRA_KIND = "org.kuracare.app.INVITE_KIND"
    const val EXTRA_KEY = "org.kuracare.app.INVITE_KEY"
    const val ACTION_FIRE = "org.kuracare.app.FIRE"
    const val ACTION_NOT_NOW = "org.kuracare.app.NOT_NOW"
    const val NOTE_ID = 1
    private const val CHANNEL_ID = "invites"
    private const val PREFS = "invites"
    /** An alarm that comes a little early still shows what is due. */
    private const val EARLY_MS = 60_000L
    /**
     * Allowed delay of an alarm: 10 minutes is the shortest window Android grants
     * without the "exact alarms" permission. (setAndAllowWhileIdle was measured on
     * the Pixel, 21/11/2026, with a window of +1 hour: too late for an invitation.)
     */
    private const val WINDOW_MS = 10 * 60_000L

    private class Item(
        val key: String, val kind: String, val at: Long, val until: Long,
        val title: String, val body: String,
    )

    private class Plan(val channel: String, val notNow: String, val items: List<Item>)

    @Volatile
    private var pending: String? = null

    private fun prefs(ctx: Context) = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    private fun parse(json: String?): Plan? = try {
        val o = JSONObject(json ?: return null)
        val a = o.getJSONArray("items")
        Plan(
            o.getString("channel"), o.getString("notNow"),
            (0 until a.length()).map {
                val i = a.getJSONObject(it)
                Item(
                    i.getString("key"), i.getString("kind"), i.getLong("at"), i.getLong("until"),
                    i.getString("title"), i.getString("body"),
                )
            },
        )
    } catch (_: Exception) {
        null
    }

    private fun plan(ctx: Context) = parse(prefs(ctx).getString("plan", null))

    // ---- called from Rust ----

    @JvmStatic
    fun setPlan(ctx: Context, json: String) {
        val app = ctx.applicationContext
        val p = prefs(app)
        val e = p.edit().putString("plan", json)
        // The very first plan never shows invitations whose moment is already past.
        if (!p.contains("lastFired")) e.putLong("lastFired", System.currentTimeMillis())
        e.commit()
        cancelGone(app, parse(json))
        scheduleNext(app)
    }

    @JvmStatic
    fun notificationsAllowed(ctx: Context): Boolean =
        NotificationManagerCompat.from(ctx).areNotificationsEnabled()

    @JvmStatic
    fun askNotifications(ctx: Context) {
        if (Build.VERSION.SDK_INT >= 33 && ctx is Activity &&
            ActivityCompat.checkSelfPermission(ctx, Manifest.permission.POST_NOTIFICATIONS)
            != PackageManager.PERMISSION_GRANTED
        ) {
            ctx.runOnUiThread {
                ActivityCompat.requestPermissions(ctx, arrayOf(Manifest.permission.POST_NOTIFICATIONS), 41)
            }
        }
    }

    @JvmStatic
    fun openNotificationSettings(ctx: Context) {
        val i = Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
            .putExtra(Settings.EXTRA_APP_PACKAGE, ctx.packageName)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        ctx.startActivity(i)
    }

    /** Shows the first item of `json` (plan format) right away. */
    @JvmStatic
    fun showNow(ctx: Context, json: String) {
        val plan = parse(json) ?: return
        val item = plan.items.firstOrNull() ?: return
        show(ctx.applicationContext, item, plan)
    }

    /**
     * Status and navigation bars in the theme's background colour.
     * mode: 0 as the phone (day/evening), 1 always light, 2 always dark.
     */
    @JvmStatic
    fun setBars(ctx: Context, day: Int, night: Int, mode: Int) {
        val a = ctx as? Activity ?: return
        a.runOnUiThread {
            val phoneNight = (a.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) ==
                Configuration.UI_MODE_NIGHT_YES
            val dark = when (mode) {
                1 -> false
                2 -> true
                else -> phoneNight
            }
            val color = if (dark) night else day
            @Suppress("DEPRECATION")
            a.window.statusBarColor = color
            @Suppress("DEPRECATION")
            a.window.navigationBarColor = color
            WindowCompat.getInsetsController(a.window, a.window.decorView).apply {
                isAppearanceLightStatusBars = !dark
                isAppearanceLightNavigationBars = !dark
            }
        }
    }

    /** "kind|key" of the invitation that opened the app, once; null otherwise. */
    @JvmStatic
    fun takeInvite(): String? {
        val p = pending
        pending = null
        return p
    }

    // ---- Android side ----

    fun noteIntent(ctx: Context, intent: Intent?) {
        val kind = intent?.getStringExtra(EXTRA_KIND) ?: return
        val key = intent.getStringExtra(EXTRA_KEY) ?: ""
        intent.removeExtra(EXTRA_KIND)
        intent.removeExtra(EXTRA_KEY)
        NotificationManagerCompat.from(ctx).cancel(key, NOTE_ID)
        pending = "$kind|$key"
    }

    private fun firePending(ctx: Context): PendingIntent = PendingIntent.getBroadcast(
        ctx, 0, Intent(ctx, InviteReceiver::class.java).setAction(ACTION_FIRE),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )

    /** One alarm at a time: the next moment after the last one shown, within 10 minutes. */
    fun scheduleNext(ctx: Context) {
        val am = ctx.getSystemService(AlarmManager::class.java) ?: return
        val pi = firePending(ctx)
        am.cancel(pi)
        val last = prefs(ctx).getLong("lastFired", System.currentTimeMillis())
        val next = plan(ctx)?.items?.map { it.at }?.filter { it > last }?.minOrNull() ?: return
        am.setWindow(AlarmManager.RTC_WAKEUP, next, WINDOW_MS, pi)
    }

    fun fire(ctx: Context) {
        val now = System.currentTimeMillis()
        val p = prefs(ctx)
        val last = p.getLong("lastFired", now)
        val plan = plan(ctx)
        if (plan != null && notificationsAllowed(ctx)) {
            for (it in plan.items) {
                if (it.at > last && it.at <= now + EARLY_MS && it.until > now) show(ctx, it, plan)
            }
        }
        p.edit().putLong("lastFired", now + EARLY_MS).commit()
        scheduleNext(ctx)
    }

    /** Invitations no longer in the plan (answered, done, paused, turned off) go away. */
    private fun cancelGone(ctx: Context, plan: Plan?) {
        val keys = plan?.items?.map { it.key }?.toSet() ?: emptySet()
        val nm = ctx.getSystemService(NotificationManager::class.java) ?: return
        for (sbn in nm.activeNotifications) {
            val tag = sbn.tag ?: continue
            if (tag !in keys && !tag.startsWith("test")) nm.cancel(tag, sbn.id)
        }
    }

    private fun show(ctx: Context, it: Item, plan: Plan) {
        if (Build.VERSION.SDK_INT >= 26) {
            val nm = ctx.getSystemService(NotificationManager::class.java)
            nm?.createNotificationChannel(
                NotificationChannel(CHANNEL_ID, plan.channel, NotificationManager.IMPORTANCE_DEFAULT),
            )
        }
        val flags = PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        val open = Intent(ctx, dev.dioxus.main.MainActivity::class.java)
            .putExtra(EXTRA_KIND, it.kind)
            .putExtra(EXTRA_KEY, it.key)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        val notNow = Intent(ctx, InviteReceiver::class.java)
            .setAction(ACTION_NOT_NOW)
            .putExtra(EXTRA_KEY, it.key)
        // On the lock screen: only the app name.
        val public = NotificationCompat.Builder(ctx, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notify)
            .setContentTitle("Kuracare")
            .build()
        val b = NotificationCompat.Builder(ctx, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notify)
            .setColor(0xFF3F6B5A.toInt())
            .setContentTitle(it.title)
            .setContentText(it.body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(it.body))
            .setContentIntent(PendingIntent.getActivity(ctx, it.key.hashCode(), open, flags))
            .addAction(0, plan.notNow, PendingIntent.getBroadcast(ctx, it.key.hashCode(), notNow, flags))
            .setAutoCancel(true)
            .setOnlyAlertOnce(true)
            .setCategory(NotificationCompat.CATEGORY_REMINDER)
            .setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
            .setPublicVersion(public)
        val left = it.until - System.currentTimeMillis()
        if (left > 0) b.setTimeoutAfter(left)
        try {
            NotificationManagerCompat.from(ctx).notify(it.key, NOTE_ID, b.build())
        } catch (_: SecurityException) {
            // Notifications not allowed: nothing to do, never insist.
        }
    }
}

/** Alarm and "Non ora" (not exported: only our own PendingIntents reach it). */
class InviteReceiver : BroadcastReceiver() {
    override fun onReceive(ctx: Context, intent: Intent) {
        when (intent.action) {
            Bridge.ACTION_FIRE -> Bridge.fire(ctx)
            Bridge.ACTION_NOT_NOW ->
                NotificationManagerCompat.from(ctx).cancel(intent.getStringExtra(Bridge.EXTRA_KEY), Bridge.NOTE_ID)
        }
    }
}

/** After a reboot, an update or a clock change: arm the next alarm again. */
class RestartReceiver : BroadcastReceiver() {
    override fun onReceive(ctx: Context, intent: Intent) {
        when (intent.action) {
            Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED,
            Intent.ACTION_TIME_CHANGED, Intent.ACTION_TIMEZONE_CHANGED -> Bridge.scheduleNext(ctx)
        }
    }
}
