param(
    [string]$DistPath = (Join-Path $PSScriptRoot "..\dist"),
    [string]$ReportPath = (Join-Path $PSScriptRoot "..\artifacts\offline-audit.json")
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$checks = [System.Collections.Generic.List[object]]::new()

function Add-Check {
    param([string]$Name, [bool]$Passed, [object]$Evidence)
    $checks.Add([pscustomobject]@{
        name = $Name
        passed = $Passed
        evidence = $Evidence
    })
    if (-not $Passed) {
        throw "Offline audit failed: $Name"
    }
}

$runtimeFiles = @(
    Get-ChildItem (Join-Path $root "src") -Recurse -File -Include *.ts,*.tsx |
        Where-Object { $_.Name -notmatch '\.(test|spec)\.' }
    Get-ChildItem (Join-Path $root "src-tauri\src") -Recurse -File -Include *.rs
)
$networkApiPattern = '(?i)\b(fetch|XMLHttpRequest|WebSocket|EventSource)\s*\('
$networkApiMatches = @($runtimeFiles | Select-String -Pattern $networkApiPattern)
Add-Check "runtime source contains no browser network API calls" ($networkApiMatches.Count -eq 0) @($networkApiMatches | ForEach-Object { "$($_.Path):$($_.LineNumber):$($_.Line.Trim())" })

$package = Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json
$forbiddenNpm = @("@tauri-apps/plugin-http", "@tauri-apps/plugin-websocket", "@tauri-apps/plugin-updater", "@tauri-apps/plugin-shell", "axios", "socket.io-client")
$installedForbiddenNpm = @($forbiddenNpm | Where-Object { $package.dependencies.PSObject.Properties.Name -contains $_ })
Add-Check "package.json has no runtime network plugin or client" ($installedForbiddenNpm.Count -eq 0) $installedForbiddenNpm

$cargoToml = Get-Content (Join-Path $root "src-tauri\Cargo.toml") -Raw
$forbiddenCargoMatches = @([regex]::Matches($cargoToml, '(?im)^\s*(reqwest|hyper|ureq|tokio-tungstenite|async-tungstenite)\s*=' ) | ForEach-Object { $_.Value.Trim() })
Add-Check "Cargo.toml has no direct runtime network client" ($forbiddenCargoMatches.Count -eq 0) $forbiddenCargoMatches

$config = Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json
$csp = [string]$config.app.security.csp
Add-Check "Tauri CSP blocks all renderer connections" ($csp -match "connect-src\s+'none'") $csp

$capabilityFiles = Get-ChildItem (Join-Path $root "src-tauri\capabilities") -File -Filter *.json
$capabilityEvidence = @()
$forbiddenPermissions = @()
foreach ($file in $capabilityFiles) {
    $capability = Get-Content $file.FullName -Raw | ConvertFrom-Json
    foreach ($permission in $capability.permissions) {
        $permissionName = if ($permission -is [string]) { $permission } else { $permission.identifier }
        $capabilityEvidence += "$($file.Name):$permissionName"
        if ($permissionName -match '^(http|websocket|shell|upload|updater|fs):') {
            $forbiddenPermissions += "$($file.Name):$permissionName"
        }
    }
}
Add-Check "Tauri capabilities expose no HTTP, WebSocket, shell, updater, upload, or filesystem plugin" ($forbiddenPermissions.Count -eq 0) $capabilityEvidence

$distFiles = @(Get-ChildItem $DistPath -Recurse -File -Include *.js,*.html)
$distNetworkMatches = @($distFiles | Select-String -Pattern $networkApiPattern)
Add-Check "production renderer bundle contains no network API calls" ($distNetworkMatches.Count -eq 0) @($distNetworkMatches | ForEach-Object { "$($_.Path):$($_.LineNumber)" })

$reportDirectory = Split-Path $ReportPath -Parent
New-Item -ItemType Directory -Path $reportDirectory -Force | Out-Null
$report = [pscustomobject]@{
    generatedAt = (Get-Date).ToString("o")
    root = $root
    passed = ($checks | Where-Object { -not $_.passed }).Count -eq 0
    checks = $checks
}
$report | ConvertTo-Json -Depth 8 | Set-Content -Path $ReportPath -Encoding utf8
Write-Output "Offline audit PASS: $ReportPath"
