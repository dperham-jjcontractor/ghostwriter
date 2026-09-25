<#
Rebuild the coach prompt JSON after editing the text files.

  prompts\coach.txt           the generic coach instructions (in git)
  prompts\store-context.txt   personal background about her store and role (NOT in git)

Usage (PowerShell, from the repo folder):
  .\tools\make-prompt-json.ps1

Writes prompts\coach.json from coach.txt alone (the generic copy bundled into the
binary) and, when store-context.txt exists, prompts\coach.local.json from both.
deploy.ps1 sends coach.local.json to the tablet when it exists. To update the
tablet without a full deploy:
  scp prompts\coach.local.json root@10.11.99.1:/home/root/ghostwriter/prompts/coach.json
The next tap uses the new prompt; no restart is needed.
#>
param(
    [string] $Source = "prompts/coach.txt",
    [string] $Context = "prompts/store-context.txt",
    [string] $Target = "prompts/coach.json",
    [string] $LocalTarget = "prompts/coach.local.json"
)

$ErrorActionPreference = "Stop"

function Write-PromptJson([string] $text, [string] $path) {
    $nonAscii = [regex]::Matches($text, '[^\x00-\x7F]')
    if ($nonAscii.Count -gt 0) {
        $chars = ($nonAscii | ForEach-Object { $_.Value } | Select-Object -Unique) -join ' '
        Write-Warning "$path contains characters the tablet keyboard cannot type: $chars"
    }
    $doc = [ordered]@{
        description = "Notes coach for a new retail employee: reads a handwritten page and types back a short evaluation with recommendations. Edit prompts/coach.txt (and prompts/store-context.txt) and regenerate with tools/make-prompt-json.ps1."
        prompt      = $text
        tools       = @("draw_text")
    }
    $json = ($doc | ConvertTo-Json -Depth 3) -replace "`r`n", "`n"
    $full = [System.IO.Path]::GetFullPath((Join-Path (Get-Location).Path $path))
    # LF line endings and no byte-order mark, so the file is identical in git and on the tablet.
    [System.IO.File]::WriteAllText($full, $json + "`n", [System.Text.UTF8Encoding]::new($false))
    Write-Host "Wrote $path ($($text.Length) characters)"
}

$coach = (Get-Content -Raw -Encoding UTF8 $Source).TrimEnd()
Write-PromptJson $coach $Target

if (Test-Path $Context) {
    $store = (Get-Content -Raw -Encoding UTF8 $Context).TrimEnd()
    Write-PromptJson ($coach + "`n`n" + $store) $LocalTarget
} else {
    Write-Host "No $Context found; only the generic prompt was built."
}
