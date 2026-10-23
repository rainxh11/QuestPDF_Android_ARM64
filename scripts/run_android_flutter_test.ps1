param(
    [Parameter(Mandatory)][string]$Serial,
    [ValidateSet('armeabi-v7a', 'arm64-v8a', 'x86_64')][string]$Abi = 'armeabi-v7a',
    [switch]$SoftwareRendering
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$project = Join-Path $repo 'samples/flutter_android'
$apk = Join-Path $project 'build/app/outputs/flutter-apk/app-profile.apk'
if (-not (Test-Path -LiteralPath $apk)) { throw 'Build the profile integration-test APK first' }
function Invoke-Adb {
    & adb -s $Serial @args
    if ($LASTEXITCODE -ne 0) { throw "adb failed: $args" }
}
Invoke-Adb install --no-streaming -t -r --abi $Abi $apk
Invoke-Adb shell am force-stop com.questpdf.questpdf_android_demo
Invoke-Adb logcat -c
$launchArgs = @('shell', 'am', 'start', '-n', 'com.questpdf.questpdf_android_demo/.MainActivity')
if ($SoftwareRendering) {
    $launchArgs += @('--ez', 'enable-impeller', 'false', '--ez', 'enable-software-rendering', 'true')
}
Invoke-Adb @launchArgs
$deadline = [DateTime]::UtcNow.AddMinutes(2)
$serviceUri = $null
do {
    $flutterLog = Invoke-Adb logcat -d -s flutter
    foreach ($line in $flutterLog) {
        if ($line -match 'The Dart VM service is listening on (http://127\.0\.0\.1:\d+/\S+)') {
            $serviceUri = [Uri]$Matches[1]
        }
    }
    if (-not $serviceUri) { Start-Sleep -Seconds 1 }
} while (-not $serviceUri -and [DateTime]::UtcNow -lt $deadline)
if (-not $serviceUri) { throw 'No Dart VM service found; inspect adb logcat for app startup errors' }
$forwardedPort = (& adb -s $Serial forward tcp:0 "tcp:$($serviceUri.Port)").Trim()
if ($LASTEXITCODE -ne 0) { throw 'Could not forward Dart VM service port' }
$flutterBin = Split-Path (Get-Command flutter).Source -Parent
$dart = Join-Path $flutterBin 'cache/dart-sdk/bin/dart'
$previousVmService = $env:VM_SERVICE_URL
Push-Location $project
try {
    $env:VM_SERVICE_URL = "http://127.0.0.1:$forwardedPort$($serviceUri.AbsolutePath)"
    # The host-only driver does not need Windows builds of Android native hooks.
    & $dart --disable-dart-dev --packages=.dart_tool/package_config.json test_driver/integration_test.dart
    if ($LASTEXITCODE -ne 0) { throw 'Flutter integration test failed' }
} finally {
    $env:VM_SERVICE_URL = $previousVmService
    Pop-Location
    & adb -s $Serial forward --remove "tcp:$forwardedPort"
}
