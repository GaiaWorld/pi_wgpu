Set-Location $PSScriptRoot
wasm-pack build --target web --dev
if ($?) {
    Write-Host 'OK: python -m http.server 8731 -d "$PSScriptRoot" 然后打开 http://localhost:8731/index.html'
}
