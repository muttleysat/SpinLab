package com.spinlab.dj

import android.Manifest
import android.content.pm.PackageManager
import android.graphics.Color
import android.graphics.drawable.ColorDrawable
import android.os.Build
import android.os.Bundle
import android.view.View
import android.app.Activity
import android.view.WindowManager
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // sfondo scuro dell'interfaccia: le barre di sistema hanno icone chiare qualunque sia il tema del telefono
    enableEdgeToEdge(SystemBarStyle.dark(Color.TRANSPARENT), SystemBarStyle.dark(Color.TRANSPARENT))
    super.onCreate(savedInstanceState)
    window.setBackgroundDrawable(ColorDrawable(0xFF070A10.toInt()))
    // con l'app in primo piano lo schermo non si spegne (durante un set non si tocca il telefono per minuti)
    window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    // Android 15+ disegna sempre sotto le barre di stato e di navigazione: lascia loro lo spazio, altrimenti coprono l'intestazione
    val root = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(root) { v, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
      v.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }
    askAudioPermission()
  }

  // l'orologio parla con la pagina: WatchBridge ne tiene il riferimento, SpinlabWatch e' l'oggetto che la pagina chiama
  override fun onWebViewCreate(webView: WebView) {
    WatchBridge.webView = webView
    webView.addJavascriptInterface(WatchJs(applicationContext), "SpinlabWatch")
    webView.addJavascriptInterface(AppJs(this), "SpinlabApp")
  }

  // la libreria legge i brani dal filesystem: serve il permesso di lettura dell'audio
  private fun askAudioPermission() {
    val perm = if (Build.VERSION.SDK_INT >= 33) Manifest.permission.READ_MEDIA_AUDIO else Manifest.permission.READ_EXTERNAL_STORAGE
    if (ContextCompat.checkSelfPermission(this, perm) != PackageManager.PERMISSION_GRANTED) {
      ActivityCompat.requestPermissions(this, arrayOf(perm), 1)
    }
  }
}

/** Chiusura dell'app richiesta dalla pagina (doppia pressione di Indietro dalla schermata principale). */
class AppJs(private val activity: Activity) {
  @JavascriptInterface fun exit() { activity.runOnUiThread { activity.finishAffinity() } }
}
