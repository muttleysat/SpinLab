package com.spinlab.dj.watch

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.os.SystemClock
import androidx.core.app.NotificationCompat
import androidx.wear.ongoing.OngoingActivity
import androidx.wear.ongoing.Status

/**
 * Tiene vivi collegamento e metronomo con lo schermo spento e mostra SpinLab come attivita' in corso
 * (icona sul quadrante e nelle app recenti: un tocco apre la schermata). Si ferma da solo dopo 90 s senza musica.
 */
class PulseService : Service() {
    private val main = Handler(Looper.getMainLooper())
    private val idle = object : Runnable {
        override fun run() {
            val quiet = SystemClock.elapsedRealtime() - Engine.lastPlayingMs
            if (Engine.screens == 0 && quiet > IDLE_MS) { stopSelf(); return }
            main.postDelayed(this, 5000)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        Engine.start(this)
        val n = buildNotification(this)
        if (Build.VERSION.SDK_INT >= 34) startForeground(NOTIF_ID, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
        else startForeground(NOTIF_ID, n)
        main.postDelayed(idle, 5000)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int) = START_NOT_STICKY

    override fun onDestroy() {
        main.removeCallbacks(idle)
        if (Engine.screens == 0) Engine.stop()
        super.onDestroy()
    }

    companion object {
        const val CHANNEL = "spinlab"
        const val NOTIF_ID = 1
        const val IDLE_MS = 90_000L

        fun ensureChannel(ctx: Context) {
            val nm = ctx.getSystemService(NotificationManager::class.java)
            if (nm.getNotificationChannel(CHANNEL) == null) {
                nm.createNotificationChannel(NotificationChannel(CHANNEL, "SpinLab", NotificationManager.IMPORTANCE_LOW))
            }
        }

        fun openIntent(ctx: Context): PendingIntent = PendingIntent.getActivity(
            ctx, 0, Intent(ctx, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )

        fun buildNotification(ctx: Context): Notification {
            ensureChannel(ctx)
            val touch = openIntent(ctx)
            val b = NotificationCompat.Builder(ctx, CHANNEL)
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setContentTitle("SpinLab")
                .setContentText("Vibrazione a tempo attiva")
                .setCategory(NotificationCompat.CATEGORY_SERVICE)
                .setOngoing(true)
                .setContentIntent(touch)
            // attivita' in corso: compare sul quadrante e nelle app recenti
            OngoingActivity.Builder(ctx, NOTIF_ID, b)
                .setStaticIcon(android.R.drawable.ic_media_play)
                .setTouchIntent(touch)
                .setStatus(Status.Builder().addTemplate("SpinLab collegato").build())
                .build()
                .apply(ctx)
            return b.build()
        }
    }
}
