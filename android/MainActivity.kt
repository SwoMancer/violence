package dev.dioxus.main

// 1. Import the REAL config using your App ID
import com.example.violence.BuildConfig as AppBuildConfig

// 2. The "Bridge": This makes BuildConfig visible to Logger.kt

import android.app.Activity
import android.app.AlarmManager
import android.app.KeyguardManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.media.AudioAttributes
import android.media.Ringtone
import android.media.RingtoneManager
import android.os.Build
import android.os.Bundle
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.util.Log
import android.util.TypedValue
import android.view.Gravity
import android.view.WindowManager
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale

typealias BuildConfig = AppBuildConfig

class MainActivity : WryActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        // Force the JVM default locale before WryActivity instantiates the WebView,
        // so Chromium's <input type="time"> renders 24-hour instead of AM/PM.
        Locale.setDefault(Locale.UK)
        instance = this  // set before super so Rust can call getFilesDir() during Dioxus init
        super.onCreate(savedInstanceState)

        window.statusBarColor = Color.BLACK
        @Suppress("DEPRECATION")
        window.decorView.systemUiVisibility =
            window.decorView.systemUiVisibility and
                android.view.View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR.inv()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            ActivityCompat.requestPermissions(
                this,
                arrayOf(android.Manifest.permission.POST_NOTIFICATIONS),
                0
            )
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val nm = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            nm.createNotificationChannel(
                NotificationChannel(
                    ALARM_CHANNEL_ID,
                    "Alarms",
                    NotificationManager.IMPORTANCE_HIGH
                ).apply { enableVibration(true) }
            )
            nm.createNotificationChannel(
                NotificationChannel(
                    CONFIRM_CHANNEL_ID,
                    "Alarm confirmations",
                    NotificationManager.IMPORTANCE_HIGH
                ).apply {
                    enableVibration(false)
                    setSound(null, null)
                }
            )
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O_MR1) {
            setShowWhenLocked(true)
            setTurnScreenOn(true)
        } else {
            window.addFlags(
                WindowManager.LayoutParams.FLAG_SHOW_WHEN_LOCKED or
                WindowManager.LayoutParams.FLAG_TURN_SCREEN_ON or
                WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON
            )
        }
    }

    companion object {
        lateinit var instance: MainActivity

        const val ALARM_CHANNEL_ID = "alarm_channel"
        const val CONFIRM_CHANNEL_ID = "alarm_confirm_channel_v2"

        const val ACTION_FIRE = "dev.dioxus.main.FIRE"
        const val ACTION_CONFIRM = "dev.dioxus.main.CONFIRM"
        const val ACTION_REFIRE = "dev.dioxus.main.REFIRE"
        const val ACTION_ACK = "dev.dioxus.main.ACK"

        const val EXTRA_ALARM_ID = "ALARM_ID"
        const val EXTRA_CONFIRM_DELAY_MS = "CONFIRM_DELAY_MS"
        const val EXTRA_REFIRE_DELAY_MS = "REFIRE_DELAY_MS"
        const val EXTRA_REFIRE_AT_MS = "REFIRE_AT_MS"

        const val CONFIRM_DELAY_MS = 5L * 60 * 1000
        const val REFIRE_DELAY_MS = 15L * 60 * 1000

        private const val PREFS_NAME = "violence_settings"
        private const val PREF_CONFIRM_DELAY_MS = "confirm_delay_ms"
        private const val PREF_REFIRE_DELAY_MS = "refire_delay_ms"

        private const val CONFIRM_NOTIF_OFFSET = 100_000

        @JvmStatic
        fun getGlobalConfirmDelayMs(): Long {
            val prefs = instance.applicationContext
                .getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            return prefs.getLong(PREF_CONFIRM_DELAY_MS, CONFIRM_DELAY_MS)
        }

        @JvmStatic
        fun getGlobalRefireDelayMs(): Long {
            val prefs = instance.applicationContext
                .getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            return prefs.getLong(PREF_REFIRE_DELAY_MS, REFIRE_DELAY_MS)
        }

        @JvmStatic
        fun setGlobalSettings(confirmMs: Long, refireMs: Long) {
            val prefs = instance.applicationContext
                .getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.edit()
                .putLong(PREF_CONFIRM_DELAY_MS, confirmMs)
                .putLong(PREF_REFIRE_DELAY_MS, refireMs)
                .apply()
            Log.i("Violence", "setGlobalSettings: confirm=${confirmMs}ms refire=${refireMs}ms")
        }

        @JvmStatic
        fun getFilesDir(): String = instance.filesDir.absolutePath

        private fun phaseOffset(action: String): Int = when (action) {
            ACTION_FIRE -> 0
            ACTION_CONFIRM -> 1
            ACTION_REFIRE -> 2
            ACTION_ACK -> 3
            else -> 0
        }

        fun requestCodeFor(alarmId: Int, action: String): Int =
            alarmId * 10 + phaseOffset(action)

        fun receiverPendingIntent(
            context: Context,
            alarmId: Int,
            action: String,
            extras: Bundle? = null
        ): PendingIntent {
            val intent = Intent(context, AlarmReceiver::class.java).apply {
                this.action = action
                putExtra(EXTRA_ALARM_ID, alarmId)
                if (extras != null) putExtras(extras)
            }
            return PendingIntent.getBroadcast(
                context,
                requestCodeFor(alarmId, action),
                intent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        }

        fun confirmNotifId(alarmId: Int): Int = alarmId + CONFIRM_NOTIF_OFFSET

        @JvmStatic
        fun scheduleAlarmFromHHMM(time: String, id: Int, confirmMs: Long, refireMs: Long) {
            Log.i(
                "Violence",
                "scheduleAlarmFromHHMM called: time=$time id=$id confirm=${confirmMs}ms refire=${refireMs}ms"
            )
            val parts = time.split(":")
            if (parts.size != 2) {
                Log.e("Violence", "Invalid time format: $time")
                return
            }
            val hour = parts[0].toIntOrNull() ?: run {
                Log.e("Violence", "Failed to parse hour from: $time")
                return
            }
            val minute = parts[1].toIntOrNull() ?: run {
                Log.e("Violence", "Failed to parse minute from: $time")
                return
            }

            val cal = Calendar.getInstance()
            cal.set(Calendar.HOUR_OF_DAY, hour)
            cal.set(Calendar.MINUTE, minute)
            cal.set(Calendar.SECOND, 0)
            cal.set(Calendar.MILLISECOND, 0)
            if (cal.timeInMillis <= System.currentTimeMillis()) {
                cal.add(Calendar.DAY_OF_YEAR, 1)
            }

            Log.i("Violence", "Scheduling alarm id=$id for ${cal.time} (${cal.timeInMillis}ms)")
            val extras = Bundle().apply {
                putLong(EXTRA_CONFIRM_DELAY_MS, confirmMs)
                putLong(EXTRA_REFIRE_DELAY_MS, refireMs)
            }
            scheduleFollowup(
                instance.applicationContext, id, ACTION_FIRE, cal.timeInMillis, extras
            )
        }

        @JvmStatic
        fun scheduleFollowup(
            context: Context,
            alarmId: Int,
            action: String,
            whenMillis: Long,
            extras: Bundle? = null
        ) {
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
                !alarmManager.canScheduleExactAlarms()
            ) {
                Log.e("Violence", "scheduleFollowup: canScheduleExactAlarms=false")
                return
            }
            val pi = receiverPendingIntent(context, alarmId, action, extras)
            alarmManager.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, whenMillis, pi)
            Log.i(
                "Violence",
                "scheduleFollowup: alarmId=$alarmId action=$action at=$whenMillis"
            )
        }

        @JvmStatic
        fun cancelAlarm(id: Int) {
            val context = instance.applicationContext
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
            for (action in listOf(ACTION_FIRE, ACTION_CONFIRM, ACTION_REFIRE)) {
                val pi = receiverPendingIntent(context, id, action)
                alarmManager.cancel(pi)
            }
            val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            nm.cancel(id)
            nm.cancel(confirmNotifId(id))
            Log.i(
                "Violence",
                "cancelAlarm: cancelled FIRE/CONFIRM/REFIRE and notifications for id=$id"
            )
        }
    }
}

class AlarmReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action ?: MainActivity.ACTION_FIRE
        val alarmId = intent.getIntExtra(MainActivity.EXTRA_ALARM_ID, 0)
        val confirmDelayMs = intent.getLongExtra(
            MainActivity.EXTRA_CONFIRM_DELAY_MS, MainActivity.CONFIRM_DELAY_MS
        )
        val refireDelayMs = intent.getLongExtra(
            MainActivity.EXTRA_REFIRE_DELAY_MS, MainActivity.REFIRE_DELAY_MS
        )
        Log.i(
            "Violence",
            "AlarmReceiver action=$action alarmId=$alarmId confirm=${confirmDelayMs}ms refire=${refireDelayMs}ms"
        )

        when (action) {
            MainActivity.ACTION_FIRE,
            MainActivity.ACTION_REFIRE -> fireAlarm(context, alarmId, confirmDelayMs, refireDelayMs)
            MainActivity.ACTION_CONFIRM -> showConfirmNotification(context, alarmId, intent)
            MainActivity.ACTION_ACK -> acknowledgeAlarm(context, alarmId)
            else -> Log.w("Violence", "AlarmReceiver: unknown action $action")
        }
    }

    private fun fireAlarm(
        context: Context,
        alarmId: Int,
        confirmDelayMs: Long,
        refireDelayMs: Long
    ) {
        val activityIntent = Intent(context, AlarmActivity::class.java).apply {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)
            putExtra(MainActivity.EXTRA_ALARM_ID, alarmId)
            putExtra(MainActivity.EXTRA_CONFIRM_DELAY_MS, confirmDelayMs)
            putExtra(MainActivity.EXTRA_REFIRE_DELAY_MS, refireDelayMs)
        }
        val fullScreenPi = PendingIntent.getActivity(
            context,
            alarmId,
            activityIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val notification = NotificationCompat.Builder(context, MainActivity.ALARM_CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_lock_idle_alarm)
            .setContentTitle("Violence HQ")
            .setContentText("Wake up")
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setCategory(NotificationCompat.CATEGORY_ALARM)
            .setContentIntent(fullScreenPi)
            .setFullScreenIntent(fullScreenPi, true)
            .setAutoCancel(true)
            .build()

        val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N && !nm.areNotificationsEnabled()) {
            Log.e("Violence", "fireAlarm: notifications disabled")
        }
        nm.notify(alarmId, notification)
        Log.i("Violence", "fireAlarm: notification posted alarmId=$alarmId")

        // Belt and braces: setFullScreenIntent is best-effort on some OEMs, so we
        // also try to launch AlarmActivity directly. startActivity is permitted from
        // a BroadcastReceiver that was triggered by setExactAndAllowWhileIdle.
        try {
            context.startActivity(activityIntent)
        } catch (e: Exception) {
            Log.e("Violence", "fireAlarm: startActivity failed: $e")
        }
    }

    private fun showConfirmNotification(context: Context, alarmId: Int, intent: Intent) {
        val ackPi = MainActivity.receiverPendingIntent(
            context, alarmId, MainActivity.ACTION_ACK
        )

        val refireAt = intent.getLongExtra(
            MainActivity.EXTRA_REFIRE_AT_MS,
            System.currentTimeMillis() + (MainActivity.REFIRE_DELAY_MS - MainActivity.CONFIRM_DELAY_MS)
        )
        val notification = NotificationCompat.Builder(context, MainActivity.CONFIRM_CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_lock_idle_alarm)
            .setContentTitle("Still up?")
            .setContentText("Confirm before the countdown ends or the alarm fires again.")
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setCategory(NotificationCompat.CATEGORY_REMINDER)
            .setWhen(refireAt)
            .setShowWhen(true)
            .setUsesChronometer(true)
            .setChronometerCountDown(true)
            .setContentIntent(ackPi)
            .addAction(android.R.drawable.ic_menu_send, "Yes, I'm up", ackPi)
            .setAutoCancel(true)
            .build()

        val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        nm.notify(MainActivity.confirmNotifId(alarmId), notification)
        Log.i("Violence", "showConfirmNotification: posted alarmId=$alarmId")
    }

    private fun acknowledgeAlarm(context: Context, alarmId: Int) {
        val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
        val refirePi = MainActivity.receiverPendingIntent(
            context, alarmId, MainActivity.ACTION_REFIRE
        )
        alarmManager.cancel(refirePi)

        val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        nm.cancel(MainActivity.confirmNotifId(alarmId))
        nm.cancel(alarmId)
        Log.i(
            "Violence",
            "acknowledgeAlarm: refire cancelled, notifications dismissed alarmId=$alarmId"
        )
    }
}

class AlarmActivity : Activity() {

    private var ringtone: Ringtone? = null
    private var vibrator: Vibrator? = null
    private var alarmId: Int = 0
    private var confirmDelayMs: Long = MainActivity.CONFIRM_DELAY_MS
    private var refireDelayMs: Long = MainActivity.REFIRE_DELAY_MS

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        alarmId = intent.getIntExtra(MainActivity.EXTRA_ALARM_ID, 0)
        confirmDelayMs = intent.getLongExtra(
            MainActivity.EXTRA_CONFIRM_DELAY_MS, MainActivity.CONFIRM_DELAY_MS
        )
        refireDelayMs = intent.getLongExtra(
            MainActivity.EXTRA_REFIRE_DELAY_MS, MainActivity.REFIRE_DELAY_MS
        )
        Log.i(
            "Violence",
            "AlarmActivity.onCreate alarmId=$alarmId confirm=${confirmDelayMs}ms refire=${refireDelayMs}ms"
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O_MR1) {
            setShowWhenLocked(true)
            setTurnScreenOn(true)
            (getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager)
                .requestDismissKeyguard(this, null)
        } else {
            window.addFlags(
                WindowManager.LayoutParams.FLAG_SHOW_WHEN_LOCKED or
                WindowManager.LayoutParams.FLAG_TURN_SCREEN_ON or
                WindowManager.LayoutParams.FLAG_DISMISS_KEYGUARD
            )
        }
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        setContentView(buildLayout())

        (getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager)
            .cancel(alarmId)

        startRingtone()
        startVibration()
    }

    private fun buildLayout(): LinearLayout {
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(Color.BLACK)
            gravity = Gravity.CENTER
            setPadding(48, 48, 48, 48)
        }

        val timeText = TextView(this).apply {
            text = SimpleDateFormat("HH:mm", Locale.getDefault()).format(Date())
            setTextColor(Color.parseColor("#E5E5E5"))
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 96f)
            gravity = Gravity.CENTER
            typeface = android.graphics.Typeface.MONOSPACE
        }

        val titleText = TextView(this).apply {
            text = "WAKE UP"
            setTextColor(Color.parseColor("#7A7A7A"))
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 32f)
            gravity = Gravity.CENTER
            setPadding(0, 48, 0, 96)
            typeface = android.graphics.Typeface.MONOSPACE
        }

        val stopButton = Button(this).apply {
            text = "STOP"
            setTextColor(Color.WHITE)
            background = android.graphics.drawable.GradientDrawable().apply {
                setColor(Color.TRANSPARENT)
                setStroke(2, Color.parseColor("#E5E5E5"))
            }
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 36f)
            setPadding(96, 48, 96, 48)
            typeface = android.graphics.Typeface.MONOSPACE
            setOnClickListener { onStopPressed() }
        }

        root.addView(timeText)
        root.addView(titleText)
        root.addView(stopButton)
        return root
    }

    @Deprecated("back disabled by design")
    override fun onBackPressed() {
        // no-op — the only way out is the STOP button
    }

    private fun startRingtone() {
        try {
            val uri = RingtoneManager.getDefaultUri(RingtoneManager.TYPE_ALARM)
                ?: RingtoneManager.getDefaultUri(RingtoneManager.TYPE_NOTIFICATION)
            ringtone = RingtoneManager.getRingtone(this, uri).apply {
                audioAttributes = AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_ALARM)
                    .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                    .build()
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                    isLooping = true
                }
                play()
            }
        } catch (e: Exception) {
            Log.e("Violence", "AlarmActivity: ringtone failed: $e")
        }
    }

    private fun startVibration() {
        try {
            vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                (getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager).defaultVibrator
            } else {
                @Suppress("DEPRECATION")
                getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
            }
            val pattern = longArrayOf(0, 1000, 500)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                vibrator?.vibrate(VibrationEffect.createWaveform(pattern, 0))
            } else {
                @Suppress("DEPRECATION")
                vibrator?.vibrate(pattern, 0)
            }
        } catch (e: Exception) {
            Log.e("Violence", "AlarmActivity: vibration failed: $e")
        }
    }

    private fun stopRingtoneAndVibration() {
        try { ringtone?.stop() } catch (_: Exception) {}
        ringtone = null
        try { vibrator?.cancel() } catch (_: Exception) {}
        vibrator = null
    }

    private fun onStopPressed() {
        Log.i("Violence", "AlarmActivity: stop pressed alarmId=$alarmId")
        stopRingtoneAndVibration()
        (getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager)
            .cancel(alarmId)

        val now = System.currentTimeMillis()
        val refireAt = now + refireDelayMs
        val confirmExtras = Bundle().apply {
            putLong(MainActivity.EXTRA_REFIRE_AT_MS, refireAt)
        }
        val refireExtras = Bundle().apply {
            putLong(MainActivity.EXTRA_CONFIRM_DELAY_MS, confirmDelayMs)
            putLong(MainActivity.EXTRA_REFIRE_DELAY_MS, refireDelayMs)
        }
        MainActivity.scheduleFollowup(
            applicationContext, alarmId, MainActivity.ACTION_CONFIRM,
            now + confirmDelayMs, confirmExtras
        )
        MainActivity.scheduleFollowup(
            applicationContext, alarmId, MainActivity.ACTION_REFIRE,
            refireAt, refireExtras
        )

        finish()
    }

    override fun onDestroy() {
        super.onDestroy()
        stopRingtoneAndVibration()
    }
}
