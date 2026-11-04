package com.spinlab.dj.watch

import android.graphics.Canvas
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RadialGradient
import android.graphics.Shader

/**
 * Marchio di SpinLab (ui/assets/spinlab-mark.svg): due archi e un punto, in uno spazio 100 x 100.
 * File generato dall'SVG: se il logo cambia, va rigenerato.
 */
object Logo {
    private val CYAN = floatArrayOf(80.3f, 70.4f, 78.4f, 73.2f, 76.2f, 75.8f, 73.8f, 78.3f, 71.3f, 80.5f, 68.5f, 82.5f, 65.6f, 84.3f, 62.6f, 85.8f, 59.4f, 87f, 56.1f, 87.9f, 52.8f, 88.6f, 49.4f, 88.9f, 46f, 89f, 42.6f, 88.7f, 39.2f, 88.2f, 35.9f, 87.4f, 32.7f, 86.3f, 29.6f, 84.9f, 26.6f, 83.3f, 23.8f, 81.4f, 21.2f, 79.2f, 18.7f, 76.8f, 16.5f, 74.3f, 14.5f, 71.5f, 12.7f, 68.6f, 11.2f, 65.6f, 10f, 62.4f, 9.1f, 59.1f, 8.4f, 55.8f, 8.1f, 52.4f, 8f, 49f, 8.3f, 45.6f, 8.8f, 42.2f, 9.6f, 38.9f, 10.7f, 35.7f, 12.1f, 32.6f, 13.7f, 29.6f, 15.6f, 26.8f, 17.8f, 24.2f, 20.2f, 21.7f, 22.7f, 19.5f, 25.5f, 17.5f, 28.4f, 15.7f, 31.4f, 14.2f, 34.6f, 13f, 37.9f, 12.1f, 41.2f, 11.4f, 44.6f, 11.1f, 48f, 11f, 51.4f, 11.3f, 54.8f, 11.8f, 54.7f, 12.3f, 51.9f, 12.8f, 49.1f, 13.5f, 46.4f, 14.5f, 43.8f, 15.7f, 41.3f, 17.2f, 38.9f, 18.8f, 36.7f, 20.7f, 34.7f, 22.7f, 32.8f, 24.9f, 31.2f, 27.3f, 29.7f, 29.8f, 28.5f, 32.4f, 27.5f, 35.1f, 26.8f, 37.9f, 26.3f, 40.7f, 26f, 43.6f, 26f, 46.4f, 26.3f, 49.3f, 26.8f, 52.1f, 27.5f, 54.9f, 28.5f, 57.6f, 29.7f, 60.2f, 31.2f, 62.7f, 32.8f, 65.1f, 34.7f, 67.3f, 36.7f, 69.3f, 38.9f, 71.2f, 41.3f, 72.8f, 43.8f, 74.3f, 46.4f, 75.5f, 49.1f, 76.5f, 51.9f, 77.2f, 54.7f, 77.7f, 57.6f, 78f, 60.4f, 78f, 63.3f, 77.7f, 66.1f, 77.2f, 68.9f, 76.5f, 71.6f, 75.5f, 74.2f, 74.3f, 76.7f, 72.8f, 79.1f, 71.2f)
    private val ORANGE = floatArrayOf(27f, 32.7f, 28.6f, 30.6f, 30.3f, 28.6f, 32.2f, 26.7f, 34.3f, 25f, 36.5f, 23.5f, 38.8f, 22.1f, 41.2f, 20.9f, 43.7f, 20f, 46.2f, 19.2f, 48.8f, 18.6f, 51.4f, 18.2f, 54.1f, 18f, 56.8f, 18f, 59.4f, 18.3f, 62.1f, 18.7f, 64.7f, 19.4f, 67.2f, 20.3f, 69.6f, 21.3f, 72f, 22.6f, 74.3f, 24f, 76.4f, 25.6f, 78.4f, 27.3f, 80.3f, 29.2f, 82f, 31.3f, 83.5f, 33.5f, 84.9f, 35.8f, 86.1f, 38.2f, 87f, 40.7f, 87.8f, 43.2f, 88.4f, 45.8f, 88.8f, 48.4f, 89f, 51.1f, 89f, 53.8f, 88.7f, 56.4f, 88.3f, 59.1f, 87.6f, 61.7f, 86.7f, 64.2f, 85.7f, 66.6f, 84.4f, 69f, 83f, 71.3f, 81.4f, 73.4f, 79.7f, 75.4f, 77.8f, 77.3f, 75.7f, 79f, 73.5f, 80.5f, 71.2f, 81.9f, 68.8f, 83.1f, 66.3f, 84f, 63.8f, 84.8f, 61.2f, 85.4f, 58.6f, 85.8f, 55.9f, 86f, 53.2f, 86f, 50.6f, 85.7f, 49.3f, 85.3f, 51.5f, 84.7f, 53.6f, 84f, 55.7f, 83.1f, 57.7f, 82f, 59.6f, 80.8f, 61.4f, 79.4f, 63.1f, 77.9f, 64.7f, 76.2f, 66.2f, 74.5f, 67.5f, 72.6f, 68.6f, 70.6f, 69.6f, 68.6f, 70.4f, 66.4f, 71.1f, 64.3f, 71.6f, 62f, 71.9f, 59.8f, 72f, 57.5f, 71.9f, 55.2f, 71.7f, 53f, 71.3f, 50.7f, 70.7f, 48.5f, 70f, 46.4f, 69.1f, 44.3f, 68f, 42.3f, 66.8f, 40.4f, 65.4f, 38.6f, 63.9f, 36.9f, 62.2f, 35.3f, 60.5f, 33.8f, 58.6f, 32.5f, 56.6f, 31.4f, 54.6f, 30.4f, 52.4f, 29.6f, 50.3f, 28.9f, 48f, 28.4f, 45.8f, 28.1f, 43.5f, 28f, 41.2f, 28.1f, 39f, 28.3f, 36.7f, 28.7f, 34.5f, 29.3f, 32.4f, 30f, 30.3f, 30.9f, 28.3f, 32f)

    private fun path(a: FloatArray) = Path().apply {
        moveTo(a[0], a[1])
        var i = 2
        while (i < a.size) { lineTo(a[i], a[i + 1]); i += 2 }
        close()
    }

    private val cyan = path(CYAN)
    private val orange = path(ORANGE)
    // sfumature come nell'SVG: coordinate nello spazio del logo, la matrice del canvas le scala
    private val pCyan = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        shader = LinearGradient(8f, 11f, 80.3f, 89f, 0xFF7AE9FF.toInt(), 0xFF0A8FE6.toInt(), Shader.TileMode.CLAMP)
    }
    private val pOrange = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        shader = LinearGradient(89f, 86f, 27f, 18f, 0xFFFF8A1C.toInt(), 0xFFFFC247.toInt(), Shader.TileMode.CLAMP)
    }
    private val pDot = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        shader = RadialGradient(46.7f, 48.05f, 10.4f, 0xFFBFF4FF.toInt(), 0xFF18B6FF.toInt(), Shader.TileMode.CLAMP)
    }

    /** Disegna il marchio di diametro [size] px, centrato in ([cx], [cy]), ruotato di [deg] gradi. */
    fun draw(c: Canvas, cx: Float, cy: Float, size: Float, deg: Float, alpha: Int) {
        val a = alpha.coerceIn(0, 255)
        pCyan.alpha = a; pOrange.alpha = a; pDot.alpha = a
        c.save()
        c.translate(cx, cy)
        c.rotate(deg)
        c.scale(size / 100f, size / 100f)
        c.translate(-50f, -50f)
        c.drawPath(cyan, pCyan)
        c.drawPath(orange, pOrange)
        c.drawCircle(50f, 50f, 6.5f, pDot)
        c.restore()
    }
}
