param(
    [Parameter(Mandatory)][string]$Serial,
    [ValidateSet('armeabi-v7a', 'arm64-v8a', 'x86_64')][string]$Abi = 'armeabi-v7a',
    [string]$Ndk = "$env:ANDROID_HOME/ndk/29.0.14206865",
    [string]$NativeDirectory = '',
    # Set this only for ARM translation on an x86 Android emulator.
    [string]$Loader = ''
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$hostTag = if ($IsWindows -or $env:OS -eq 'Windows_NT') { 'windows-x86_64' } else { 'linux-x86_64' }
$toolchain = Join-Path $Ndk "toolchains/llvm/prebuilt/$hostTag/bin"
$triple = @{ 'armeabi-v7a' = 'armv7a-linux-androideabi24'; 'arm64-v8a' = 'aarch64-linux-android24'; 'x86_64' = 'x86_64-linux-android24' }[$Abi]
$cpu = @{ 'armeabi-v7a' = 'arm'; 'arm64-v8a' = 'arm64'; 'x86_64' = 'x64' }[$Abi]
$output = Join-Path $repo "artifacts/android/$Abi"
$runner = Join-Path $output 'smoke'
& (Join-Path $toolchain 'clang') "--target=$triple" (Join-Path $repo 'native/android/smoke.c') -o $runner -ldl -fPIE -pie '-Wl,-z,max-page-size=16384'
if ($LASTEXITCODE -ne 0) { throw 'Native C ABI test runner compilation failed' }
$remote = "/data/local/tmp/questpdf-$cpu"
function Invoke-Adb {
    & adb -s $Serial @args
    if ($LASTEXITCODE -ne 0) { throw "adb failed: $args" }
}
Invoke-Adb shell "mkdir -p $remote/results"
Invoke-Adb push $runner "$remote/smoke"
$jni = if ($NativeDirectory) { $NativeDirectory } else {
    Join-Path $repo "samples/flutter_android/android/app/src/main/jniLibs/$Abi"
}
foreach ($name in @('libquestpdf_android.so', 'libquestpdf_skia.so', 'libquestpdf_qpdf.so')) {
    Invoke-Adb push (Join-Path $jni $name) "$remote/$name"
}
Invoke-Adb shell "chmod 755 $remote/smoke"
Invoke-Adb shell "cd $remote && LD_LIBRARY_PATH=$remote $Loader $remote/smoke $remote/results"
$results = Join-Path $repo "artifacts/emulator-$cpu"
New-Item -ItemType Directory -Force $results | Out-Null
Invoke-Adb pull "$remote/results/." $results
$report = Get-Content -LiteralPath (Join-Path $results 'results.json') -Raw | ConvertFrom-Json
if ($report.Runtime -ne "linux-bionic-$cpu") { throw "Unexpected runtime: $($report.Runtime)" }
$pointerSize = if ($cpu -eq 'arm') { 4 } else { 8 }
if ($report.PointerSize -ne $pointerSize) { throw "Unexpected pointer size: $($report.PointerSize)" }
if ($report.Results.Count -ne 22 -or @($report.Results | Where-Object { -not $_.Passed }).Count) {
    throw 'Essential feature suite failed or is incomplete'
}
Write-Host "$Abi passed all 22 native feature tests; reports and PDFs: $results"
