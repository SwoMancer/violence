package dev.dioxus.main

// 1. Import the REAL config using your App ID
import com.example.violence.BuildConfig as AppBuildConfig

// 2. The "Bridge": This makes BuildConfig visible to Logger.kt

import android.os.Bundle
import android.content.Intent
import android.content.Context
import android.content.BroadcastReceiver
import android.app.AlarmManager
import android.app.PendingIntent
import android.os.Build
import android.view.WindowManager

typealias BuildConfig = AppBuildConfig

class MainActivity : WryActivity() {
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        instance = this
        
        // Unlock Screen Logic
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

            alarmManager.setExactAndAllowWhileIdle(
                AlarmManager.RTC_WAKEUP,
                timestamp,
                pendingIntent
            )
        }
    }
}

class AlarmReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val i = Intent(context, MainActivity::class.java)
        i.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        i.putExtra("ALARM_FIRED", true) 
        context.startActivity(i)
    }
}
