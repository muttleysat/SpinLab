package com.spinlab.dj.watch

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.os.PowerManager
import android.os.SystemClock

/**
 * Stato e motore dell'orologio, indipendenti dallo schermo: collegamento col telefono e metronomo a vibrazione vivono qui,
 * cosi' continuano anche con lo schermo spento (li tiene in vita PulseService). L'activity si limita a mostrarli.
 */
object Engine {
    @Volatile var running = false
        private set
    /** Quante schermate sono aperte (0 = schermo spento o app chiusa). */
    @Volatile var screens = 0
    @Volatile var lastPlayingMs = 0L
        private set

    lateinit var pulse: Pulse
        private set
    lateinit var link: Link
        private set

    @Volatile var linked = false
    @Volatile var rttMs = -1.0
    @Volatile var ackMs = -1L
    @Volatile var stems = floatArrayOf(1f, 1f, 1f, 1f)
    /** Chiamato (sul thread principale) a ogni stato ricevuto: la vista controlla se gli stem sono rientrati. */
    @Volatile var onStateHook: (() -> Unit)? = null

    // modalita' demo (solo per fotografare la schermata): tempo finto, senza vibrare e senza ascoltare il telefono
    @Volatile var demo = false
        private set
    private var demoUntil = 0L

    private var oneWayNs = 0L
    private val main = Handler(Looper.getMainLooper())
    private var wake: PowerManager.WakeLock? = null
    private lateinit var appCtx: Context

    private val tick = object : Runnable {
        override fun run() {
            if (!running) return
            if (demo) {
                if (SystemClock.elapsedRealtime() < demoUntil) pulse.sync(pulse.posAbs() % 4.0, 128.6, true, 0) else demo = false
            }
            pulse.expireIfStale()
            setAwake(pulse.playing)
            main.postDelayed(this, 500)
        }
    }

    fun start(ctx: Context) {
        if (running) return
        appCtx = ctx.applicationContext
        val prefs = appCtx.getSharedPreferences("spinlab", Context.MODE_PRIVATE)
        pulse = Pulse(appCtx).also { it.leadMs = prefs.getInt("lead", 30) }
        link = Link(
            appCtx,
            onState = { s ->
                if (!demo) {
                    pulse.sync(s.bpos, s.bpm, s.playing, oneWayNs)
                    stems = s.stems
                    if (s.playing) lastPlayingMs = SystemClock.elapsedRealtime()
                    onStateHook?.invoke()
                }
            },
            onRtt = { ms -> if (!demo) rttMs = ms; oneWayNs = (ms * 1e6 / 2).toLong() },
            onAck = { ms -> ackMs = ms },
            onLinked = { ok -> if (!demo) linked = ok },
        )
        running = true
        lastPlayingMs = SystemClock.elapsedRealtime()
        link.start()
        main.post(tick)
    }

    fun stop() {
        if (!running) return
        running = false
        main.removeCallbacks(tick)
        link.stop()
        pulse.stop()
        setAwake(false)
    }

    fun startDemo() {
        demo = true
        demoUntil = SystemClock.elapsedRealtime() + 14_000
        pulse.enabled = false
        pulse.sync(0.0, 128.6, true, 0)
        linked = true; rttMs = 42.0; ackMs = 61
    }

    fun setLead(ms: Int) {
        pulse.leadMs = ms.coerceIn(0, 200)
        appCtx.getSharedPreferences("spinlab", Context.MODE_PRIVATE).edit().putInt("lead", pulse.leadMs).apply()
    }

    /** CPU sveglia solo mentre c'e' musica: con lo schermo spento senza questo i timer si fermano e la vibrazione salta. */
    private fun setAwake(on: Boolean) {
        if (on) {
            if (wake == null) {
                val pm = appCtx.getSystemService(Context.POWER_SERVICE) as PowerManager
                wake = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "spinlab:pulse")
            }
            if (wake?.isHeld != true) wake?.acquire(10 * 60 * 1000L)
        } else if (wake?.isHeld == true) wake?.release()
    }
}
