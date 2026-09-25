param(
    [string] $Dir = "C:\Users\dperham\source\ghostwriter-builds\templates",
    # Icon centre in native pixels (1404 x 1872). Bottom centre matches trigger_corner = "BC".
    [int] $CenterX = 702,
    [int] $CenterY = 1806
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$inPng  = Join-Path $Dir "tpl_lines_medium.png"
$outPng = Join-Path $Dir "P Lines medium coach.png"
$inSvg  = Join-Path $Dir "tpl_lines_medium.svg"
$outSvg = Join-Path $Dir "P Lines medium coach.svg"

$cx = $CenterX; $cy = $CenterY

# Spiky hair outline: alternating base points (on the head) and spike tips, left to right.
$hair = @(
    @(($cx - 34), ($cy - 4)),  @(($cx - 54), ($cy - 26)), @(($cx - 38), ($cy - 24)),
    @(($cx - 46), ($cy - 56)), @(($cx - 24), ($cy - 34)), @(($cx - 18), ($cy - 66)),
    @(($cx - 4),  ($cy - 36)), @(($cx + 6),  ($cy - 70)), @(($cx + 16), ($cy - 36)),
    @(($cx + 30), ($cy - 62)), @(($cx + 32), ($cy - 30)), @(($cx + 54), ($cy - 34)),
    @(($cx + 34), ($cy - 4))
)

# ---- PNG ----
$src = [System.Drawing.Image]::FromFile($inPng)
$bmp = New-Object System.Drawing.Bitmap $src.Width, $src.Height, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.DrawImage($src, 0, 0, $src.Width, $src.Height)

$black = [System.Drawing.Color]::Black
$white = [System.Drawing.Color]::White
$grey  = [System.Drawing.Color]::FromArgb(255, 70, 70, 70)
$pen   = New-Object System.Drawing.Pen $black, 3
$pen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$ring  = New-Object System.Drawing.Pen $grey, 2
$ring.DashStyle = [System.Drawing.Drawing2D.DashStyle]::Dot
$whiteBrush = New-Object System.Drawing.SolidBrush $white
$blackBrush = New-Object System.Drawing.SolidBrush $black
$greyBrush  = New-Object System.Drawing.SolidBrush $grey

# white disc so the icon is not drawn over the ruled lines, then the dotted tap ring
$g.FillEllipse($whiteBrush, $cx - 60, $cy - 78, 120, 130)
$g.DrawEllipse($ring, $cx - 58, $cy - 76, 116, 126)

# face (jaw and cheeks)
$g.FillEllipse($whiteBrush, $cx - 33, $cy - 30, 66, 66)
$g.DrawEllipse($pen, $cx - 33, $cy - 30, 66, 66)

# hair: filled white polygon with a black outline, covering the top of the face
[System.Drawing.PointF[]] $points = @($hair | ForEach-Object { New-Object System.Drawing.PointF ([single] $_[0]), ([single] $_[1]) })
$g.FillPolygon($whiteBrush, $points)
$g.DrawPolygon($pen, $points)

# blindfold: a black band across the eyes with two tails on the right
$g.FillRectangle($blackBrush, $cx - 34, $cy - 9, 68, 15)
$g.DrawLine((New-Object System.Drawing.Pen $black, 5), $cx + 33, $cy - 2, $cx + 50, $cy + 8)
$g.DrawLine((New-Object System.Drawing.Pen $black, 5), $cx + 33, $cy - 3, $cx + 52, $cy - 10)

# confident half-smile
$g.DrawArc($pen, $cx - 12, $cy + 8, 24, 16, 15, 150)

# label
$font = New-Object System.Drawing.Font "Segoe UI", 12, ([System.Drawing.FontStyle]::Bold)
$size = $g.MeasureString("tap", $font)
$g.DrawString("tap", $font, $greyBrush, $cx - $size.Width / 2, $cy + 32)

$g.Dispose()
$bmp.Save($outPng, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose(); $src.Dispose()
Write-Host "Wrote $outPng"

# ---- SVG (same icon in vector form) ----
$poly = ($hair | ForEach-Object { "$($_[0]),$($_[1])" }) -join " "
$icon = @"
<g id="coach-tap-spot" stroke-linejoin="round">
<ellipse cx="$cx" cy="$($cy - 13)" rx="60" ry="65" fill="#fff" stroke="none"/>
<ellipse cx="$cx" cy="$($cy - 13)" rx="58" ry="63" fill="none" stroke="#464646" stroke-width="2" stroke-dasharray="2 5"/>
<circle cx="$cx" cy="$($cy + 3)" r="33" fill="#fff" stroke="#000" stroke-width="3"/>
<polygon points="$poly" fill="#fff" stroke="#000" stroke-width="3"/>
<rect x="$($cx - 34)" y="$($cy - 9)" width="68" height="15" fill="#000"/>
<path d="M $($cx + 33) $($cy - 2) L $($cx + 50) $($cy + 8) M $($cx + 33) $($cy - 3) L $($cx + 52) $($cy - 10)" stroke="#000" stroke-width="5" fill="none"/>
<path d="M $($cx - 10) $($cy + 14) Q $cx $($cy + 26) $($cx + 12) $($cy + 16)" stroke="#000" stroke-width="3" fill="none"/>
<text x="$cx" y="$($cy + 46)" font-family="Segoe UI, Noto Sans, sans-serif" font-size="14" font-weight="bold" fill="#464646" text-anchor="middle">tap</text>
</g>
"@
$svg = Get-Content -Raw $inSvg
$idx = $svg.LastIndexOf("</g>")
$svgOut = $svg.Substring(0, $idx) + $icon + $svg.Substring($idx)
[System.IO.File]::WriteAllText($outSvg, $svgOut, [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote $outSvg"
