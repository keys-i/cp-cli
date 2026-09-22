[CmdletBinding()]
param(
    [string]$Version = "latest",
    [string]$InstallDir,
    [string]$BaseUrl = "https://github.com/keys-i/cp-cli/releases",
    [switch]$NoPathUpdate,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

function Show-Usage {
    @"
Install cp-cli from its GitHub Release.

Usage: install.ps1 [-Version VERSION] [-InstallDir DIRECTORY] [-NoPathUpdate]

Options:
  -Version VERSION       Install VERSION (for example 1.2.3); defaults to latest
  -InstallDir DIRECTORY  Install in DIRECTORY; defaults to the current user profile
  -NoPathUpdate          Do not add the installation directory to the user PATH
  -Help                  Show this help
"@
}

function Get-Target {
    if (-not [Environment]::Is64BitOperatingSystem) {
        throw "cp-cli needs 64-bit Windows"
    }

    $architecture = $env:PROCESSOR_ARCHITEW6432
    if ([string]::IsNullOrWhiteSpace($architecture)) {
        $architecture = $env:PROCESSOR_ARCHITECTURE
    }
    if ([string]::IsNullOrWhiteSpace($architecture)) {
        throw "Could not detect the Windows architecture"
    }

    switch ($architecture.ToUpperInvariant()) {
        "AMD64" { return "x86_64-pc-windows-msvc" }
        "ARM64" { return "aarch64-pc-windows-msvc" }
        default { throw "Unsupported Windows architecture: $architecture" }
    }
}

function Add-UserPathEntry([string]$PathEntry) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $entries = @($userPath -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    $exists = $entries | Where-Object {
        $_.TrimEnd("\", "/").Equals($PathEntry.TrimEnd("\", "/"), [StringComparison]::OrdinalIgnoreCase)
    }

    if ($exists) {
        Write-Host "POSSUM//INSTALL  PATH already contains $PathEntry"
        return
    }

    [Environment]::SetEnvironmentVariable("Path", (@($entries) + $PathEntry) -join ";", "User")
    $env:Path = "$env:Path;$PathEntry"
    Write-Host "POSSUM//INSTALL  Added $PathEntry to your user PATH"
    Write-Host "POSSUM//INSTALL  Open a new terminal to use cp-cli everywhere"
}

function Get-ExpectedHash([string]$ManifestPath, [string]$AssetName) {
    foreach ($line in Get-Content -LiteralPath $ManifestPath) {
        if ($line -notmatch "^([A-Fa-f0-9]{64})\s+(.+)$") {
            continue
        }

        $hash = $Matches[1]
        $fileName = $Matches[2].Trim()
        if ($fileName.StartsWith("*")) {
            $fileName = $fileName.Substring(1)
        }
        if ($fileName.StartsWith("./")) {
            $fileName = $fileName.Substring(2)
        }
        if ($fileName -eq $AssetName) {
            return $hash.ToUpperInvariant()
        }
    }

    throw "SHA256SUMS does not contain $AssetName"
}

function Get-SemVer([string]$Value) {
    $normalized = if ($Value.StartsWith("v")) { $Value.Substring(1) } else { $Value }
    $pattern = "^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
    $match = [Regex]::Match($normalized, $pattern)
    if (-not $match.Success) {
        return $null
    }
    foreach ($identifier in $match.Groups[4].Value -split "\.") {
        if ($identifier -match "^[0-9]+$" -and $identifier.Length -gt 1 -and $identifier.StartsWith("0")) {
            return $null
        }
    }
    return $normalized
}

if ($Help) {
    Show-Usage
    exit 0
}

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    if ([string]::IsNullOrWhiteSpace($env:LOCALAPPDATA)) {
        throw "LOCALAPPDATA is not set; pass -InstallDir explicitly"
    }
    $InstallDir = Join-Path $env:LOCALAPPDATA "cp-cli\bin"
}

$baseUri = [Uri]$BaseUrl
$isLocalHttp = $baseUri.Scheme -eq "http" -and $baseUri.Host -in @("localhost", "127.0.0.1", "::1")
if (($baseUri.Scheme -ne "https" -and -not $isLocalHttp) -or
    [string]::IsNullOrWhiteSpace($baseUri.Host) -or $baseUri.Query -or
    $baseUri.Fragment -or $baseUri.UserInfo) {
    throw "BaseUrl must use HTTPS, except for local HTTP testing"
}

$requestedVersion = $null
if ($Version -eq "latest") {
    $releasePath = "latest/download"
}
else {
    $requestedVersion = Get-SemVer $Version
    if (-not $requestedVersion) {
        throw "Version must be latest or a semantic version such as 1.2.3"
    }
    $releasePath = "download/v$requestedVersion"
}

$target = Get-Target
$assetName = "cp-cli-$target.zip"
$releaseUrl = "$($BaseUrl.TrimEnd('/'))/$releasePath"
$temporaryRoot = Join-Path ([IO.Path]::GetTempPath()) "cp-cli-install-$([Guid]::NewGuid().ToString('N'))"
$zipPath = Join-Path $temporaryRoot $assetName
$manifestPath = Join-Path $temporaryRoot "SHA256SUMS"
$extractPath = Join-Path $temporaryRoot "extract"
$stagedExe = $null

try {
    New-Item -ItemType Directory -Path $temporaryRoot | Out-Null
    Write-Host "POSSUM//INSTALL  Downloading cp-cli for $target"
    Invoke-WebRequest -Uri "$releaseUrl/SHA256SUMS" -OutFile $manifestPath -UseBasicParsing
    Invoke-WebRequest -Uri "$releaseUrl/$assetName" -OutFile $zipPath -UseBasicParsing

    $expectedHash = Get-ExpectedHash $manifestPath $assetName
    $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $zipPath).Hash.ToUpperInvariant()
    if ($expectedHash -ne $actualHash) {
        throw "Checksum verification failed for $assetName"
    }

    Expand-Archive -LiteralPath $zipPath -DestinationPath $extractPath
    $executables = @(Get-ChildItem -LiteralPath $extractPath -Filter "cp-cli.exe" -File -Recurse)
    if ($executables.Count -ne 1) {
        throw "Release archive must contain exactly one cp-cli.exe"
    }

    $resolvedInstallDir = [IO.Path]::GetFullPath($InstallDir)
    New-Item -ItemType Directory -Path $resolvedInstallDir -Force | Out-Null
    $installedExe = Join-Path $resolvedInstallDir "cp-cli.exe"
    $stagedExe = Join-Path $resolvedInstallDir ".cp-cli.$([Guid]::NewGuid().ToString('N')).exe"
    Copy-Item -LiteralPath $executables[0].FullName -Destination $stagedExe

    $installedVersion = ((& $stagedExe --version 2>&1) | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "Downloaded cp-cli could not run"
    }
    if ($requestedVersion -and $installedVersion -ne "cp-cli $requestedVersion") {
        throw "Downloaded cp-cli reported '$installedVersion', expected cp-cli $requestedVersion"
    }

    if (Test-Path -LiteralPath $installedExe) {
        [IO.File]::Replace($stagedExe, $installedExe, $null)
    }
    else {
        [IO.File]::Move($stagedExe, $installedExe)
    }

    if (-not $NoPathUpdate) {
        Add-UserPathEntry $resolvedInstallDir
    }
    Write-Host "POSSUM//INSTALL  Installed $installedVersion at $installedExe"
}
finally {
    if ($stagedExe -and (Test-Path -LiteralPath $stagedExe)) {
        Remove-Item -LiteralPath $stagedExe -Force
    }
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}
