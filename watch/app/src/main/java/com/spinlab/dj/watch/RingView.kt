package com.spinlab.dj.watch

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.Typeface
import android.os.SystemClock
import android.view.MotionEvent
import android.view.View
import kotlin.math.exp
import kotlin.math.floor
import kotlin.math.hypot
import kotlin.math.min

/**
 * Schermata unica, stesso linguaggio del video di presentazione:
 * anello a 16 tacche (4 battiti x 4), disco centrale che pulsa a tempo e conta le battute mentre si tiene premuto,
 * quattro pillole per gli stem, lampo bianco al drop. Toccando in alto si apre la taratura (misure e anticipo).
 */
class RingView(ctx: Context, private val pulse: Pulse) : View(ctx) {
    var onHold: ((Boolean) -> Unit)? = null
    var onLead: ((Int) -> Unit)? = null

    // stato che vive in Engine (collegamento e metronomo continuano anche con lo schermo spento)
    private val linked get() = Engine.linked
    private val rttMs get() = Engine.rttMs
    private val ackMs get() = Engine.ackMs
    private val stems get() = Engine.stems
    private val leadMs get() = pulse.leadMs
    var held = false
        private set

    private var tune = false
    /** Fra il rilascio e il rientro vero degli stem sul telefono (che avviene sulla battuta). */
    var armed = false
        private set
    private var armedAt = 0L
    private var pressPos = 0.0
    private var dropAt = 0L
    private var mode = 0f   // 0 blu, 1 arancio: si anima fra i due

    private val BLUE = 0xFF18B6FF.toInt()
    private val ORANGE = 0xFFFF7A1A.toInt()
    private val BG = 0xFF04060B.toInt()
    private val DIM = 0xFF9AA6C2.toInt()
    private val chipColor = intArrayOf(0xFFFF3B4E.toInt(), 0xFFFFC21A.toInt(), 0xFF35E08A.toInt(), 0xFF18B6FF.toInt())
    private val chipName = arrayOf("BATTERIA", "BASSO", "ALTRO", "VOCE")

    private val fill = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL }
    private val line = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeCap = Paint.Cap.ROUND }
    private val txt = Paint(Paint.ANTI_ALIAS_FLAG).apply { textAlign = Paint.Align.CENTER; typeface = Typeface.create("sans-serif-condensed", Typeface.BOLD) }
    private val glowBlue = Paint(Paint.ANTI_ALIAS_FLAG)
    private val glowOrange = Paint(Paint.ANTI_ALIAS_FLAG)
    private val rect = RectF()

    override fun onSizeChanged(w: Int, h: Int, ow: Int, oh: Int) {
        val cx = w / 2f
        val cy = h / 2f
        val r = min(cx, cy)
        glowBlue.shader = RadialGradient(cx, cy, r, intArrayOf(withAlpha(BLUE, 70), withAlpha(BLUE, 0)), floatArrayOf(0.25f, 1f), Shader.TileMode.CLAMP)
        glowOrange.shader = RadialGradient(cx, cy, r, intArrayOf(withAlpha(ORANGE, 70), withAlpha(ORANGE, 0)), floatArrayOf(0.25f, 1f), Shader.TileMode.CLAMP)
    }

    /** Chiamata a ogni stato dal telefono: se dopo il rilascio gli stem sono tornati tutti, e' il drop. */
    fun checkRestored() { if (armed && stems.all { it > 0.01f }) stemsRestored() }

    /** Gli stem sono rientrati sul telefono: lampo del drop. */
    fun stemsRestored() {
        if (!armed) return
        armed = false
        dropAt = SystemClock.uptimeMillis()
        pulse.tick()
    }

    private fun withAlpha(c: Int, a: Int) = (c and 0x00FFFFFF) or (a.coerceIn(0, 255) shl 24)
    private fun lerp(a: Int, b: Int, t: Float): Int {
        val k = t.coerceIn(0f, 1f)
        return Color.rgb(
            (Color.red(a) + (Color.red(b) - Color.red(a)) * k).toInt(),
            (Color.green(a) + (Color.green(b) - Color.green(a)) * k).toInt(),
            (Color.blue(a) + (Color.blue(b) - Color.blue(a)) * k).toInt(),
        )
    }

    private fun label(c: Canvas, s: String, x: Float, y: Float, size: Float, color: Int, alpha: Int = 255) {
        txt.textSize = size
        txt.color = color
        txt.alpha = alpha
        c.drawText(s, x, y + size * 0.35f, txt)
    }

    private fun drawMain(c: Canvas) {
        val cx = width / 2f
        val cy = height / 2f
        val r = min(cx, cy)
        val now = SystemClock.uptimeMillis()
        val pos = pulse.posAbs()
        val playing = pulse.playing
        val beat = floor(pos).toLong()
        val frac = (pos - beat).toFloat()
        val bib = Math.floorMod(beat, 4L).toInt()
        val env = if (playing) exp(-4.5f * frac) else 0f
        if (armed && now - armedAt > 4500) armed = false   // sicurezza: se il rientro non arriva, non restare in attesa
        mode += ((if (held || armed) 1f else 0f) - mode) * 0.22f
        val dropT = ((now - dropAt) / 450f).let { if (it in 0f..1f) 1f - it else 0f }
        val col = lerp(BLUE, ORANGE, mode)

        c.drawColor(BG)
        // alone di sfondo, che respira a tempo
        glowBlue.alpha = ((1f - mode) * (150 + 105 * env)).toInt()
        glowOrange.alpha = (mode * (150 + 105 * env)).toInt()
        c.drawRect(0f, 0f, width.toFloat(), height.toFloat(), glowBlue)
        c.drawRect(0f, 0f, width.toFloat(), height.toFloat(), glowOrange)

        // anello: 16 tacche; si accende quella corrente, restano accese le precedenti della battuta
        val cur = bib * 4 + (frac * 4f).toInt().coerceIn(0, 3)
        val ringR = r * 0.88f
        rect.set(cx - ringR, cy - ringR, cx + ringR, cy + ringR)
        for (i in 0 until 16) {
            val beatStart = i % 4 == 0
            val a0 = -90f + i * 22.5f + (if (beatStart) 3.2f else 1.4f)
            val sweep = 22.5f - (if (beatStart) 4.6f else 2.8f)
            val lit = playing && i == cur
            val past = playing && i < cur
            val base = lerp(col, Color.WHITE, dropT)
            line.strokeWidth = r * (if (beatStart) 0.078f else 0.058f)
            if (lit) {
                line.color = withAlpha(base, (70 + 60 * env).toInt())
                line.strokeWidth *= 2.1f
                c.drawArc(rect, a0, sweep, false, line)
                line.strokeWidth /= 2.1f
            }
            line.color = base
            line.alpha = when {
                dropT > 0f -> 255
                lit -> (170 + 85 * env).toInt()
                past -> 120
                else -> 34
            }
            c.drawArc(rect, a0, sweep, false, line)
        }

        // onde che si allargano dal disco a ogni battito
        if (playing) {
            for (k in 0..1) {
                val t = (frac * 1.6f - k * 0.22f)
                if (t in 0f..1f) {
                    line.strokeWidth = r * 0.014f
                    line.color = withAlpha(col, (170 * (1 - t) * (1 - t) * (if (bib == 0) 1f else 0.6f)).toInt())
                    c.drawCircle(cx, cy, r * (0.34f + 0.40f * t), line)
                }
            }
        }

        // disco centrale
        val discR = r * 0.32f * (1f + 0.045f * env)
        fill.color = withAlpha(col, if (held) 235 else (46 + 50 * env).toInt())
        c.drawCircle(cx, cy, discR, fill)
        line.strokeWidth = r * 0.014f
        line.color = withAlpha(col, if (held) 255 else 190)
        c.drawCircle(cx, cy, discR, line)
        when {
            dropT > 0f -> label(c, "DROP", cx, cy, r * 0.17f, Color.WHITE)
            !linked -> {
                label(c, "NESSUN", cx, cy - r * 0.07f, r * 0.092f, 0xFFFF6B7A.toInt())
                label(c, "TELEFONO", cx, cy + r * 0.06f, r * 0.092f, 0xFFFF6B7A.toInt())
            }
            held -> {
                val n = floor((pos - pressPos) / 4.0).toInt() + 1
                label(c, "BATTUTA", cx, cy - r * 0.14f, r * 0.068f, Color.WHITE, 210)
                label(c, n.toString(), cx, cy + r * 0.04f, r * 0.21f, Color.WHITE)
            }
            armed -> label(c, "DROP", cx, cy, r * 0.15f, Color.WHITE, (150 + 105 * env).toInt())
            !playing -> {
                label(c, "IN ATTESA", cx, cy - r * 0.07f, r * 0.082f, DIM)
                label(c, "DEL BRANO", cx, cy + r * 0.06f, r * 0.082f, DIM)
            }
            else -> {
                label(c, "TIENI", cx, cy - r * 0.07f, r * 0.105f, Color.WHITE)
                label(c, "PREMUTO", cx, cy + r * 0.065f, r * 0.105f, Color.WHITE)
            }
        }

        // pillole degli stem: due sopra (batteria, basso), due sotto (altro, voce)
        val pw = r * 0.52f
        val ph = r * 0.15f
        for (i in 0..3) {
            val px = cx + (if (i % 2 == 0) -1 else 1) * (pw / 2 + r * 0.02f)
            val py = cy + (if (i < 2) -1 else 1) * r * 0.51f
            val on = stems[i] > 0.01f
            rect.set(px - pw / 2, py - ph / 2, px + pw / 2, py + ph / 2)
            fill.color = if (on) chipColor[i] else 0xFF141B29.toInt()
            c.drawRoundRect(rect, ph / 2, ph / 2, fill)
            label(c, chipName[i], px, py, r * 0.078f, if (on) 0xFF05070C.toInt() else 0xFF5E6A86.toInt())
            if (!on) {
                line.strokeWidth = r * 0.012f
                line.color = 0xFF5E6A86.toInt()
                line.alpha = 255
                c.drawLine(px - pw * 0.36f, py, px + pw * 0.36f, py, line)
            }
        }

        // in alto: stato e tempo (toccare qui apre la taratura)
        val topY = cy - r * 0.715f
        fill.color = if (linked) 0xFF35E08A.toInt() else 0xFFFF3B4E.toInt()
        c.drawCircle(cx - r * 0.25f, topY, r * 0.025f, fill)
        label(c, if (linked && pulse.bpm > 0) "%.1f BPM".format(pulse.bpm) else if (linked) "COLLEGATO" else "NON COLLEGATO", cx + r * 0.04f, topY, r * 0.082f, Color.WHITE, 235)

        if (tune) {
            label(c, "RTT %.0f   CMD %s".format(if (rttMs >= 0) rttMs else 0.0, if (ackMs >= 0) "$ackMs" else "-"), cx, cy + r * 0.665f, r * 0.068f, DIM)
            label(c, "ANTICIPO $leadMs ms", cx, cy + r * 0.745f, r * 0.072f, Color.WHITE)
            for (s in intArrayOf(-1, 1)) {
                val bx = cx + s * r * 0.60f
                line.strokeWidth = r * 0.012f
                line.color = withAlpha(Color.WHITE, 150)
                c.drawCircle(bx, cy, r * 0.115f, line)
                label(c, if (s < 0) "−" else "+", bx, cy, r * 0.15f, Color.WHITE)
            }
        }

        postInvalidateOnAnimation()
    }

    // ------------------------------------------------------------ logo animato all'avvio (come lo splash del programma)
    private var splash = true
    private var splashT0 = 0L
    private val SPLASH_MS = 2350f
    private val gloss = Paint(Paint.ANTI_ALIAS_FLAG)

    private fun seg(t: Float, a: Float, b: Float) = ((t - a) / (b - a)).coerceIn(0f, 1f)
    private fun easeOut(t: Float) = 1f - (1f - t) * (1f - t) * (1f - t)
    private fun back(t: Float): Float { val u = t - 1f; return 1f + 2.70158f * u * u * u + 1.70158f * u * u }

    override fun onDraw(c: Canvas) {
        drawMain(c)
        if (splash) drawSplash(c, SystemClock.uptimeMillis())
    }

    private fun drawSplash(c: Canvas, now: Long) {
        if (splashT0 == 0L) splashT0 = now
        val t = (now - splashT0).toFloat()
        if (t >= SPLASH_MS) { splash = false; return }
        val cx = width / 2f
        val cy = height / 2f
        val r = min(cx, cy)
        val out = 1f - seg(t, 1950f, SPLASH_MS)                   // dissolvenza verso l'interfaccia
        val appear = easeOut(seg(t, 0f, 450f))
        val a = { v: Float -> (v * out * appear).toInt().coerceIn(0, 255) }
        fill.color = withAlpha(BG, (255 * out).toInt())
        c.drawRect(0f, 0f, width.toFloat(), height.toFloat(), fill)

        // dopo 1,1 s il logo si rimpicciolisce e sale, per far posto alla scritta
        val shrink = easeOut(seg(t, 1100f, 1800f))
        val ccy = cy - r * 0.13f * shrink
        val ringR = r * 0.80f * (1f - 0.38f * shrink) * (0.9f + 0.1f * appear)
        val vinylR = ringR * 0.93f
        val rot = 260f * (1f - exp(-t / 900f))                    // gira e rallenta

        // vinile: fondo, solchi, riflessi
        fill.color = withAlpha(0xFF0B0F17.toInt(), a(255f))
        c.drawCircle(cx, ccy, vinylR, fill)
        line.strokeWidth = r * 0.006f
        line.color = withAlpha(0xFF1D2636.toInt(), a(255f))
        var gr = vinylR * 0.42f
        while (gr < vinylR) { c.drawCircle(cx, ccy, gr, line); gr += r * 0.021f * (ringR / (r * 0.8f)) }
        gloss.shader = android.graphics.SweepGradient(
            0f, 0f,
            intArrayOf(0x00FFFFFF, 0x00FFFFFF, 0x22FFFFFF, 0x00FFFFFF, 0x00FFFFFF, 0x1AFFFFFF, 0x00FFFFFF, 0x00FFFFFF),
            floatArrayOf(0f, 0.06f, 0.11f, 0.18f, 0.5f, 0.61f, 0.69f, 1f),
        )
        gloss.alpha = a(255f)
        c.save(); c.translate(cx, ccy); c.rotate(rot - 70f); c.drawCircle(0f, 0f, vinylR, gloss); c.restore()

        // anello: arco azzurro e arco arancio che si disegnano
        val p = easeOut(seg(t, 100f, 900f))
        rect.set(cx - ringR, ccy - ringR, cx + ringR, ccy + ringR)
        val orangeCol = 0xFFFFA51F.toInt()
        for (k in 0..1) {
            val col = if (k == 0) BLUE else orangeCol
            val start = if (k == 0) 105f else -67.8f
            val sweep = (if (k == 0) 86.4f else 79.2f) * p
            line.strokeWidth = r * 0.075f
            line.color = withAlpha(col, a(70f))
            c.drawArc(rect, start, sweep, false, line)
            line.strokeWidth = r * 0.03f
            line.color = withAlpha(col, a(255f))
            c.drawArc(rect, start, sweep, false, line)
        }

        // etichetta con il marchio, che entra con un piccolo rimbalzo e gira al contrario
        val lp = back(seg(t, 250f, 750f))
        val labelR = vinylR * 0.38f * lp
        fill.color = withAlpha(0xFF0C1626.toInt(), a(255f))
        c.drawCircle(cx, ccy, labelR, fill)
        line.strokeWidth = r * 0.012f
        line.color = withAlpha(BLUE, a(200f))
        c.drawCircle(cx, ccy, labelR, line)
        Logo.draw(c, cx, ccy, labelR * 2f * 0.86f, -rot, a(255f))

        // scritta: «Spin» bianco e «Lab» arancio
        val w = easeOut(seg(t, 1150f, 1750f))
        if (w > 0f) {
            txt.textSize = r * 0.19f
            txt.textAlign = Paint.Align.LEFT
            val ws = txt.measureText("Spin")
            val wl = txt.measureText("Lab")
            val x0 = cx - (ws + wl) / 2f
            val y = cy + r * 0.52f + r * 0.19f * 0.35f
            txt.color = Color.WHITE; txt.alpha = a(255f * w)
            c.drawText("Spin", x0, y, txt)
            txt.color = orangeCol; txt.alpha = a(255f * w)
            c.drawText("Lab", x0 + ws, y, txt)
            txt.textAlign = Paint.Align.CENTER
        }
    }

    override fun onTouchEvent(e: MotionEvent): Boolean {
        val cx = width / 2f
        val cy = height / 2f
        val r = min(cx, cy)
        if (splash) { if (e.actionMasked == MotionEvent.ACTION_DOWN) splash = false; return true }
        when (e.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                if (hypot(e.x - cx, e.y - cy) < r * 0.40f) {
                    held = true
                    pressPos = pulse.posAbs()
                    pulse.tick()
                    onHold?.invoke(true)
                } else if (tune && hypot(e.x - (cx - r * 0.60f), e.y - cy) < r * 0.18f) {
                    onLead?.invoke(-5)
                } else if (tune && hypot(e.x - (cx + r * 0.60f), e.y - cy) < r * 0.18f) {
                    onLead?.invoke(5)
                } else if (e.y < cy - r * 0.55f) {
                    tune = !tune
                }
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> if (held) {
                held = false
                armed = true
                armedAt = SystemClock.uptimeMillis()
                onHold?.invoke(false)
            }
        }
        return true
    }
}
