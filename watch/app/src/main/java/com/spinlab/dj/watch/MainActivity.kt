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
            // in demo non si toccano gli stem del telefono; al rilascio il telefono li fa rientrare sulla battuta (conosce la griglia)
            if (!Engine.demo) Engine.link.sendStems(
                if (down) floatArrayOf(0f, 0f, 1f, 1f) else floatArrayOf(1f, 1f, 1f, 1f), onBar = !down,
            )
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
