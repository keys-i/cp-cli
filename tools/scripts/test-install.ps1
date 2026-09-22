[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$BinaryPath
)

$ErrorActionPreference = "Stop"
$projectDir = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$testRoot = Join-Path ([IO.Path]::GetTempPath()) "cp-cli-install-test-$([Guid]::NewGuid().ToString('N'))"
$releaseDir = Join-Path $testRoot "release"
$packageDir = Join-Path $testRoot "package"
$installDir = Join-Path $testRoot "install"
$corruptInstallDir = Join-Path $testRoot "corrupt-install"
$binary = [IO.Path]::GetFullPath($BinaryPath)
$expectedVersion = ((& $binary --version 2>&1) | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $expectedVersion -notmatch "^cp-cli (.+)$") {
    throw "BinaryPath must point to a runnable cp-cli binary"
}
$version = $Matches[1]

function Invoke-WebRequest {
    param(
        [Uri]$Uri,
        [string]$OutFile,
        [switch]$UseBasicParsing
    )

    Copy-Item -LiteralPath (Join-Path $releaseDir ([IO.Path]::GetFileName($Uri.AbsolutePath))) -Destination $OutFile
}

try {
    New-Item -ItemType Directory -Path $releaseDir, $packageDir | Out-Null
    Copy-Item -LiteralPath $binary -Destination (Join-Path $packageDir "cp-cli.exe")

    $architecture = $env:PROCESSOR_ARCHITEW6432
    if ([string]::IsNullOrWhiteSpace($architecture)) {
        $architecture = $env:PROCESSOR_ARCHITECTURE
    }
    switch ($architecture.ToUpperInvariant()) {
        "AMD64" { $target = "x86_64-pc-windows-msvc" }
        "ARM64" { $target = "aarch64-pc-windows-msvc" }
        default { throw "Unsupported test architecture: $architecture" }
    }

    $archive = "cp-cli-$target.zip"
    $archivePath = Join-Path $releaseDir $archive
    Compress-Archive -Path (Join-Path $packageDir "*") -DestinationPath $archivePath
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archivePath).Hash.ToLowerInvariant()
    "$hash  *./$archive" | Set-Content -LiteralPath (Join-Path $releaseDir "SHA256SUMS") -Encoding ascii

    & "$projectDir\tools\scripts\install.ps1" -Version $version -InstallDir $installDir `
        -BaseUrl "http://127.0.0.1:8080/releases" -NoPathUpdate
    if ((& "$installDir\cp-cli.exe" --version) -ne $expectedVersion) {
        throw "Installed binary reported the wrong version"
    }

    ("0" * 64) + "  $archive" | Set-Content -LiteralPath (Join-Path $releaseDir "SHA256SUMS") -Encoding ascii
    $rejected = $false
    try {
        & "$projectDir\tools\scripts\install.ps1" -Version $version -InstallDir $corruptInstallDir `
            -BaseUrl "http://127.0.0.1:8080/releases" -NoPathUpdate
    }
    catch {
        $rejected = $_.Exception.Message -like "*Checksum verification failed*"
    }
    if (-not $rejected -or (Test-Path -LiteralPath (Join-Path $corruptInstallDir "cp-cli.exe"))) {
        throw "Corrupt checksum was accepted"
    }

    foreach ($invalidVersion in @("0.02.0", "1.2", "1.2.3.", "1.2.3-01", "vlatest", "../1.2.3")) {
        $rejected = $false
        try {
            & "$projectDir\tools\scripts\install.ps1" -Version $invalidVersion -InstallDir $corruptInstallDir `
                -BaseUrl "http://127.0.0.1:8080/releases" -NoPathUpdate
        }
        catch {
            $rejected = $_.Exception.Message -like "*semantic version*"
        }
        if (-not $rejected) {
            throw "Invalid semantic version was accepted: $invalidVersion"
        }
    }

    Write-Host "installer self-test passed"
}
finally {
    if (Test-Path -LiteralPath $testRoot) {
        Remove-Item -LiteralPath $testRoot -Recurse -Force
    }
}
