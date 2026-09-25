param(
    [string] $Dir = "C:\Users\dperham\source\ghostwriter-builds\templates"
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$inPng  = Join-Path $Dir "tpl_lines_medium.png"
$outPng = Join-Path $Dir "P Lines medium coach.png"
$inSvg  = Join-Path $Dir "tpl_lines_medium.svg"
$outSvg = Join-Path $Dir "P Lines medium coach.svg"

# Icon centre: inside the top-right tap zone (68 virtual px = 124 native px square).
$cx = 1342; $cy = 62

# ---- PNG ----
$src = [System.Drawing.Image]::FromFile($inPng)
$bmp = New-Object System.Drawing.Bitmap $src.Width, $src.Height, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.DrawImage($src, 0, 0, $src.Width, $src.Height)

$pen  = New-Object System.Drawing.Pen ([System.Drawing.Color]::Black), 3
$pen.StartCap = 'Round'; $pen.EndCap = 'Round'
$ring = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(255, 70, 70, 70)), 2
$ring.DashStyle = [System.Drawing.Drawing2D.DashStyle]::Dot

# dotted ring = the tap spot
$g.DrawEllipse($ring, $cx - 52, $cy - 50, 104, 104)
# brain body
$g.DrawEllipse($pen, $cx - 32, $cy - 30, 64, 50)
# central fissure
$g.DrawBezier($pen, $cx, $cy - 30, $cx - 9, $cy - 16, $cx + 9, $cy + 4, $cx, $cy + 20)
# folds, left lobe
$g.DrawArc($pen, $cx - 27, $cy - 22, 18, 14, 200, 150)
$g.DrawArc($pen, $cx - 25, $cy - 6, 16, 14, 20, 150)
$g.DrawArc($pen, $cx - 15, $cy - 18, 12, 12, 250, 150)
# folds, right lobe
$g.DrawArc($pen, $cx + 9, $cy - 22, 18, 14, 190, 150)
$g.DrawArc($pen, $cx + 9, $cy - 6, 16, 14, 10, 150)
$g.DrawArc($pen, $cx + 3, $cy - 18, 12, 12, 140, 150)
# stem
$g.DrawLine($pen, $cx + 4, $cy + 18, $cx + 9, $cy + 30)
$g.DrawLine($pen, $cx - 4, $cy + 19, $cx - 1, $cy + 30)
# label
$font = New-Object System.Drawing.Font "Segoe UI", 12, ([System.Drawing.FontStyle]::Bold)
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 70, 70, 70))
$size = $g.MeasureString("tap", $font)
$g.DrawString("tap", $font, $brush, $cx - $size.Width / 2, $cy + 30)

$g.Dispose()
$bmp.Save($outPng, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose(); $src.Dispose()
Write-Host "Wrote $outPng"

# ---- SVG (same icon in vector form) ----
$svg = Get-Content -Raw $inSvg
$icon = @"
<g id="coach-tap-spot" fill="none" stroke="#000" stroke-width="3" stroke-linecap="round">
<circle cx="$cx" cy="$($cy + 2)" r="52" stroke="#464646" stroke-width="2" stroke-dasharray="2 5"/>
<ellipse cx="$cx" cy="$($cy - 5)" rx="32" ry="25"/>
<path d="M $cx $($cy - 30) C $($cx - 9) $($cy - 16), $($cx + 9) $($cy + 4), $cx $($cy + 20)"/>
<path d="M $($cx - 26) $($cy - 12) q 6 -12 16 -4"/>
<path d="M $($cx - 24) $($cy + 2) q 8 10 14 0"/>
<path d="M $($cx + 10) $($cy - 12) q 6 -12 16 -4"/>
<path d="M $($cx + 10) $($cy + 2) q 8 10 14 0"/>
<path d="M $($cx + 4) $($cy + 18) l 5 12 M $($cx - 4) $($cy + 19) l 3 11"/>
<text x="$cx" y="$($cy + 46)" font-family="Segoe UI, Noto Sans, sans-serif" font-size="14" font-weight="bold" fill="#464646" stroke="none" text-anchor="middle">tap</text>
</g>
"@
$idx = $svg.LastIndexOf("</g>")
$svgOut = $svg.Substring(0, $idx) + $icon + $svg.Substring($idx)
[System.IO.File]::WriteAllText($outSvg, $svgOut, [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote $outSvg"
