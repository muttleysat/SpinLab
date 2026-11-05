package com.spinlab.dj.watch

import android.content.Context
import android.os.Handler
import android.os.HandlerThread
import android.os.Process
import android.os.SystemClock
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.roundToLong

/**
 * Metronomo a vibrazione. Il telefono manda ogni 250 ms tempo e posizione nella battuta (0..4);
 * qui un orologio proprio fa girare la posizione fra un messaggio e l'altro e vibra a ogni battito.
 * Cosi' un messaggio in ritardo o perso non sposta la vibrazione: al massimo la corregge piano.
 */
class Pulse(ctx: Context) {
    private val vib: Vibrator = ctx.getSystemService(VibratorManager::class.java).defaultVibrator
    private val thread = HandlerThread("pulse", Process.THREAD_PRIORITY_URGENT_AUDIO).also { it.start() }
    private val h = Handler(thread.looper)

    @Volatile var bpm = 0.0
        private set
    @Volatile var playing = false
        private set
    @Volatile var enabled = true
    /** Anticipo dell'impulso in ms: il motorino parte con un po' di ritardo, qui lo si compensa a mano. */
    @Volatile var leadMs = 30

    @Volatile private var anchorNs = 0L
    @Volatile private var anchorPos = 0.0        // battiti assoluti all'istante anchorNs
    @Volatile private var lastBeat = Long.MIN_VALUE
    @Volatile private var lastSyncNs = 0L
    /** Quanto la posizione locale e' stata corretta all'ultimo messaggio (ms): serve a capire se il tempo e' stabile. */
    @Volatile var lastCorrectionMs = 0.0
        private set

    /** Posizione in battiti assoluti (cresce sempre); il battito nella battuta e' posAbs mod 4. */
    fun posAbs(): Double {
        if (!playing || bpm <= 0) return anchorPos
        return anchorPos + (SystemClock.elapsedRealtimeNanos() - anchorNs) / 1e9 * bpm / 60.0
    }

    /** @param bpos posizione nella battuta (0..4) al momento dell'invio; @param oneWayNs meta' del ritardo andata-ritorno */
    fun sync(bpos: Double, newBpm: Double, isPlaying: Boolean, oneWayNs: Long) {
        val now = SystemClock.elapsedRealtimeNanos()
        lastSyncNs = now
        val local = posAbs()
        val wasPlaying = playing
        val bpmChanged = abs(newBpm - bpm) > 0.05
        bpm = newBpm
        playing = isPlaying && newBpm > 20
        if (!playing) {
            anchorNs = now; anchorPos = local
            h.removeCallbacksAndMessages(null)
            return
        }
        // posizione del telefono ora = quella inviata + il tempo di viaggio
        val remote = bpos + oneWayNs / 1e9 * newBpm / 60.0
        // numero di battute intero piu' vicino alla posizione locale, cosi' la posizione resta continua
        val cand = floor((local - remote) / 4.0 + 0.5) * 4.0 + remote
        val diff = cand - local
        val diffMs = diff * 60000.0 / newBpm
        lastCorrectionMs = diffMs
        anchorNs = now
        anchorPos = if (!wasPlaying || abs(diffMs) > 120) cand else local + diff * 0.25
        if (!wasPlaying || bpmChanged || abs(diffMs) > 120) {
            lastBeat = floor(anchorPos).roundToLong()
            schedule()
        } else if (!h.hasMessages(0)) schedule()
    }

    private fun schedule() {
        h.removeCallbacksAndMessages(null)
        if (!playing || !enabled || bpm <= 0) return
        val pos = posAbs()
        val beatMs = 60000.0 / bpm
        // primo battito il cui istante di vibrazione (battito - anticipo) e' ancora futuro
        val n = maxOf(lastBeat + 1, ceil(pos + (leadMs - 4) / beatMs).roundToLong())
        val delay = ((n - pos) * beatMs - leadMs).coerceAtLeast(0.0).roundToLong()
        h.postDelayed({
            if (!playing || !enabled) return@postDelayed
            val strong = Math.floorMod(n, 4L) == 0L
            vib.vibrate(
                if (strong) VibrationEffect.createOneShot(38, 255) else VibrationEffect.createOneShot(22, 150)
            )
            lastBeat = n
            schedule()
        }, delay)
    }

    /** Se il telefono non manda piu' lo stato (app in secondo piano, Bluetooth caduto) ci si ferma: niente vibrazione "fantasma". */
    fun expireIfStale(maxNs: Long = 1_500_000_000L) {
        val now = SystemClock.elapsedRealtimeNanos()
        if (playing && now - lastSyncNs > maxNs) {
            anchorPos = posAbs()
            anchorNs = now
            playing = false
            h.removeCallbacksAndMessages(null)
        }
    }

    fun stop() {
        h.removeCallbacksAndMessages(null)
        thread.quitSafely()
    }

    fun tick() {
        // piccolo impulso di conferma al tocco
        vib.vibrate(VibrationEffect.createOneShot(14, 200))
    }
}
