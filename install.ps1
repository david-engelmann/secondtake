#Requires -Version 5.1
<#
.SYNOPSIS
    Install prebuilt secondtake.exe for Windows (x86_64-pc-windows-msvc) with SHA-256 verification.
.DESCRIPTION
    Resolves the requested or latest GitHub Release tag once, downloads both the Windows
    .zip archive and SHA256SUMS, verifies the archive SHA-256 digest before extraction,
    and atomically stages secondtake.exe into the target bin directory.
.PARAMETER Version
    Release tag such as v0.1.0 (default: latest GitHub release, or $SECONDTAKE_VERSION).
.PARAMETER BinDir
    Destination directory for secondtake.exe (default: $HOME\.local\bin, or $SECONDTAKE_INSTALL_DIR).
#>
[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$BinDir = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoSlug = if ($env:SECONDTAKE_REPO) { $env:SECONDTAKE_REPO } else { "david-engelmann/secondtake" }
if (-not $Version) {
    $Version = if ($env:SECONDTAKE_VERSION) { $env:SECONDTAKE_VERSION } else { "latest" }
}
if (-not $BinDir) {
    $BinDir = if ($env:SECONDTAKE_INSTALL_DIR) { $env:SECONDTAKE_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
}

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($Arch -ne "X64") {
    throw "secondtake-install: prebuilt Windows archives currently target X64 (detected $Arch). Install from source with: cargo install --git https://github.com/$RepoSlug.git --locked"
}

$Target = "x86_64-pc-windows-msvc"
$Archive = "secondtake-$Target.zip"

if ($env:SECONDTAKE_BASE_URL) {
    $BaseUrl = $env:SECONDTAKE_BASE_URL.TrimEnd("/")
} else {
    if ($Version -eq "latest") {
        $ApiUrl = "https://api.github.com/repos/$RepoSlug/releases/latest"
        $Release = Invoke-RestMethod -Uri $ApiUrl -Headers @{ "User-Agent" = "secondtake-install-ps1" }
        if (-not $Release.tag_name) {
            throw "secondtake-install: could not resolve latest tag from $ApiUrl"
        }
        $Tag = [string]$Release.tag_name
    } elseif ($Version.StartsWith("v")) {
        $Tag = $Version
    } else {
        $Tag = "v$Version"
    }
    $BaseUrl = "https://github.com/$RepoSlug/releases/download/$Tag"
}

$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("secondtake-install-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

function Copy-OrDownload([string]$SourceUri, [string]$Destination) {
    if ($SourceUri.StartsWith("file://")) {
        $LocalPath = ([System.Uri]$SourceUri).LocalPath
        Copy-Item -LiteralPath $LocalPath -Destination $Destination -Force
    } elseif (Test-Path -LiteralPath $SourceUri) {
        Copy-Item -LiteralPath $SourceUri -Destination $Destination -Force
    } else {
        Invoke-WebRequest -Uri $SourceUri -OutFile $Destination -UseBasicParsing
    }
}

try {
    $ArchivePath = Join-Path $TmpDir $Archive
    $SumsPath = Join-Path $TmpDir "SHA256SUMS"

    Write-Host "Downloading $BaseUrl/$Archive"
    Copy-OrDownload "$BaseUrl/$Archive" $ArchivePath
    Copy-OrDownload "$BaseUrl/SHA256SUMS" $SumsPath

    $ExpectedSha = $null
    foreach ($Line in (Get-Content -LiteralPath $SumsPath)) {
        $Trimmed = $Line.Trim()
        if (-not $Trimmed) { continue }
        $Parts = $Trimmed -split "\s+", 2
        if ($Parts.Count -ge 2) {
            $Name = $Parts[1].TrimStart("*").replace("./", "")
            if ($Name -eq $Archive) {
                $ExpectedSha = $Parts[0].ToLowerInvariant()
                break
            }
        }
    }
    if (-not $ExpectedSha) {
        throw "secondtake-install: archive $Archive not listed in SHA256SUMS"
    }

    $ActualSha = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ActualSha -ne $ExpectedSha) {
        throw "secondtake-install: SHA-256 mismatch for $Archive (expected $ExpectedSha, got $ActualSha)"
    }

    $UnpackDir = Join-Path $TmpDir "unpack"
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $UnpackDir -Force

    $CandidateRoot = Join-Path $UnpackDir "secondtake.exe"
    $CandidateNested = Join-Path (Join-Path $UnpackDir "secondtake-$Target") "secondtake.exe"
    if (Test-Path -LiteralPath $CandidateRoot) {
        $BinSrc = $CandidateRoot
    } elseif (Test-Path -LiteralPath $CandidateNested) {
        $BinSrc = $CandidateNested
    } else {
        throw "secondtake-install: secondtake.exe not found inside $Archive"
    }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    $StagedBin = Join-Path $BinDir ".secondtake.tmp.$PID.exe"
    $FinalBin = Join-Path $BinDir "secondtake.exe"
    Copy-Item -LiteralPath $BinSrc -Destination $StagedBin -Force
    Move-Item -LiteralPath $StagedBin -Destination $FinalBin -Force

    Write-Host "Verified SHA-256 ($ActualSha) and installed secondtake.exe to $FinalBin"
    $PathEntries = ($env:PATH -split ";") | ForEach-Object { $_.TrimEnd("\") }
    if ($PathEntries -notcontains $BinDir.TrimEnd("\")) {
        Write-Host "Note: add '$BinDir' to your user PATH to invoke 'secondtake' directly."
    }
} finally {
    if (Test-Path -LiteralPath $TmpDir) {
        Remove-Item -LiteralPath $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
