<#
Rebuild prompts/coach.json from prompts/coach.txt after editing the text.

Usage (PowerShell, from the repo folder):
  .\tools\make-prompt-json.ps1
Then copy it to the tablet:
  scp prompts\coach.json root@10.11.99.1:/home/root/ghostwriter/prompts/
The next tap uses the new prompt; no restart is needed.
#>
param(
    [string] $Source = "prompts/coach.txt",
    [string] $Target = "prompts/coach.json"
)

$ErrorActionPreference = "Stop"
$text = (Get-Content -Raw -Encoding UTF8 $Source).TrimEnd()

$nonAscii = [regex]::Matches($text, '[^\x00-\x7F]')
if ($nonAscii.Count -gt 0) {
    $chars = ($nonAscii | ForEach-Object { $_.Value } | Select-Object -Unique) -join ' '
    Write-Warning "The prompt contains characters the tablet keyboard cannot type: $chars"
}

$doc = [ordered]@{
    description = "Notes coach for a new retail employee: reads a handwritten page and types back a short evaluation with recommendations. Edit prompts/coach.txt and regenerate this file with tools/make-prompt-json.ps1."
    prompt      = $text
    tools       = @("draw_text")
}

$json = ($doc | ConvertTo-Json -Depth 3) -replace "`r`n", "`n"
$full = [System.IO.Path]::GetFullPath((Join-Path (Get-Location).Path $Target))
# LF line endings and no byte-order mark, so the file is identical in git and on the tablet.
[System.IO.File]::WriteAllText($full, $json + "`n", [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote $Target ($($text.Length) characters)"
