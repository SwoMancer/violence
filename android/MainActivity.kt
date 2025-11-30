package dev.dioxus.main

import android.os.Bundle
import android.content.Intent
import android.content.Context
import android.content.BroadcastReceiver
import android.app.AlarmManager
import android.app.PendingIntent
import android.os.Build
import android.view.WindowManager

// 1. THE FIX: Define a dummy BuildConfig so Logger.kt stops complaining.
//    We don't need the real one; we just need to satisfy the "DEBUG" check.
object BuildConfig {
    const val DEBUG = true
}

// 2. The Activity: Handles the UI and bridging with Rust
class MainActivity : WryActivity() {
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        instance = this

        // Unlock Screen Logic (Critical for Alarm Apps)
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

        // Rust calls this function via JNI
        @JvmStatic
        fun scheduleAlarm(timestamp: Long, id: Int) {
            val context = instance.applicationContext
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
            
            val intent = Intent(context, AlarmReceiver::class.java)
            intent.putExtra("ALARM_ID", id)

            val pendingIntent = PendingIntent.getBroadcast(
                context, id, intent, 
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )

            // The "Nuclear" Option: Wakes up device even from Doze mode
            alarmManager.setExactAndAllowWhileIdle(
                AlarmManager.RTC_WAKEUP,
                timestamp,
                pendingIntent
            )
        }
    }
}

// 3. The Receiver: Wakes up the app when the alarm fires
class AlarmReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        // Logic: Launch the App immediately
        val i = Intent(context, MainActivity::class.java)
        i.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        i.putExtra("ALARM_FIRED", true) 
        context.startActivity(i)
    }
}
