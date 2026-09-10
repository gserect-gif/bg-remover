# Downloads the IS-Net general-use background-removal model (Apache-2.0)
# into src-tauri/models/ so it gets bundled as an app resource.
# Run this once before your first `cargo tauri build` or `cargo tauri dev`.

$ErrorActionPreference = "Stop"

$modelDir = Join-Path $PSScriptRoot "..\src-tauri\models"
New-Item -ItemType Directory -Force -Path $modelDir | Out-Null

$modelPath = Join-Path $modelDir "isnet-general-use.onnx"

if (Test-Path $modelPath) {
    Write-Host "Model already present at $modelPath — skipping download."
    exit 0
}

# Official Apache-2.0 IS-Net general-use ONNX export, published by the
# rembg project (the canonical distributor of this exact model file).
$url = "https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-general-use.onnx"

Write-Host "Downloading IS-Net general-use model (~176 MB)..."
Invoke-WebRequest -Uri $url -OutFile $modelPath -UseBasicParsing

$hash = (Get-FileHash -Path $modelPath -Algorithm SHA256).Hash
Write-Host "Downloaded. SHA-256: $hash"
Write-Host "Saved to $modelPath"
