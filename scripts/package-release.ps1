#!/usr/bin/env pwsh
# Package memrl release for GitHub

param(
    [string]$Version = "0.1.3",
    [string]$OutputDir = "releases"
)

$ErrorActionPreference = "Stop"

# Get project root
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $ProjectRoot

try {
    Write-Host "📦 Packaging memrl v$Version for release..." -ForegroundColor Cyan
    
    # Ensure release builds exist
    $ReleasePath = Join-Path $ProjectRoot "target\release"
    if (-not (Test-Path $ReleasePath)) {
        Write-Host "❌ Release directory not found. Run 'cargo build --release' first." -ForegroundColor Red
        exit 1
    }
    
    # Check for executables
    $MemrlExe = Join-Path $ReleasePath "memrl.exe"
    $McpExe = Join-Path $ReleasePath "memrl-mcp.exe"
    
    if (-not (Test-Path $MemrlExe)) {
        Write-Host "❌ memrl.exe not found in target\release" -ForegroundColor Red
        exit 1
    }
    
    if (-not (Test-Path $McpExe)) {
        Write-Host "❌ memrl-mcp.exe not found in target\release" -ForegroundColor Red
        exit 1
    }
    
    # Create output directory
    $OutputPath = Join-Path $ProjectRoot $OutputDir
    if (-not (Test-Path $OutputPath)) {
        New-Item -ItemType Directory -Path $OutputPath | Out-Null
    }
    
    # Create temp staging directory
    $StagingDir = Join-Path $env:TEMP "memrl-release-staging"
    if (Test-Path $StagingDir) {
        Remove-Item -Path $StagingDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $StagingDir | Out-Null
    
    Write-Host "📋 Copying files to staging..." -ForegroundColor Yellow
    
    # Copy executables
    Copy-Item -Path $MemrlExe -Destination $StagingDir
    Copy-Item -Path $McpExe -Destination $StagingDir
    Write-Host "  ✓ Copied executables" -ForegroundColor Green
    
    # Copy documentation and license
    $DocsFiles = @("README.md", "LICENSE", "default_config.toml")
    foreach ($file in $DocsFiles) {
        $sourcePath = Join-Path $ProjectRoot $file
        if (Test-Path $sourcePath) {
            Copy-Item -Path $sourcePath -Destination $StagingDir
            Write-Host "  ✓ Copied $file" -ForegroundColor Green
        }
    }
    
    # Detect platform
    $Platform = if ($IsWindows -or $env:OS -eq "Windows_NT") {
        "windows-x64"
    } elseif ($IsMacOS) {
        "macos-x64"
    } elseif ($IsLinux) {
        "linux-x64"
    } else {
        "unknown"
    }
    
    # Create archive name
    $ArchiveName = "memrl-v$Version-$Platform"
    $ZipPath = Join-Path $OutputPath "$ArchiveName.zip"
    
    Write-Host "🗜️  Creating archive: $ArchiveName.zip" -ForegroundColor Yellow
    
    # Remove existing archive if present
    if (Test-Path $ZipPath) {
        Remove-Item -Path $ZipPath -Force
    }
    
    # Create zip archive
    Compress-Archive -Path "$StagingDir\*" -DestinationPath $ZipPath -CompressionLevel Optimal
    
    # Calculate checksum
    Write-Host "🔐 Calculating SHA256 checksum..." -ForegroundColor Yellow
    $Hash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash
    $ChecksumPath = Join-Path $OutputPath "$ArchiveName.sha256"
    "$Hash  $ArchiveName.zip" | Out-File -FilePath $ChecksumPath -Encoding utf8
    
    # Cleanup staging
    Remove-Item -Path $StagingDir -Recurse -Force
    
    # Display results
    Write-Host ""
    Write-Host "✅ Release package created successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📦 Archive: $ZipPath" -ForegroundColor Cyan
    Write-Host "📏 Size: $([math]::Round((Get-Item $ZipPath).Length / 1MB, 2)) MB" -ForegroundColor Cyan
    Write-Host "🔐 SHA256: $Hash" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Contents:" -ForegroundColor Yellow
    $archive = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
    foreach ($entry in $archive.Entries) {
        Write-Host "  - $($entry.Name)" -ForegroundColor Gray
    }
    $archive.Dispose()
    
    Write-Host ""
    Write-Host "📤 Ready to upload to GitHub release!" -ForegroundColor Green
    
} finally {
    Pop-Location
}
