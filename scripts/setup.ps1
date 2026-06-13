#!/usr/bin/env pwsh
# One-time project rename for Windows (mirror of scripts/setup.sh).
# Replaces the template identity (App Starter / app-starter / app_starter) in
# every tracked text file, then deletes itself. Idempotent: a second run finds
# nothing left to replace.
[CmdletBinding()]
param([Parameter(Position = 0)][string]$Name)

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$DefaultName  = "App Starter"
$DefaultSlug  = "app-starter"
$DefaultCrate = "app_starter"

if (-not $Name) { $Name = Read-Host 'Project display name (e.g. "Invoice Ninja")' }
if (-not $Name) { Write-Error "no name given"; exit 1 }

$slug = ($Name.ToLower() -replace '[^a-z0-9]+', '-').Trim('-')
if (-not $slug) { Write-Error "could not derive a slug from `"$Name`""; exit 1 }
$crate = $slug -replace '-', '_'

Write-Host "display name: $Name"
Write-Host "slug:         $slug"
Write-Host "crate:        $crate"
Write-Host ""

# Tracked text files only; skip generated dirs, binary assets, and the setup scripts.
$excludeDirs = @('.git', 'target', 'node_modules', 'dist', 'binaries', 'gen')
$excludeExt  = @('.png', '.ico', '.icns')
$files = git ls-files | Where-Object {
    $top = ($_ -split '/')[0]
    ($excludeDirs -notcontains $top) -and
    ($excludeExt -notcontains [System.IO.Path]::GetExtension($_)) -and
    ($_ -ne 'scripts/setup.sh') -and ($_ -ne 'scripts/setup.ps1')
}

$changed = $false
foreach ($f in $files) {
    if (-not (Test-Path $f)) { continue }
    $content = Get-Content -Raw -- $f
    if ($null -eq $content) { continue }
    if ($content.Contains($DefaultCrate) -or $content.Contains($DefaultSlug) -or $content.Contains($DefaultName)) {
        $new = $content.Replace($DefaultCrate, $crate).Replace($DefaultSlug, $slug).Replace($DefaultName, $Name)
        Set-Content -NoNewline -- $f $new
        Write-Host "updated $f"
        $changed = $true
    }
}

if (-not $changed) { Write-Host "Nothing to rename. Already set up?"; exit 0 }

Write-Host ""
Write-Host "Done. Next steps:"
Write-Host "  1. Review: git diff"
Write-Host "  2. Optional: change the bundle identifier in desktop/src-tauri/tauri.conf.json (currently com.example.$slug)"
Write-Host "  3. Commit: git add -A; git commit -m 'rename project to $slug'"

Remove-Item -- $PSCommandPath
Write-Host "(setup.ps1 removed itself)"
