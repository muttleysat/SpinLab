package com.spinlab.dj.watch

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.view.WindowManager
import androidx.core.content.ContextCompat

/**
 * Schermata dell'orologio: mostra lo stato che vive in Engine (collegamento e metronomo, tenuti in vita da PulseService).
 * Tenendo premuto il tasto centrale escono batteria e basso; al rilascio rientrano sulla prossima battuta.
 */
class MainActivity : Activity() {
    private lateinit var view: RingView
    /** Volumi degli stem prima di tenere premuto: al rilascio si torna a quelli, non per forza a tutti accesi. */
    private var preHold = floatArrayOf(1f, 1f, 1f, 1f)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        Engine.start(this)
        // il servizio tiene vivo il collegamento anche con lo schermo spento
        ContextCompat.startForegroundService(this, Intent(this, PulseService::class.java))
        if (Build.VERSION.SDK_INT >= 33 &&
            checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)

        if (intent.getBooleanExtra("demo", false)) Engine.startDemo()

        view = RingView(this, Engine.pulse)
        setContentView(view)
        view.onHold = { down ->
            // in demo non si toccano gli stem del telefono
            if (!Engine.demo) {
                if (down) {
                    // fuori batteria e basso, il resto resta com'e'
                    preHold = Engine.stems.copyOf()
                    val g = preHold.copyOf().also { it[0] = 0f; it[1] = 0f }
                    Engine.stems = g
                    Engine.link.sendStems(g)
                } else {
                    // al rilascio tornano come prima, sulla prossima battuta (lo decide il telefono, che conosce la griglia)
                    Engine.link.sendStems(preHold.copyOf().also { if (it[0] < 0.01f && it[1] < 0.01f) { it[0] = 1f; it[1] = 1f } }, onBar = true)
                }
            }
        }
        view.onToggleStem = { i ->
            // le pillole colorate: un tocco silenzia o riammette quello stem (subito); lo stato vero lo conferma il telefono
            if (!Engine.demo) {
                val g = Engine.stems.copyOf()
                g[i] = if (g[i] > 0.01f) 0f else 1f
                Engine.stems = g
                Engine.link.sendStems(g)
            }
        }
        view.onLead = { d -> Engine.setLead(Engine.pulse.leadMs + d) }
    }

    override fun onResume() {
        super.onResume()
        Engine.screens++
        Engine.onStateHook = { view.checkRestored() }
    }

    override fun onPause() {
        Engine.onStateHook = null
        Engine.screens--
        super.onPause()
    }
}
