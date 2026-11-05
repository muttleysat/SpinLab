package com.spinlab.dj.watch

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import com.google.android.gms.wearable.MessageClient
import com.google.android.gms.wearable.MessageEvent
import com.google.android.gms.wearable.Wearable
import org.json.JSONArray
import org.json.JSONObject
import java.nio.ByteBuffer

/**
 * Collegamento con SpinLab sul telefono, tramite il Data Layer di Google (Bluetooth; la nuvola di Google
 * solo se il Bluetooth cade). Percorsi dei messaggi:
 *   orologio -> telefono: /spinlab/ping (8 byte: istante locale), /spinlab/stems (JSON)
 *   telefono -> orologio: /spinlab/pong (lo stesso ping), /spinlab/state (JSON), /spinlab/ack (id del comando)
 */
class Link(
    ctx: Context,
    private val onState: (State) -> Unit,
    private val onRtt: (Double) -> Unit,
    private val onAck: (Long) -> Unit,
    private val onLinked: (Boolean) -> Unit,
) : MessageClient.OnMessageReceivedListener {

    class State(val bpm: Double, val bpos: Double, val playing: Boolean, val stems: FloatArray, val title: String)

    private val appCtx = ctx.applicationContext
    private val msg = Wearable.getMessageClient(appCtx)
    private val nodes = Wearable.getNodeClient(appCtx)
    private val main = Handler(Looper.getMainLooper())
    @Volatile private var ids: List<String> = emptyList()
    private var running = false
    private val rtts = ArrayDeque<Double>()

    private val loop = object : Runnable {
        override fun run() {
            if (!running) return
            nodes.connectedNodes.addOnSuccessListener { n ->
                ids = n.map { it.id }
                onLinked(ids.isNotEmpty())
            }
            send("/spinlab/ping", ByteBuffer.allocate(8).putLong(SystemClock.elapsedRealtimeNanos()).array())
            main.postDelayed(this, 2000)
        }
    }

    fun start() {
        if (running) return
        running = true
        msg.addListener(this)
        loop.run()
    }

    fun stop() {
        running = false
        main.removeCallbacks(loop)
        msg.removeListener(this)
    }

    private fun send(path: String, data: ByteArray) {
        for (id in ids) msg.sendMessage(id, path, data)
    }

    /** Volumi degli stem (batteria, basso, altro, voce) sul deck selezionato del telefono. */
    fun sendStems(gains: FloatArray, onBar: Boolean = false) {
        val j = JSONObject()
            .put("t", "stems")
            .put("id", SystemClock.elapsedRealtime())
            .put("gains", JSONArray(gains.map { it.toDouble() }))
        if (onBar) j.put("q", "bar")   // il telefono applica il cambio alla prossima battuta
        send("/spinlab/stems", j.toString().toByteArray())
    }

    override fun onMessageReceived(e: MessageEvent) {
        val rx = SystemClock.elapsedRealtimeNanos()
        try {
            when (e.path) {
                "/spinlab/state" -> {
                    val j = JSONObject(String(e.data))
                    val a = j.getJSONArray("stems")
                    onState(
                        State(
                            j.optDouble("bpm", 0.0), j.optDouble("bpos", 0.0), j.optBoolean("playing", false),
                            FloatArray(4) { a.optDouble(it, 1.0).toFloat() }, j.optString("title", ""),
                        )
                    )
                }
                "/spinlab/pong" -> {
                    val ms = (rx - ByteBuffer.wrap(e.data).long) / 1e6
                    rtts.addLast(ms)
                    if (rtts.size > 9) rtts.removeFirst()
                    onRtt(rtts.sorted()[rtts.size / 2])   // mediana: i valori isolati altissimi non contano
                }
                "/spinlab/ack" -> onAck(SystemClock.elapsedRealtime() - String(e.data).toLong())
            }
        } catch (_: Exception) {
            // messaggio non valido: si ignora
        }
    }
}
