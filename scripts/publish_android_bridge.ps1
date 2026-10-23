param(
    [string]$Ndk = "$env:ANDROID_HOME/ndk/29.0.14206865",
    [string]$RuntimeVersion = '10.0.10',
    [ValidateSet('armeabi-v7a', 'arm64-v8a', 'x86_64')][string]$Abi = 'arm64-v8a'
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$hostTag = if ($IsWindows -or $env:OS -eq 'Windows_NT') { 'windows-x86_64' } else { 'linux-x86_64' }
$toolchain = Join-Path $Ndk "toolchains/llvm/prebuilt/$hostTag/bin"
if (-not (Test-Path $toolchain)) { throw "Android NDK toolchain not found: $toolchain" }
$env:PATH = "$toolchain$([IO.Path]::PathSeparator)$env:PATH"
$project = Join-Path $repo 'samples/dotnet/QuestPdf.Android.Native'
$cpu = @{ 'armeabi-v7a' = 'arm'; 'arm64-v8a' = 'arm64'; 'x86_64' = 'x64' }[$Abi]
$rid = "linux-bionic-$cpu"
# Apply these properties to referenced projects during restore as well. ARM32
# Bionic ships a NativeAOT runtime pack but has no CoreCLR runtime pack.
# Only net10.0 is needed by this bridge, even though QuestPDF targets older TFMs.
& dotnet publish $project -c Release -f net10.0 -r $rid `
    -p:TargetFrameworks=net10.0 -p:PublishAot=true -p:PublishAotUsingRuntimePack=true `
    "-p:RuntimeFrameworkVersion=$RuntimeVersion" -p:CppCompilerAndLinker=clang -p:ObjCopyName=llvm-objcopy `
    -p:GenerateDocumentationFile=false
if ($LASTEXITCODE -ne 0) { throw 'NativeAOT publish failed' }
$bridge = Join-Path $project "bin/Release/net10.0/$rid/publish/questpdf_android.so"
if (-not (Test-Path $bridge)) { throw 'Publish did not produce a NativeAOT ELF shared library' }
$destination = Join-Path $repo "samples/flutter_android/android/app/src/main/jniLibs/$Abi"
New-Item -ItemType Directory -Force $destination | Out-Null
Copy-Item -LiteralPath $bridge -Destination (Join-Path $destination 'libquestpdf_android.so')
foreach ($name in @('libquestpdf_skia.so', 'libquestpdf_qpdf.so')) {
    $native = Join-Path $repo "artifacts/android/$Abi/$name"
    if (-not (Test-Path $native)) { throw "Build native Android libraries first: missing $native" }
    Copy-Item -LiteralPath $native -Destination $destination
}
Write-Host "$Abi libraries packaged in $destination"
