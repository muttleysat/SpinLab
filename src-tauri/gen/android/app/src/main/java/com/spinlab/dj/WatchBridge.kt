package com.spinlab.dj

import android.content.Context
import android.os.SystemClock
import android.webkit.JavascriptInterface
import android.webkit.WebView
import com.google.android.gms.wearable.MessageEvent
import com.google.android.gms.wearable.Wearable
import com.google.android.gms.wearable.WearableListenerService
import org.json.JSONObject

/**
 * Ponte con l'orologio Wear OS (Data Layer di Google). Stato e comandi passano dall'interfaccia (ui/watch.js),
 * cosi' un comando dall'orologio fa le stesse cose di un tocco sullo schermo e lo schermo si aggiorna.
 *   telefono -> orologio: /spinlab/state (JSON), /spinlab/pong, /spinlab/ack
 *   orologio -> telefono: /spinlab/ping, /spinlab/stems (JSON)
 */
object WatchBridge {
    @Volatile var webView: WebView? = null
    @Volatile private var nodes: List<String> = emptyList()
    private var nodesAt = 0L

    /** Esegue codice nella pagina, se l'app e' aperta (altrimenti il comando si perde: l'orologio ha bisogno dell'app in primo piano). */
    fun toJs(js: String) {
        val w = webView ?: return
        w.post { w.evaluateJavascript(js, null) }
    }

    fun send(ctx: Context, path: String, data: ByteArray) {
        val now = SystemClock.elapsedRealtime()
        if (now - nodesAt > 4000) {
            nodesAt = now
            Wearable.getNodeClient(ctx).connectedNodes.addOnSuccessListener { n -> nodes = n.map { it.id } }
        }
        val c = Wearable.getMessageClient(ctx)
        for (id in nodes) c.sendMessage(id, path, data)
    }
}

/** Funzioni chiamate dalla pagina come SpinlabWatch.state(...) e SpinlabWatch.ack(...). */
class WatchJs(private val ctx: Context) {
    @JavascriptInterface fun state(json: String) = WatchBridge.send(ctx, "/spinlab/state", json.toByteArray())
    @JavascriptInterface fun ack(id: String) = WatchBridge.send(ctx, "/spinlab/ack", id.toByteArray())
}

/** Google Play Services avvia questo servizio quando l'orologio manda un messaggio. */
class WatchService : WearableListenerService() {
    override fun onMessageReceived(e: MessageEvent) {
        when (e.path) {
            // risposta immediata, senza passare dalla pagina: misura il solo collegamento
            "/spinlab/ping" -> Wearable.getMessageClient(this).sendMessage(e.sourceNodeId, "/spinlab/pong", e.data)
            "/spinlab/stems" ->
                WatchBridge.toJs("window.spinlabWatch&&window.spinlabWatch(" + JSONObject.quote(String(e.data)) + ")")
        }
    }
}
