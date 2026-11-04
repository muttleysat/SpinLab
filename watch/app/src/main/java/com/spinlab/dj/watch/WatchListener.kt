package com.spinlab.dj.watch

import android.app.NotificationManager
import android.content.Intent
import android.os.SystemClock
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import com.google.android.gms.wearable.MessageEvent
import com.google.android.gms.wearable.WearableListenerService
import org.json.JSONObject

/**
 * Google Play Services avvia questo servizio quando il telefono manda un messaggio, anche con l'app chiusa.
 * Se arriva lo stato con la musica in corso, prova ad avviare PulseService; se Wear OS non lo permette
 * dallo sfondo, mostra una notifica da toccare per aprire l'app.
 */
class WatchListener : WearableListenerService() {
    override fun onMessageReceived(e: MessageEvent) {
        if (e.path != "/spinlab/state" || Engine.running) return
        val playing = try { JSONObject(String(e.data)).optBoolean("playing", false) } catch (_: Exception) { false }
        if (!playing) return
        try {
            ContextCompat.startForegroundService(this, Intent(this, PulseService::class.java))
        } catch (_: Exception) {
            notifyOpen()
        }
    }

    private fun notifyOpen() {
        val now = SystemClock.elapsedRealtime()
        if (now - lastNotifyMs < 30_000) return
        lastNotifyMs = now
        PulseService.ensureChannel(this)
        val n = NotificationCompat.Builder(this, PulseService.CHANNEL)
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setContentTitle("SpinLab in riproduzione")
            .setContentText("Tocca per aprire")
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setContentIntent(PulseService.openIntent(this))
            .build()
        getSystemService(NotificationManager::class.java).notify(2, n)
    }

    companion object { private var lastNotifyMs = -60_000L }
}
